//! Location and retrieval of the index data from the docs page of a crate (or the stdlib docs).

use log::debug;
use reqwest::redirect::Policy;
use semver::Version;

use crate::{Error, Result};

/// Base URL for the `docs.rs` docs service.
const DOCSRS_URL: &str = "https://docs.rs";

/// Download the search index for a single crate from <https://docs.rs>, optionally a specific
/// version of it.
///
/// ## Version extraction
///
/// If a specific version was passed as argument no further extraction is done as it is already
/// known, but in case it wasn't given it is extracted from the returned URL after sending a web
/// request to the service.
///
/// The URL's path is currently in the format `<crate>/<version>/<crate>`. Therefore, the path
/// segment at index `1` is taken and converted into a semver.
pub async fn get_docsrs(name: &str, version: Option<Version>) -> Result<(Version, String)> {
    let page_url = version.as_ref().map_or_else(
        || format!("{}/{}", DOCSRS_URL, name),
        |v| format!("{}/{}/{}", DOCSRS_URL, name, v),
    );

    debug!("getting content at {}", page_url);
    let resp = reqwest::Client::builder()
        .redirect(Policy::limited(10))
        .build()?
        .get(page_url)
        .send()
        .await?
        .error_for_status()?;

    let version = match version {
        Some(v) => v,
        None => resp
            .url()
            .path_segments()
            .unwrap() // safe as we always have a proper https://... URL
            .nth(1)
            .ok_or_else(|| Error::MissingVersion(resp.url().to_string()))?
            .parse()?,
    };
    let body = resp.text().await?;

    let index_path = find_url(&body).ok_or(Error::IndexNotFound)?;
    debug!("found index path: {}", index_path);
    let index_url = format!("{}/{}/{}/{}", DOCSRS_URL, name, version, index_path);

    debug!("getting index at {}", index_url);
    let index = reqwest::get(index_url)
        .await?
        .error_for_status()?
        .text()
        .await?;

    Ok((version, index))
}

/// URL for the index page of the stdlib std crate.
const STDLIB_INDEX_URL: &str = "https://doc.rust-lang.org/nightly/std/index.html";
/// Base URL for the stdlib docs.
const STDLIB_URL: &str = "https://doc.rust-lang.org/nightly";

/// Download the latest stdbib search index.
///
/// ## Version extraction
///
/// The version of the stdlib is always extracted as part of retrieving the index file and can not
/// be set by the caller. In contrast to [`get_docsrs`], the version is not extracted from the URL
/// but from the index's name. The file name has the format `search-index<version>.js`.
pub async fn get_std() -> Result<(Version, String)> {
    debug!("getting content at {}", STDLIB_INDEX_URL);
    let body = reqwest::get(STDLIB_INDEX_URL)
        .await?
        .error_for_status()?
        .text()
        .await?;

    let index_path = find_url(&body).ok_or(Error::IndexNotFound)?;
    debug!("found index path: {}", index_path);
    let index_url = format!("{}/{}", STDLIB_URL, index_path);

    let version = index_path
        .strip_prefix("search-index")
        .and_then(|url| url.strip_suffix(".js"))
        .ok_or_else(|| Error::InvalidVersionFormat(index_path.to_owned()))?
        .parse()?;

    debug!("getting index at {}", index_url);
    let index = reqwest::get(index_url)
        .await?
        .error_for_status()?
        .text()
        .await?;

    Ok((version, index))
}

/// Try to find the URL for the search index from a crate's main page. This is currently a `div` tag
/// with the id `rustdoc-vars` and an attribute `data-search-js` (or `data-search-index-js` for the
/// stdlib docs) that contains the wanted URL.
///
/// As the URL is currently unique, it's relatively safe to assume that there will be only one
/// string in the whole page that starts with `".../search-index` and ends with `.js"`. Therefore
/// a simple string extraction is sufficient and we don't have to pull in big dependencies to parse
/// the HTML content first.
fn find_url(body: &str) -> Option<&str> {
    if let Some(start) = body.find("\"../search-index") {
        if let Some(end) = body[start..].find(".js\"") {
            return Some(&body[start + 4..start + end + 3]);
        }
    }

    None
}
