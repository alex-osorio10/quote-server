// templates.rs
use crate::quote::Quote;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub quote: Quote,
    pub stylesheet: &'static str,
    pub tags: String,
}

impl IndexTemplate {
    pub fn new(quote: Quote, tags: String) -> Self {
        Self {
            quote,
            stylesheet: "/style.css",
            tags,
        }
    }
}
