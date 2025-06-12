// main.rs
// name: alex osorio trujillo
mod error;
mod quote;
mod templates;
mod web;
mod api;

use crate::quote::read_quotes_from_file;

use axum::{

    http::{StatusCode, Method}, response::IntoResponse, routing::get, Router,
};

use clap::Parser;
use sqlx::{

    migrate::MigrateDatabase, sqlite::SqliteConnectOptions, ConnectOptions, SqlitePool
};


use std::borrow::Cow;
use std::str::FromStr;
use std::sync::Arc;
use tokio::{net::TcpListener, sync::RwLock};



use tower_http::{
    cors::{Any, CorsLayer}, services::ServeFile,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_swagger_ui::SwaggerUi;


#[derive(Parser, Debug)]
struct Args 
{
    #[arg(short, long, name = "init-from")]
    init_from: Option<std::path::PathBuf>,
    #[arg(long, env = "DATABASE_URL")]
    db_uri: Option<String>,
    #[arg(short, long, default_value = "3000", env = "PORT")]
    port: u16,
}



#[derive(Clone)]
pub struct AppState 
{
    pub db: SqlitePool,
}



fn get_db_uri_from_args_or_env(args_db_uri: Option<&str>) -> Cow<str> {
    if let Some(uri) = args_db_uri {
        uri.into()
    } else if let Ok(uri) = std::env::var("DATABASE_URL") {
        uri.into()
    } else {
        "sqlite:db/quotes.db".into()
    }
}


#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::get_quote_api,
        crate::api::get_tagged_quote_api,
        crate::api::get_random_quote_api
    ),
    components(
        schemas(crate::quote::Quote, crate::quote::JsonQuote)
    ),
    tags(
        (name = "quote_server", description = "Quote API for NBA enthusiasts")
    )
)]
struct ApiDoc;


async fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let args = Args::parse();
    let db_uri_str = get_db_uri_from_args_or_env(args.db_uri.as_deref());
    let db_uri = db_uri_str.as_ref();

    if !sqlx::Sqlite::database_exists(db_uri).await.unwrap_or(false) 
    
    {
        if let Some(colon_idx) = db_uri.rfind(':') 
        {
            let path_part = &db_uri[colon_idx + 1..];
            if let Some(last_slash_idx) = path_part.rfind('/') 
            
            {
                let dir_to_create = &path_part[..last_slash_idx];
                if !dir_to_create.is_empty() {
                    tokio::fs::create_dir_all(dir_to_create).await?;
                }


            }

        }


        sqlx::Sqlite::create_database(db_uri).await?;

    }

    let connect_options = SqliteConnectOptions::from_str(db_uri)?
        .create_if_missing(true)
        .log_statements(log::LevelFilter::Debug);

    let db_pool = SqlitePool::connect_with(connect_options).await?;
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    if let Some(path) = args.init_from 
    {
        tracing::info!("Initializing database from: {:?}", path);
        let json_quotes_vec = read_quotes_from_file(path)?;
    'outer_init_loop: for jq_item in json_quotes_vec {
            let (quote_data, tags_iter) = jq_item.to_quote();

            let mut tx = db_pool.begin().await?;
            
            let insert_res = sqlx::query!(
                "INSERT OR IGNORE INTO quotes (id, whos_there, answer_who, source) VALUES ($1, $2, $3, $4)",
                quote_data.id, quote_data.whos_there, quote_data.answer_who, quote_data.source
            )
            .execute(&mut *tx)
            .await;

            if let Err(e) = insert_res {
                tracing::error!("Failed to insert quote {}: {}", quote_data.id, e);
                tx.rollback().await?;
                continue 'outer_init_loop;
            }

            for tag_val in tags_iter {
                let normalized_tag = tag_val.trim().to_lowercase();
                if normalized_tag.is_empty() { continue; }

                let tag_res = sqlx::query!(
                    "INSERT OR IGNORE INTO quote_tags (quote_id, tag) VALUES ($1, $2)",
                    quote_data.id,
                    normalized_tag
                )
                .execute(&mut *tx)
                .await;
                if let Err(e) = tag_res {
                    tracing::error!("Failed to insert tag '{}' for quote {}: {}", normalized_tag, quote_data.id, e);
                    tx.rollback().await?;
                    continue 'outer_init_loop;
                }


            }
            if let Err(e) = tx.commit().await {
                tracing::error!("Failed to commit transaction for quote {}: {}", quote_data.id, e);
            }
        }

        tracing::info!("Database initialization complete.");

        
    }

    let app_state = AppState { db: db_pool };
    let shared_state = Arc::new(RwLock::new(app_state));

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "quote_server=debug,tower_http=info".into()),
        )
        .with(fmt::layer())
        .init();

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(DefaultOnResponse::new().level(tracing::Level::INFO));

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_origin(Any);

    let mime_favicon = "image/vnd.microsoft.icon".parse::<mime::Mime>().unwrap();
    let mime_css = mime::TEXT_CSS_UTF_8;
    
    let openapi_document: utoipa::openapi::OpenApi = ApiDoc::openapi();

    let app = Router::new()
        .route("/", get(web::get_main_page_handler))
        .route_service(
            "/style.css",
            ServeFile::new_with_mime("assets/static/style.css", &mime_css),
        )
        .route_service(


            "/favicon.ico",
            ServeFile::new_with_mime("assets/static/favicon.ico", &mime_favicon),
        )
        .nest("/api/v1", api::router())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi_document.clone()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .fallback(handler_404)
        .layer(cors)
        .layer(trace_layer)
        .with_state(shared_state);
    let listener = TcpListener::bind(&format!("127.0.0.1:{}", args.port)).await?;
    tracing::info!("Quote server listening on http://127.0.0.1:{}", args.port);
    axum::serve(listener, app).await?;
    Ok(())


}


async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Oops! Page not found.")

}

#[tokio::main]
async fn main() {
    if let Err(err) = run_app().await {
        eprintln!("quote_server: error: {:#}", err);
        std::process::exit(1);
    }



}

