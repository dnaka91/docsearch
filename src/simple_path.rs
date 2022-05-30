//! Implementation of the simple path according to the Rust spec as well as helpers in regards to
//! this crate to make easy use of the path.

use std::{
    fmt::{self, Display},
    str::FromStr,
};

use crate::{error::ParseError, STD_CRATES};

/// Path for any item within a crate (or just the crate itself) like `std::vec::Vec`,
/// `anyhow::Result` or `thiserror`.
///
/// New paths are created by the [`FromStr`] trait:
///
/// ```rust
/// "anyhow::Result".parse::<docsearch::SimplePath>().unwrap();
/// ```
pub struct SimplePath(String, usize);

impl SimplePath {
    /// Get back the original string.
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Crate name part of this path.
    ///
    /// This can be used as argument for the [`start_search`](crate::start_search) function.
    #[must_use]
    pub fn crate_name(&self) -> &str {
        &self.0[..self.1]
    }

    /// Whether this path is for the standard library.
    #[must_use]
    pub fn is_std(&self) -> bool {
        STD_CRATES.contains(&self.crate_name())
    }

    /// Whether the path only contains the crate name and no item information.
    pub(crate) fn is_crate_only(&self) -> bool {
        self.0.len() == self.1
    }
}

impl FromStr for SimplePath {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Self::Err::TooShort);
        }

        if !s.split("::").all(is_identifier) {
            return Err(Self::Err::InvalidIdentifier);
        }

        let index = s.find("::").unwrap_or(s.len());

        Ok(Self(s.to_owned(), index))
    }
}

impl AsRef<str> for SimplePath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for SimplePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Check whether the given value is an identifier or a keyword.
///
/// An identifier is any nonempty Unicode string of the following form:
///
/// Either
///
/// - The first character has property [`XID_start`].
/// - The remaining characters have property [`XID_continue`].
///
/// Or
///
/// - The first character is `_`.
/// - The identifier is more than one character. `_` alone is not an identifier.
/// - The remaining characters have property [`XID_continue`].
///
/// [`XID_start`]: http://unicode.org/cldr/utility/list-unicodeset.jsp?a=%5B%3AXID_Start%3A%5D&abb=on&g=&i=
/// [`XID_continue`]: http://unicode.org/cldr/utility/list-unicodeset.jsp?a=%5B%3AXID_Continue%3A%5D&abb=on&g=&i=
fn is_identifier_or_keyword(value: &str) -> bool {
    fn variant_one(first_char: char, value: &str) -> bool {
        unicode_ident::is_xid_start(first_char)
            && value.chars().skip(1).all(unicode_ident::is_xid_continue)
    }

    fn variant_two(first_char: char, value: &str) -> bool {
        first_char == '_'
            && value.chars().skip(1).count() > 0
            && value.chars().skip(1).all(unicode_ident::is_xid_continue)
    }

    let first_char = match value.chars().next() {
        Some(ch) => ch,
        None => return false,
    };

    variant_one(first_char, value) || variant_two(first_char, value)
}

/// Check whether the given value is a raw identifier.
///
/// A raw identifier is any nonempty Unicode string of the following form:
///
/// - The value starts with `r#`.
/// - The followed content is a valid [identifier or keyword](is_identifier_or_keyword).
/// - The followed content is none of: `crate`, `self`, `super`, `Self`.
fn is_raw_identifier(value: &str) -> bool {
    const KEYWORDS: &[&str] = &["crate", "self", "super", "Self"];

    value
        .strip_prefix("r#")
        .map(|value| is_identifier_or_keyword(value) && !KEYWORDS.contains(&value))
        .unwrap_or_default()
}

/// Check whether the given value is a non-keyword identifier.
///
/// A non-keyword identifier is any nonempty Unicode string of the following form:
///
/// - The value is a valid [identifier or keyword](is_identifier_or_keyword).
/// - The value is not a [strict] or [reserved] keyword.
///
/// [strict]: https://doc.rust-lang.org/stable/reference/keywords.html#strict-keywords
/// [reserved]: https://doc.rust-lang.org/stable/reference/keywords.html#reserved-keywords
fn is_non_keyword_identifier(value: &str) -> bool {
    const STRICT_KEYWORDS: &[&str] = &[
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
        "use", "where", "while", "async", "await", "dyn",
    ];
    const RESERVED_KEYWORDS: &[&str] = &[
        "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof",
        "unsized", "virtual", "yield",
    ];

    is_identifier_or_keyword(value)
        && !STRICT_KEYWORDS.contains(&value)
        && !RESERVED_KEYWORDS.contains(&value)
}

/// Check whether the given value is an identifier.
///
/// An identifier is any nonempty Unicode string of the following form:
///
/// Either
///
/// - The value is [raw identifier](is_raw_identifier).
///
/// Or
///
/// - The value is a [non-keyword identifier](is_non_keyword_identifier).
fn is_identifier(value: &str) -> bool {
    is_non_keyword_identifier(value) || is_raw_identifier(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid() {
        let inputs = &["anyhow", "anyhow::Result", "special::__", "__", "r#unsafe"];

        for input in inputs {
            assert!(input.parse::<SimplePath>().is_ok());
        }
    }

    #[test]
    fn parse_invalid() {
        let inputs = &["", "a::::b", "::", "_", "unsafe", "Self", "r#Self"];

        for input in inputs {
            assert!(input.parse::<SimplePath>().is_err());
        }
    }
}
