//! Different types of errors that can occur when using this crate.
#![allow(clippy::module_name_repetitions)]

/// Custom result type of docsearch for convenience.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors that can happen when retrieving and parsing a crate index.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("failed deserializing JSON")]
    Json(#[from] serde_json::Error),
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

/// Errors that can happen when parsing a [`SimplePath`](crate::SimplePath).
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// The value is too short to represent a simple path.
    #[error("The value is too short")]
    TooShort,
    /// One (and possibly more) of the segments aren't valid identifiers.
    #[error("One or more segments aren't valid identifiers")]
    InvalidIdentifier,
}
