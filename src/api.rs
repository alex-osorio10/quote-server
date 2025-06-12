// api.rs
use crate::quote::{self, JsonQuote};
use crate::AppState;
use crate::authjwt::{self, Claims, Registration};
use axum::{
    extract::{Path, State, Json},
    http::{self, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post}, 
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;


pub fn router() -> Router<Arc<RwLock<AppState>>> {
    Router::new()
        .route("/quote/{quote_id}", get(get_quote_api))


        .route("/tagged-quote", post(get_tagged_quote_api))
        .route("/random-quote", get(get_random_quote_api))
        .route("/register", post(register))
        .route("/add-quote", post(add_quote))
}

async fn get_quote_data_for_api(db: &sqlx::SqlitePool, quote_id: &str) -> Result<Response, http::StatusCode> {
    match quote::get_quote_by_id_from_db(db, quote_id).await {
        Ok((quote_obj, tags_vec)) => {
            let json_response = JsonQuote::new(&quote_obj, tags_vec);



            Ok(Json(json_response).into_response())
        }
        Err(e) => {
            tracing::warn!("API: quote fetch failed for id {}: {}", quote_id, e);
            if matches!(e, sqlx::Error::RowNotFound) {
                Err(StatusCode::NOT_FOUND)
            } else 
            {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }

        }


    }




}

#[utoipa::path(
    get,
    path = "/api/v1/quote/{quote_id}",
    responses(
        (status = 200, description = "Get a quote by id", body = JsonQuote),
        (status = 404, description = "No matching quote found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("quote_id" = String, Path, description = "ID of the quote to retrieve")
    )
)]
pub async fn get_quote_api(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(quote_id): Path<String>,
) -> impl IntoResponse {
    let state_guard = app_state.read().await;
    get_quote_data_for_api(&state_guard.db, &quote_id).await
}


#[utoipa::path(
    post,
    path = "/api/v1/tagged-quote",
    request_body = Vec<String>,
    responses(
        (status = 200, description = "Get a quote by matching tags", body = JsonQuote),
        (status = 404, description = "No quote found for the given tags"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_tagged_quote_api(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(tags_payload): Json<Vec<String>>,
) -> impl IntoResponse {
    tracing::info!("API: get tagged quote with tags: {:?}", tags_payload);
    let state_guard = app_state.read().await;

    let db_pool = &state_guard.db;
    match quote::get_tagged_quote_id_from_db(db_pool, tags_payload.iter().map(String::as_str)).await {
        Ok(Some(found_quote_id)) => get_quote_data_for_api(db_pool, &found_quote_id).await,
        Ok(None) => {
            tracing::info!("API: No quote found for tags: {:?}", tags_payload);



            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("API: Database error fetching tagged quote: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }


    }


}


#[utoipa::path(
    get,
    path = "/api/v1/random-quote",
    responses(
        (status = 200, description = "Get a random quote", body = JsonQuote),
        (status = 404, description = "No quotes available in the database"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_random_quote_api(
    State(app_state): State<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    let state_guard = app_state.read().await;
    let db_pool = &state_guard.db;
    match quote::get_random_quote_id_from_db(db_pool).await {
        Ok(found_quote_id) => get_quote_data_for_api(db_pool, &found_quote_id).await,
        Err(e) => {
            tracing::warn!("API: Failed to get random quote: {}", e);
            if matches!(e, sqlx::Error::RowNotFound) {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}


#[utoipa::path(
    post,
    path = "/api/v1/register",
    request_body = Registration,
    responses(
        (status = 200, description = "Successfully registered and received token", body = authjwt::AuthBody),
        (status = 401, description = "Registration failed due to invalid key", body = authjwt::AuthError),
    )
)]
pub async fn register(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(registration): Json<Registration>,
) -> impl IntoResponse {
    let state_guard = app_state.read().await;
    match authjwt::register_and_create_token(&state_guard, &registration) {
        Ok(response) => response.into_response(),
        Err(e) => e.into_response(),
    }
}


#[utoipa::path(
    post,
    path = "/api/v1/add-quote",
    request_body = JsonQuote,
    responses(
        (status = 201, description = "Quote added successfully"),
        (status = 400, description = "Bad request (e.g., invalid quote format)"),
        (status = 401, description = "Authentication error"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn add_quote(
    _claims: Claims,
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(quote_to_add): Json<JsonQuote>,
) -> impl IntoResponse {
    let state_guard = app_state.read().await;
    match quote::add_quote_to_db(&state_guard.db, quote_to_add).await {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => {
            tracing::error!("API: Failed to add quote: {}", e);
            
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }


    }

}

