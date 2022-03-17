//! Use the latest search index from `rustdoc` to find the docs.rs (or stdlib) URL for any item in a
//! crate by its [simple path](https://doc.rust-lang.org/stable/reference/paths.html#simple-paths).
//!
//! # Example
//!
//! In this example we search for the `anyhow::Result` item and print the web link to the
//! corresponding `docs.rs` page.
//!
//! ```no_run
//! use docsearch::{Result, SimplePath, Version};
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> Result<()> {
//!     let path = "anyhow::Result".parse::<SimplePath>().unwrap();
//!     let index = docsearch::search(path.crate_name(), Version::Latest).await?;
//!     let link = index.find_link(&path).unwrap();
//!
//!     println!("{}", link);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Feature flags
//!
//! The following features flags enable support for older versions of the search index. If they're
//! not enabled, the retrieving the [`Index`] for a crate might fail. These should be enabled or
//! disabled based on the requirements to what crates will be searched for (if known).
//!
//! The features listed are **enabled by default**.
//!
//! - `index-v2` enables support to parse the slightly outdated index format. This is needed if
//! parsing of older crates that haven't be update in a while is required.
//! - `index-v1` enables support for the even older index format. Nowadays it's rarely found and
//! this is only needed to parse very old crates that haven't been updated in a long while.

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

use serde::{Deserialize, Serialize};

pub use crate::{
    simple_path::{ParseError, SimplePath},
    version::Version,
};

mod crates;
mod index;
mod simple_path;
mod version;

/// List of crates in the stdlib index.
pub(crate) const STD_CRATES: &[&str] = &["alloc", "core", "proc_macro", "std", "test"];

/// Custom result type of docsearch for convenience.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors that can happen when retrieving and parsing a crate index.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
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
    #[error("the used index version is currently not supported")]
    UnsupportedIndexVersion,
    #[cfg(feature = "index-v1")]
    #[error("failed to parse the V1 index")]
    InvalidV1Index(#[from] IndexV1Error),
}

/// Errors that can happen when parsing the old V1 index.
#[cfg(feature = "index-v1")]
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IndexV1Error {
    #[error("missing reference variable in index")]
    MissingReference,
    #[error("failed deserializing reference content")]
    InvalidReferenceJson(#[source] serde_json::Error),
    #[error("failed parsing the JavaScript parts of the index")]
    InvalidIndexJavaScript(String),
    #[error("failed deserializing transformed index")]
    InvalidIndexJson(#[source] serde_json::Error),
}

/// Parsed crate index that contains the mappings from [`SimplePath`]s to their URL for direct
/// linking.
#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Index {
    /// Name of the crate.
    pub name: String,
    /// Version of the crate.
    pub version: Version,
    /// Mapping from simple paths to URL paths.
    pub mapping: BTreeMap<String, String>,
    /// Whether this index is for the standard library.
    pub std: bool,
}

impl Index {
    #[must_use]
    pub fn find_link(&self, path: &SimplePath) -> Option<String> {
        let link = if path.is_crate_only() {
            path.crate_name()
        } else {
            self.mapping.get(path.as_ref())?
        };

        Some(if self.std {
            format!("https://doc.rust-lang.org/nightly/{}", link)
        } else {
            format!("https://docs.rs/{}/{}/{}", self.name, self.version, link)
        })
    }
}

/// Search for the given crate name and optionally a fixed version. This is the main entry point to
/// retrieve an [`Index`] and further query that index for [`SimplePath`]s.
///
/// # Example
///
/// Download the index for the `anhow` crate and get the docs.rs link for the `anyhow::Result` item.
///
/// ```no_run
/// use docsearch::{Result, SimplePath, Version};
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<()> {
/// let path = "anyhow::Result".parse::<SimplePath>().unwrap();
/// let index = docsearch::search(path.crate_name(), Version::Latest).await?;
/// let link = index.find_link(&path).unwrap();
///
/// println!("{}", link);
/// # Ok(())
/// # }
/// ```
pub async fn search(name: &str, version: Version) -> Result<Index> {
    let (mapping, std) = if STD_CRATES.contains(&name) {
        (crates::get_std().await?, true)
    } else {
        (crates::get_docsrs(name, version).await?, false)
    };

    Ok(transform(name, mapping, std)?)
}

/// Convert the downloaded index and convert it into a simple path to URL path mapping for each
/// contained crate. Additionally attach some extra data like the version and whether the crate is
/// considered part of the stdlib.
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
