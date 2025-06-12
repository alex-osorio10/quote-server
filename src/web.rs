// web.rs
use crate::quote::{self, Quote};
use crate::templates::IndexTemplate;
use crate::AppState;
use askama::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};

use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Deserialize, Debug)]
pub struct GetQuoteParams {
    id: Option<String>,
    tags: Option<String>,
}

pub async fn get_main_page_handler(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<GetQuoteParams>,
) -> Result<Response, StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;

    if let Some(tags_query_str) = params.tags {
        if !tags_query_str.trim().is_empty() {
            tracing::debug!("Web: Fetching quote by tags: {}", tags_query_str);
            let search_tags_vec: Vec<&str> = tags_query_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            if !search_tags_vec.is_empty() {
                match quote::get_tagged_quote_id_from_db(db, search_tags_vec.into_iter()).await {
                    Ok(Some(found_quote_id)) => {
                        let uri = format!("/?id={}", found_quote_id);
                        return Ok(Redirect::to(&uri).into_response());
                    }
                    Ok(None) => {
                        tracing::debug!("Web: No quote found for tags, getting random.");
                    }
                    Err(e) => {
                        tracing::error!("Web: DB error fetching tagged quote: {}", e);
                    }
                }
            }
        }
    }

    if let Some(id_str) = params.id {
        tracing::debug!("Web: Fetching quote by ID: {}", id_str);
        match quote::get_quote_by_id_from_db(db, &id_str).await {
            Ok((quote, tags)) => {
                let template = IndexTemplate::new(quote, tags.join(", "));
                return Ok(Html(template.render().unwrap()).into_response());
            }
            Err(e) => {
                tracing::warn!(
                    "Web: Could not find quote by ID {}: {}. Getting random.",
                    id_str,
                    e
                );
            }
        }
    }

    tracing::debug!("Web: Fetching random quote ID for redirect.");
    match quote::get_random_quote_id_from_db(db).await {
        Ok(random_id) => {
            let uri = format!("/?id={}", random_id);
            Ok(Redirect::to(&uri).into_response())
        }
        Err(e) => {
            tracing::error!("Web: Could not get any random quote from DB: {}", e);
            let fallback_quote = Quote {
                id: "error".to_string(),
                whos_there: "Oh no!".to_string(),

                answer_who:
                    "The quote you were looking for decided to take a day off. Try another!"
                        .to_string(),

                source: "The Server".to_string(),
            };
            let template = IndexTemplate::new(fallback_quote, "error".to_string());

            Ok(Html(template.render().unwrap()).into_response())
        }
    }
}
