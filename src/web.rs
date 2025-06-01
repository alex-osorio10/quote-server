// web.rs
use crate::quote::{self}; 
use crate::templates::IndexTemplate;
use crate::AppState;
use askama::Template; 
use axum::{
    extract::{Query, State}, http::StatusCode, response::{Html, IntoResponse},
};

use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Deserialize, Debug)]
pub struct GetQuoteParams {
    id: Option<String>, tags: Option<String>,
}

pub async fn get_main_page_handler( 
    State(app_state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<GetQuoteParams>,
) -> impl IntoResponse {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    let result: Result<(crate::quote::Quote, Vec<String>), Box<dyn std::error::Error + Send + Sync>> = async {
        if let Some(id_str) = params.id {
            tracing::debug!("Web: Fetching quote by ID: {}", id_str);
            return quote::get_quote_by_id_from_db(db, &id_str).await.map_err(Into::into);

        }

        if let Some(tags_query_str) = params.tags 
        {
            tracing::debug!("Web: Fetching quote by tags: {}", tags_query_str);
            let search_tags_vec: Vec<&str> = tags_query_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            if !search_tags_vec.is_empty() 
            
            {
                
                if let Some(found_quote_id) = quote::get_tagged_quote_id_from_db(db, search_tags_vec.into_iter()).await? {
                    tracing::debug!("Web: Found tagged quote ID: {}", found_quote_id);
                    return quote::get_quote_by_id_from_db(db, &found_quote_id).await.map_err(Into::into);
                } else 
                
                {
                    tracing::debug!("Web: No quote found for tags: {}", tags_query_str);
                }


            }


        }

        tracing::debug!("Web: Fetching random quote");
        let random_id = quote::get_random_quote_id_from_db(db).await?;
        quote::get_quote_by_id_from_db(db, &random_id).await.map_err(Into::into)
    }

    .await;

    match result 
    {
        Ok((selected_quote, tags_vec)) => {
            let tag_string = tags_vec.join(", ");
            let template = IndexTemplate::new(selected_quote, tag_string);
            match template.render() {
                Ok(html_output) => Html(html_output).into_response(),
                Err(e) => {
                    tracing::error!("Web: Template rendering error: {}", e);


                    (StatusCode::INTERNAL_SERVER_ERROR, format!("Error rendering page: {}", e)).into_response()
                }


            }
        }


        Err(e) => {
            tracing::warn!("Web: Failed to get quote for page: {}", e);
            let err_msg = if e.downcast_ref::<sqlx::Error>().map_or(false, |sqlx_err| matches!(sqlx_err, sqlx::Error::RowNotFound)) {
                "The quote you were looking for decided to take a day off. Try another!"
            } else 
            
            {
                "We hit a snag trying to fetch a quote. Please try again."
            };
            (StatusCode::NOT_FOUND, err_msg).into_response()
        }


    }


}

