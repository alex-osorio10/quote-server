use std::collections::HashSet;
use std::ops::Deref;
use std::path::Path;

use crate::KnockKnockError;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Jsonquote {
    id: String,
    whos_there: String,
    answer_who: String,
    tags: HashSet<String>,
    source: String,
}

pub struct Quote {
    pub id: String,
    pub whos_there: String,
    pub answer_who: String,
    pub quote_source: String,
}

pub fn read_quotes<P: AsRef<Path>>(quotes_path: P) -> Result<Vec<Jsonquote>, KnockKnockError> {
    let f = std::fs::File::open(quotes_path.as_ref())?;
    let quotes = serde_json::from_reader(f)?;
    Ok(quotes)
}

impl Jsonquote {
    pub fn to_quote(&self) -> (Quote, impl Iterator<Item = &str>) {
        let quote = Quote {
            id: self.id.clone(),
            whos_there: self.whos_there.clone(),
            answer_who: self.answer_who.clone(),
            quote_source: self.source.clone(),
        };
        let tags = self.tags.iter().map(String::deref);
        (quote, tags)
    }
}
