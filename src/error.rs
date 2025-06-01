// error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QuoteAppError {
    #[error("could not find quote file: {0}")]
    QuotesNotFound(#[from] std::io::Error),


    #[error("could not read quote data: {0}")]
    QuoteMisformat(#[from] serde_json::Error),


    #[error("invalid database uri: {0}")]
    _InvalidDbUri(String), 

    #[error("database operation failed: {0}")]
    DatabaseError(#[from] sqlx::Error),

    
}
