use std::{str::FromStr, fmt::{Display, self}};

use serde::{Deserialize, Serialize};

/// Crate version that can be either the latest available or a specific one.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Version {
    /// The latest available version.
    Latest,
    /// A specific, [`semver`]-compliant version.
    SemVer(semver::Version),
}

impl FromStr for Version {
    type Err = semver::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s == "latest" {
            Self::Latest
        } else {
            Self::SemVer(s.parse()?)
        })
    }
}

impl Display for Version{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Latest => f.write_str("latest"),
            Self::SemVer(v) => v.fmt(f),
        }
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::Latest
    }
}
