use std::collections::HashMap;

use anyhow::Result;

mod crates;
mod index;

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
