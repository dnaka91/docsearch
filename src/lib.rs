//! # docsearch
//!
//! Use the latest search index from `rustdoc` to find the docs.rs (or stdlib) URL for any item in
//! a crate by its fully qualified name.
//!
//! ## Example
//!
//! ```no_run
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() -> docsearch::Result<()> {
//! let fqn = "anyhow::Result".parse::<docsearch::Fqn>().unwrap();
//! let index = docsearch::search(fqn.crate_name(), None).await?;
//! let link = index.find_link(&fqn).unwrap();
//!
//! println!("{}", link);
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![deny(
    rust_2018_idioms,
    clippy::all,
    clippy::pedantic,
    clippy::print_stderr,
    clippy::print_stdout
)]
#![allow(clippy::missing_errors_doc)]

use std::collections::BTreeMap;

pub use semver::Version;
use serde::{Deserialize, Serialize};

pub use crate::fqn::{Fqn, ParseError as FqnParseError};

mod crates;
mod fqn;
mod index;

/// List of crates in the stdlib index.
pub(crate) const STD_CRATES: &[&str] = &["alloc", "core", "proc_macro", "std", "test"];

/// Custom result type of docsearch for convenience.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors that can happen when retrieving and parsing a crate index.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed deserializing JSON")]
    Json(#[from] serde_json::Error),
    #[error("a HTTP request failed")]
    Http(#[from] reqwest::Error),
    #[error("invalid semantic version string")]
    SemVer(#[from] semver::Error),
    #[error("the version part was missing in `{0}`")]
    MissingVersion(String),
    #[error("couldn't find the index path in a response body")]
    IndexNotFound,
    #[error("index didn't contain information for the requested crate")]
    CrateDataMissing,
    #[error("version was not in the expected `search-index<X.X.X>.js` format but `{0}`")]
    InvalidVersionFormat(String),
}

/// Parsed crate index that contains the mappings from [`Fqn`]s to their URL for direct linking.
#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Index {
    /// Name of the crate as a single index file can contain multiple crate indexes.
    name: String,
    /// Version of the parsed index.
    version: Version,
    /// Mapping from FQNs to URL paths.
    mapping: BTreeMap<String, String>,
    /// Whether this index is for the stdlib.
    std: bool,
}

impl Index {
    #[must_use]
    pub fn find_link(&self, fqn: &Fqn) -> Option<String> {
        self.mapping.get(fqn.as_ref()).map(|link| {
            if self.std {
                format!("https://doc.rust-lang.org/nightly/{}", link)
            } else {
                format!("https://docs.rs/{}/{}/{}", self.name, self.version, link)
            }
        })
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Search for the given crate name and optionally a fixed version. This is the main entry point to
/// retrieve an [`Index`] and further query that index for [`Fqn`]s.
///
/// # Example
///
/// Download the index for the `anhow` crate and get the docs.rs link for the `anyhow::Result` item.
///
/// ```no_run
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> docsearch::Result<()> {
/// let fqn = "anyhow::Result".parse::<docsearch::Fqn>().unwrap();
/// let index = docsearch::search(fqn.crate_name(), None).await?;
/// let link = index.find_link(&fqn).unwrap();
///
/// println!("{}", link);
/// # Ok(())
/// # }
/// ```
pub async fn search(name: &str, version: Option<Version>) -> Result<Index> {
    let (mapping, std) = if STD_CRATES.contains(&name) {
        (crates::get_std().await?, true)
    } else {
        (crates::get_docsrs(name, version).await?, false)
    };

    Ok(transform(name, mapping, std)?)
}

/// Convert the downloaded index and convert it into a FQN to URL path mapping for each contained
/// crate. Additionally attach some extra data like the version and whether the crate is considered
/// part of the stdlib.
fn transform(name: &str, (version, index): (Version, String), std: bool) -> Result<Index> {
    let mappings = index::load(&index)?;

    mappings
        .into_iter()
        .find(|(crate_name, _)| crate_name == name)
        .map(|(name, mapping)| Index {
            name,
            version: version.clone(),
            mapping,
            std,
        })
        .ok_or(Error::CrateDataMissing)
}
