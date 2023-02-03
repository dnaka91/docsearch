//! Use the latest search index from `rustdoc` to find the docs.rs (or stdlib) URL for any item in a
//! crate by its [simple path](https://doc.rust-lang.org/stable/reference/paths.html#simple-paths).
//!
//! # Example
//!
//! Please have a look at the [`start_search`] function for an example of how to use this crate, as
//! it is the main entry point. In addition, you can check out the `examples` directory in the
//! repository.
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

use std::{borrow::Cow, collections::BTreeMap};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
pub use crate::{simple_path::SimplePath, version::Version};

mod crates;
pub mod error;
mod index;
mod simple_path;
mod version;

/// List of crates in the stdlib index.
pub(crate) const STD_CRATES: &[&str] = &["alloc", "core", "proc_macro", "std", "test"];

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
            format!("https://doc.rust-lang.org/nightly/{link}")
        } else {
            format!("https://docs.rs/{}/{}/{link}", self.name, self.version)
        })
    }
}

/// Search for the given crate name and optionally a fixed version. This is the main entry point to
/// retrieve an [`Index`] and further query that index for [`SimplePath`]s.
///
/// # Example
///
/// Download the index for the `anyhow` crate and get the docs.rs link for the `anyhow::Result`
/// item.
///
/// ```no_run
/// use anyhow::Result;
/// use docsearch::{SimplePath, Version};
///
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() -> Result<()> {
///     // First parse the search query into a `SimplePath`. This ensures the query is actually
///     // usable and allows to provide additional info.
///     let query = "anyhow::Result".parse::<SimplePath>().unwrap();
///
///     // Initiate a new search. It allows to not depend on a specific HTTP crate and instead
///     // pass the task to the developer (that's you).
///     let state = docsearch::start_search(query.crate_name(), Version::Latest);
///     // First, download the HTML page content to find the URL to the search index.
///     let content = download_url(state.url()).await?;
///
///     // Now try to find the the link to the actual search index.
///     let state = state.find_index(&content)?;
///     // Next, download the search index content.
///     let content = download_url(state.url()).await?;
///
///     // Lastly, transform the search index content into an `Index` instance, containing all
///     // information to create webpage links to an item within the scope of the requested crate.
///     let index = state.transform_index(&content)?;
///
///     // Now we can use the index to query for our initial item.
///     let link = index.find_link(&query).unwrap();
///
///     // And print out the resolved web link to it.
///     println!("{link}");
///
///     Ok(())
/// }
///
/// /// Simple helper function to download any HTTP page with `reqwest`, using a normal GET request.
/// async fn download_url(url: &str) -> Result<String> {
///     reqwest::Client::builder()
///         .redirect(reqwest::redirect::Policy::limited(10))
///         .build()?
///         .get(url)
///         .send()
///         .await?
///         .error_for_status()?
///         .text()
///         .await
///         .map_err(Into::into)
/// }
/// ```
#[must_use]
pub fn start_search(name: &str, version: Version) -> SearchPage<'_> {
    let std = STD_CRATES.contains(&name);
    let url = crates::get_page_url(std, name, &version);

    SearchPage {
        name,
        version,
        std,
        url,
    }
}

/// Initial state when starting a new search. Use the [`Self::url`] function to get the URL to
/// download content from. The web page content must then be passed to [`Self::find_index`] to get
/// to the next state.
pub struct SearchPage<'a> {
    name: &'a str,
    version: Version,
    std: bool,
    url: Cow<'static, str>,
}

impl<'a> SearchPage<'a> {
    /// URL to content that should be retrieved and passed to [`Self::find_index`].
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Try to find the index in the content downloaded from [`Self::url`], effectively transferring
    /// to the next state in retrieving an `Index` instance.
    pub fn find_index(self, body: &str) -> Result<SearchIndex<'a>> {
        let (version, url) = crates::find_index_url(self.std, self.name, self.version, body)?;

        Ok(SearchIndex {
            name: self.name,
            version,
            std: self.std,
            url,
        })
    }
}

/// Second and last state in retrieving a search index. Use the [`Self::url`] function to get the
/// search index URL to download. The index's content must be passed to [`Self::transform_index`] to
/// create the final [`Index`] instance.
pub struct SearchIndex<'a> {
    name: &'a str,
    version: Version,
    std: bool,
    url: String,
}

impl<'a> SearchIndex<'a> {
    /// URL to the search index that should be retrieved and passed to [`Self::transform_index`].
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Try to transform the raw index content into a simple "path-to-URL" mapping for each
    /// contained crate.
    pub fn transform_index(self, index_content: &str) -> Result<Index> {
        let mappings = index::load(index_content)?;

        mappings
            .into_iter()
            .find(|(crate_name, _)| crate_name == self.name)
            .map(|(name, mapping)| Index {
                name,
                version: self.version.clone(),
                mapping,
                std: self.std,
            })
            .ok_or(Error::CrateDataMissing)
    }
}
