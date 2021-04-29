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
    let (version, index) = crates::search(name, version).await?;
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
