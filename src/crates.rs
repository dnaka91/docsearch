//! Location and retrieval of the index data from the docs page of a crate (or the stdlib docs).

use std::borrow::Cow;

use tracing::debug;

use crate::{Error, Result, Version};

/// Base URL for the `docs.rs` docs service.
const DOCSRS_URL: &str = "https://docs.rs";

pub(crate) fn get_page_url(std: bool, name: &str, version: &Version) -> Cow<'static, str> {
    if std {
        Cow::Borrowed(STDLIB_INDEX_URL)
    } else {
        Cow::Owned(format!("{DOCSRS_URL}/{name}/{version}/{name}/"))
    }
}

pub(crate) fn find_index_url(
    std: bool,
    name: &str,
    version: Version,
    body: &str,
) -> Result<(Version, String)> {
    let index_path = find_url(body).ok_or(Error::IndexNotFound)?;
    debug!("found index path: {index_path}");

    if std {
        let version = index_path
            .strip_prefix("search-index")
            .and_then(|url| url.strip_suffix(".js"))
            .ok_or_else(|| Error::InvalidVersionFormat(index_path.clone()))?
            .parse()?;

        Ok((version, format!("{STDLIB_URL}/{index_path}")))
    } else {
        let url = format!("{DOCSRS_URL}/{name}/{version}/{index_path}");
        Ok((version, url))
    }
}

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

/// URL for the index page of the stdlib std crate.
pub const STDLIB_INDEX_URL: &str = "https://doc.rust-lang.org/nightly/std/index.html";
/// Base URL for the stdlib docs.
const STDLIB_URL: &str = "https://doc.rust-lang.org/nightly";

/// Download the latest stdbib search index.
///
/// ## Version extraction
///
/// The version of the stdlib is always extracted as part of retrieving the index file and can not
/// be set by the caller. In contrast to [`get_docsrs`], the version is not extracted from the URL
/// but from the index's name. The file name has the format `search-index<version>.js`.

/// Try to find the URL for the search index from a crate's main page. This is currently a `div` tag
/// with the id `rustdoc-vars` and an attribute `data-search-js` (or `data-search-index-js` for the
/// stdlib docs) that contains the wanted URL.
///
/// As the URL is currently unique, it's relatively safe to assume that there will be only one
/// string in the whole page that starts with `".../search-index` and ends with `.js"`. Therefore
/// a simple string extraction is sufficient and we don't have to pull in big dependencies to parse
/// the HTML content first.
fn find_url(body: &str) -> Option<String> {
    let v1 = body
        .rfind("src=\"../search-index-")
        .and_then(|pos| body[pos..].split_once("src=\"../"))
        .and_then(|(_, start)| start.split_once('\"'))
        .map(|(url, _)| url.to_owned());

    let v2 = body
        .rsplit_once("data-search-index-js=\"../")
        .and_then(|(_, start)| start.split_once('\"'))
        .map(|(url, _)| url.to_owned());

    let v3 = body
        .rsplit_once("data-resource-suffix=\"")
        .and_then(|(_, start)| start.split_once('\"'))
        .map(|(suffix, _)| format!("search-index{suffix}.js"));

    v3.or(v2).or(v1)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use insta::glob;

    use super::*;

    #[test]
    fn test_find_index_path() {
        glob!("fixtures/*.html", |path| {
            let input = fs::read_to_string(path).unwrap();
            let data = find_url(&input).unwrap();
            insta::assert_yaml_snapshot!(data);
        });
    }
}
