// quote.rs
use crate::error::QuoteAppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::collections::HashSet;
use std::path::Path;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct JsonQuote {
    pub id: String,
    pub whos_there: String,
    pub answer_who: String,
    pub tags: HashSet<String>,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Quote {
    pub id: String,
    pub whos_there: String,
    pub answer_who: String,
    pub source: String,
}

pub fn read_quotes_from_file<P: AsRef<Path>>(
    quotes_path: P,
) -> Result<Vec<JsonQuote>, QuoteAppError> {
    let f = std::fs::File::open(quotes_path.as_ref())?;
    let quotes: Vec<JsonQuote> = serde_json::from_reader(f)?;
    Ok(quotes)
}

impl JsonQuote {
    pub fn new(quote: &Quote, tags: Vec<String>) -> Self {
        let tags_set = tags.into_iter().collect();
        Self {
            id: quote.id.clone(),
            whos_there: quote.whos_there.clone(),
            answer_who: quote.answer_who.clone(),
            tags: tags_set,
            source: quote.source.clone(),
        }
    }

    pub fn to_quote(&self) -> (Quote, impl Iterator<Item = &str>) {
        let quote = Quote {
            id: self.id.clone(),
            whos_there: self.whos_there.clone(),
            answer_who: self.answer_who.clone(),
            source: self.source.clone(),
        };
        let tags_iter = self.tags.iter().map(String::as_str);
        (quote, tags_iter)
    }
}

pub async fn get_quote_by_id_from_db(
    db: &SqlitePool,
    quote_id: &str,
) -> Result<(Quote, Vec<String>), sqlx::Error> {
    let quote = sqlx::query_as!(
        Quote,
        "SELECT id, whos_there, answer_who, source FROM quotes WHERE id = $1;",
        quote_id
    )
    .fetch_one(db)
    .await?;

    let tags: Vec<String> =
        sqlx::query_scalar!("SELECT tag FROM quote_tags WHERE quote_id = $1;", quote_id)
            .fetch_all(db)
            .await?;

    Ok((quote, tags))
}

pub async fn get_random_quote_id_from_db(db: &SqlitePool) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!("SELECT id FROM quotes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(db)
        .await
}

pub async fn get_tagged_quote_id_from_db<'a, I>(
    db: &SqlitePool,
    search_tags: I,
) -> Result<Option<String>, sqlx::Error>
where
    I: Iterator<Item = &'a str> + Send,
{
    let mut tx = db.begin().await?;
    sqlx::query("DROP TABLE IF EXISTS temp_search_tags;")
        .execute(&mut *tx)
        .await?;
    sqlx::query("CREATE TEMPORARY TABLE temp_search_tags (tag_query TEXT);")
        .execute(&mut *tx)
        .await?;
    let mut has_tags = false;
    for tag_item in search_tags {
        let normalized_tag = tag_item.trim().to_lowercase();
        if !normalized_tag.is_empty() {
            sqlx::query("INSERT INTO temp_search_tags (tag_query) VALUES ($1);")
                .bind(normalized_tag)
                .execute(&mut *tx)
                .await?;
            has_tags = true;
        }
    }

    if !has_tags {
        tx.commit().await?;
        return Ok(None);
    }

    let query_str = "
        SELECT qt.quote_id
        FROM quote_tags qt
        JOIN temp_search_tags tst ON LOWER(qt.tag) = tst.tag_query
        GROUP BY qt.quote_id
        HAVING COUNT(DISTINCT tst.tag_query) = (SELECT COUNT(*) FROM temp_search_tags)
        ORDER BY RANDOM()
        LIMIT 1;
    ";

    let result_id: Option<String> = sqlx::query_scalar(query_str)
        .fetch_optional(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(result_id)
}

pub async fn add_quote_to_db(db: &SqlitePool, quote: JsonQuote) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    sqlx::query!(
        "INSERT INTO quotes (id, whos_there, answer_who, source) VALUES ($1, $2, $3, $4)",
        quote.id,
        quote.whos_there,
        quote.answer_who,
        quote.source,
    )
    .execute(&mut *tx)
    .await?;

    for tag in quote.tags {
        sqlx::query!(
            "INSERT INTO quote_tags (quote_id, tag) VALUES ($1, $2)",
            quote.id,
            tag,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}
