#![forbid(unsafe_code)]
#![deny(
    rust_2018_idioms,
    clippy::all,
    clippy::pedantic,
    clippy::print_stderr,
    clippy::print_stdout
)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_errors_doc)]

use std::collections::HashMap;

mod crates;
mod index;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed deserializing JSON")]
    Json(#[from] serde_json::Error),
    #[error("a HTTP request failed")]
    Http(#[from] reqwest::Error),
    #[error("the version part was missing in `{0}`")]
    MissingVersion(String),
    #[error("couldn't find the index path in a response body")]
    IndexNotFound,
    #[error("version was not in the expected `search-index<X.X.X>.js` format but `{0}`")]
    InvalidVersionFormat(String),
}

#[derive(Debug)]
pub struct CrateIndex {
    pub name: String,
    pub version: String,
    pub mapping: HashMap<String, String>,
}

pub async fn search(name: &str, version: Option<&str>) -> Result<Vec<CrateIndex>> {
    Ok(transform(crates::search(name, version).await?)?)
}

pub async fn get_std() -> Result<Vec<CrateIndex>> {
    Ok(transform(crates::get_std().await?)?)
}

fn transform((version, index): (String, String)) -> Result<Vec<CrateIndex>> {
    let mappings = index::load(&index)?;

    Ok(mappings
        .into_iter()
        .map(|(name, mapping)| CrateIndex {
            name,
            version: version.clone(),
            mapping,
        })
        .collect())
}
