//! Handling of the index data and its transformation in a more usable format as well as a mapping
//! of simple paths to rustdoc URL.

use std::{
    collections::{BTreeMap, HashMap},
    fmt,
};

use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_repr::Deserialize_repr;

use crate::error::{Error, Result};

#[cfg(feature = "index-v1")]
mod v1;
#[cfg(feature = "index-v2")]
mod v2;

#[cfg_attr(test, derive(Clone, Copy, Eq, PartialEq, serde::Serialize))]
enum Version {
    #[cfg(feature = "index-v1")]
    V1,
    #[cfg(feature = "index-v2")]
    V2,
    V3,
}

impl Version {
    fn detect(index: &str) -> Option<Self> {
        #[cfg(feature = "index-v1")]
        if index.starts_with(r#"var N=null,E="",T="t",U="u",searchIndex={};"#) {
            return Some(Self::V1);
        }

        #[cfg(feature = "index-v2")]
        if index.ends_with(r#"addSearchOptions(searchIndex);initSearch(searchIndex);"#) {
            return Some(Self::V2);
        }

        if index.ends_with(r#"if (window.initSearch) {window.initSearch(searchIndex)};"#)
            || index.trim_end().ends_with(
                r#"if (typeof exports !== 'undefined') {exports.searchIndex = searchIndex};"#,
            )
        {
            Some(Self::V3)
        } else {
            None
        }
    }
}

/// Whole index data after transformation.
#[cfg_attr(test, derive(PartialEq, Eq, serde::Serialize))]
struct IndexData {
    /// Mapping from crate name to data.
    crates: HashMap<String, CrateData>,
}

/// Crate data after transformation.
#[cfg_attr(test, derive(PartialEq, Eq, serde::Serialize))]
struct CrateData {
    /// Doc string of the crate.
    #[allow(dead_code)]
    doc: String,
    /// Data for each individual item of the crate.
    items: Vec<IndexItem>,
    /// Parent paths that help to construct full paths and URLs from item information.
    paths: Vec<(ItemType, String)>,
    // aliases
}

/// Index data for a single item after transformation.
///
/// Taken from: <https://github.com/rust-lang/rust/blob/eba3228b2a9875d268ff3990903d04e19f6cdb0c/src/librustdoc/html/render/mod.rs#L84>.
#[cfg_attr(test, derive(PartialEq, Eq, serde::Serialize))]
struct IndexItem {
    /// The type of item.
    ty: ItemType,
    /// Simple name without path.
    name: String,
    /// Resolved, full path.
    path: String,
    /// Short, one line description. Can contain HTML tags and is likely truncated with the `…`
    /// character.
    #[allow(dead_code)]
    desc: String,
    /// Index to the parent item, if it belongs to another item.
    parent_idx: Option<usize>,
    // search_type
}

/// Different item types that can appear in the rust docs to identify the kind of item.
///
/// Taken from: <https://github.com/rust-lang/rust/blob/eba3228b2a9875d268ff3990903d04e19f6cdb0c/src/librustdoc/formats/item_type.rs>.
#[derive(Clone, Copy, Debug, Deserialize_repr)]
#[cfg_attr(test, derive(PartialEq, Eq, serde::Serialize))]
#[repr(u8)]
enum ItemType {
    Module = 0,
    ExternCrate = 1,
    Import = 2,
    Struct = 3,
    Enum = 4,
    Function = 5,
    Typedef = 6,
    Static = 7,
    Trait = 8,
    Impl = 9,
    TyMethod = 10,
    Method = 11,
    StructField = 12,
    Variant = 13,
    Macro = 14,
    Primitive = 15,
    AssocType = 16,
    Constant = 17,
    AssocConst = 18,
    Union = 19,
    ForeignType = 20,
    Keyword = 21,
    OpaqueTy = 22,
    ProcAttribute = 23,
    ProcDerive = 24,
    TraitAlias = 25,
}

impl ItemType {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Module => "mod",
            Self::ExternCrate => "externcrate",
            Self::Import => "import",
            Self::Struct => "struct",
            Self::Union => "union",
            Self::Enum => "enum",
            Self::Function => "fn",
            Self::Typedef => "type",
            Self::Static => "static",
            Self::Trait => "trait",
            Self::Impl => "impl",
            Self::TyMethod => "tymethod",
            Self::Method => "method",
            Self::StructField => "structfield",
            Self::Variant => "variant",
            Self::Macro => "macro",
            Self::Primitive => "primitive",
            Self::AssocType => "associatedtype",
            Self::Constant => "constant",
            Self::AssocConst => "associatedconstant",
            Self::ForeignType => "foreigntype",
            Self::Keyword => "keyword",
            Self::OpaqueTy => "opaque",
            Self::ProcAttribute => "attr",
            Self::ProcDerive => "derive",
            Self::TraitAlias => "traitalias",
        }
    }

    const fn from_raw(value: u8) -> Option<Self> {
        Some(match value {
            0 => Self::Module,
            1 => Self::ExternCrate,
            2 => Self::Import,
            3 => Self::Struct,
            4 => Self::Enum,
            5 => Self::Function,
            6 => Self::Typedef,
            7 => Self::Static,
            8 => Self::Trait,
            9 => Self::Impl,
            10 => Self::TyMethod,
            11 => Self::Method,
            12 => Self::StructField,
            13 => Self::Variant,
            14 => Self::Macro,
            15 => Self::Primitive,
            16 => Self::AssocType,
            17 => Self::Constant,
            18 => Self::AssocConst,
            19 => Self::Union,
            20 => Self::ForeignType,
            21 => Self::Keyword,
            22 => Self::OpaqueTy,
            23 => Self::ProcAttribute,
            24 => Self::ProcDerive,
            25 => Self::TraitAlias,
            _ => return None,
        })
    }
}

/// The whole index data for a crate. It usually contains only one entry for the crate it was
/// generated for. The stdlib index is a special case where multiple crates like `std` and `alloc`
/// are included.
#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq, serde::Serialize))]
struct RawIndexData {
    /// Mapping from crate name to raw index data.
    #[serde(flatten)]
    crates: HashMap<String, RawCrateData>,
}

/// Crate index data in its raw form. All elements are vectors and the same index over all of them
/// contain the information for a single item.
///
/// Taken from: <https://github.com/rust-lang/rust/blob/eba3228b2a9875d268ff3990903d04e19f6cdb0c/src/librustdoc/html/render/cache.rs#L121>.
#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq, serde::Serialize))]
struct RawCrateData {
    /// Doc string for the crate. Seems to always be `github\u{2002}crates-io\u{2002}docs-rs`.
    doc: String,
    /// Type of item.
    #[serde(deserialize_with = "t")]
    t: Vec<ItemType>,
    /// Simple name without the path.
    n: Vec<String>,
    /// Module path of the item. This uses previous items as reference and an empty value means to
    /// use the value of the previous item. Similar to being still in the same _directory_.
    #[serde(deserialize_with = "q")]
    q: BTreeMap<usize, String>,
    /// Short, one line description of the item. Maybe contain HTML tags and is likely truncated.
    d: Vec<String>,
    /// Index of the parent item. For example if the item is a method, it references the index of
    /// the struct/enum/... it belongs to.
    ///
    /// A value of `0` means that no parent exists. Therefore, indexes start at `1` and need to be
    /// adjusted to access the right item in the other vectors.
    i: Vec<usize>,
    // f: search type
    /// Further information about the parent item that helps in constructing the full path of an
    /// item with parent.
    ///
    /// For example a method `baz` as part of the struct `Bar` in the module `foo` will only have
    /// the basic path `foo` as the [`Self::q`] value only describes module paths. This field
    /// contains the parent name `Bar` (and its item type) so that the full path `foo::Bar::baz` can
    /// be constructed.
    p: Vec<(ItemType, String)>,
    // a: aliases
}

/// Parse and transform a raw index file and convert it into mappings from paths to URLs that can be
/// used to generate permalinks to the items' docs page.
///
/// This is the combination of the internal functions [`load_raw`], [`transform`] and
/// [`generate_mapping`].
pub fn load(index: &str) -> Result<HashMap<String, BTreeMap<String, String>>> {
    let raw = match Version::detect(index) {
        Some(Version::V3) => load_raw(index)?,
        #[cfg(feature = "index-v2")]
        Some(Version::V2) => v2::load_raw(index)?,
        #[cfg(feature = "index-v1")]
        Some(Version::V1) => v1::load_raw(index)?,
        None => return Err(Error::UnsupportedIndexVersion),
    };

    Ok(generate_mapping(transform(raw)))
}

/// Extract the JSON content from the index data and run it through [`serde`] to transform it into
/// usable data structures.
///
/// The index data looks basically as follows:
///
/// ```js
/// var searchIndex = JSON.parse('{\
/// "cratename":{"doc":"...","t":[1],"n":["Name"],"q":["path"],"d":[""],"i":[0],"f":[null],"p":[]}\
/// }');
/// if (window.initSearch) {window.initSearch(searchIndex)};
/// ```
///
/// After the initial JavaScript line, the file contains one line of JSON data for each crate
/// contained in the index. These are extracted and the surrounding `{` and `}` delimiters added
/// again to create a valid JSON object.
///
/// For further explanation of the individual fields of a single crate entry, looks at the docs of
/// [`RawIndexData`] and [`RawCrateData`].
fn load_raw(index: &str) -> Result<RawIndexData> {
    let json = {
        let mut json = index
            .lines()
            .filter_map(|l| {
                if l.starts_with('"') {
                    l.strip_suffix('\\')
                } else {
                    None
                }
            })
            .fold(String::from("{"), |mut json, l| {
                json.push_str(l);
                json
            });
        json.push('}');

        // Inverse operation of:
        // <https://github.com/rust-lang/rust/blob/eba3228b2a9875d268ff3990903d04e19f6cdb0c/src/librustdoc/html/render/cache.rs#L175-L190>.
        json.replace("\\\\\"", "\\\"")
            .replace(r"\'", "'")
            .replace(r"\\", r"\")
    };

    serde_json::from_str(&json).map_err(Into::into)
}

/// Convert from the index data into a more usable data structure that contains one full data set
/// for each item of the crate.
///
/// The raw structure contains a data driven layout (likely to reduce size of the JSON format) what
/// means that the data for a single entry is not contained in a single object but instead spread
/// over vectors of data, each one representing one field.
///
/// Data for a single entry can be retrieved by index and this transformation does exactly that to
/// get back a whole structure of information for each item.
///
/// ## Implementation
///
/// The separate elements of each item are combined back together with the [`Iterator::zip`] method.
/// A nice side effect is that we don't have to cope for differences in vector sizes (which should
/// not exist but can theoretically) as it stops as soon as one of the iterators returns [`None`].
///
/// The path field is only present if it changes compared to the previous item to reduce index size.
/// The previous path is kept around thanks to the [`Iterator::fold`] method and only updated if the
/// current path is present. Otherwise the old value is used. This increases data size but makes
/// usage much more convenient and less error prone as the path doesn't need to be searched every
/// time it is accessed.
///
/// Parent indexes are transformed from a `usize` into an `Option<usize>` to erase the special
/// handling of the `0` value and indexes are reduced by `1` to allow proper indexing.
fn transform(raw: RawIndexData) -> IndexData {
    IndexData {
        crates: raw
            .crates
            .into_iter()
            .map(|(name, mut raw_data)| {
                let length = raw_data.t.len();
                let (items, _) = raw_data
                    .t
                    .into_iter()
                    .enumerate()
                    .zip(raw_data.n)
                    .zip(raw_data.d)
                    .zip(raw_data.i)
                    .fold(
                        (Vec::with_capacity(length), String::new()),
                        |(mut items, path), ((((pos, t), n), d), i)| {
                            let path = raw_data.q.remove(&pos).unwrap_or(path);
                            items.push(IndexItem {
                                ty: t,
                                name: n,
                                path: path.clone(),
                                desc: d,
                                parent_idx: if i > 0 { Some(i - 1) } else { None },
                            });
                            (items, path)
                        },
                    );

                (
                    name,
                    CrateData {
                        doc: raw_data.doc,
                        items,
                        paths: raw_data.p,
                    },
                )
            })
            .collect(),
    }
}

/// Generate a mapping from the transformed index data. This simply calls [`generate_crate_mapping`]
/// for each crate in the index to do the actual transformation of item data.
fn generate_mapping(data: IndexData) -> HashMap<String, BTreeMap<String, String>> {
    data.crates
        .into_iter()
        .map(|(name, data)| (name, generate_crate_mapping(data)))
        .collect()
}

/// Generate the simple path for each item in the crate data and its URL variant as used by
/// `rustdoc`. This allows to get a direct mapping from simple path to URL path, which can further
/// be used to create a permalink to the rustdoc page.
///
/// ## Implementation
///
/// The path is usually in the form of `<module>::<item>` where the module path can contain further
/// `::`. If the item has a parent its form is `<module::<parent_item>::<item>`.
///
/// The URL path is slightly different, with additional information about the type. The basic form
/// is `<module>/<type>.<item>.html` where the module can contain further slashes `/` and the type
/// defines the item type like `Struct`, `Enum` and others.
///
/// If the item has a parent its form is `<module>/<parent_type>.<parent_item>.html#<type>.<item>`.
/// The original type/item combination is replaced with the parent information and the actual item
/// part is moved into a path fragment to become an anchor. That is, because an item with parent
/// doesn't have its own page but is a part of the parents page.
fn generate_crate_mapping(data: CrateData) -> BTreeMap<String, String> {
    let paths = data.paths;

    data.items
        .into_iter()
        .map(|item| {
            let full_path = if let Some(idx) = item.parent_idx {
                format!("{}::{}::{}", item.path, paths[idx].1, item.name)
            } else {
                format!("{}::{}", item.path, item.name)
            };

            let url = if let Some(parent) = item.parent_idx.map(|i| &paths[i]) {
                format!(
                    "{}/{}.{}.html#{}.{}",
                    item.path.replace("::", "/"),
                    parent.0.as_str(),
                    parent.1,
                    item.ty.as_str(),
                    item.name
                )
            } else {
                format!(
                    "{}/{}.{}.html",
                    item.path.replace("::", "/"),
                    item.ty.as_str(),
                    item.name
                )
            };

            (full_path, url)
        })
        .collect()
}

fn t<'de, D>(deserializer: D) -> Result<Vec<ItemType>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(VecItemTypeVisitor)
}

struct VecItemTypeVisitor;

impl<'de> Visitor<'de> for VecItemTypeVisitor {
    type Value = Vec<ItemType>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("item types either as an array of IDs or a string of ASCII chars")
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.bytes()
            .map(|ascii| {
                ascii
                    .is_ascii_uppercase()
                    .then(|| ItemType::from_raw(ascii - b'A'))
                    .flatten()
                    .ok_or_else(|| {
                        E::custom(format!("invalid ASCII character `{}`", ascii as char))
                    })
            })
            .collect()
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut list = Vec::with_capacity(seq.size_hint().unwrap_or(0));

        while let Some(element) = seq.next_element()? {
            list.push(element);
        }

        Ok(list)
    }
}

fn q<'de, D>(deserializer: D) -> Result<BTreeMap<usize, String>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(VecPathVisitor)
}

struct VecPathVisitor;

impl<'de> Visitor<'de> for VecPathVisitor {
    type Value = BTreeMap<usize, String>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("item types either as an array of IDs or a string of ASCII chars")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Value {
            String(String),
            Tuple((usize, String)),
        }

        let mut map = BTreeMap::new();
        let mut position = 0;

        while let Some(element) = seq.next_element::<Value>()? {
            let (key, value) = match element {
                Value::String(name) => {
                    if name.is_empty() {
                        position += 1;
                        continue;
                    }
                    (position, name)
                }
                Value::Tuple((position, name)) => (position, name),
            };

            map.insert(key, value);
            position += 1;
        }

        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use insta::glob;
    use serde_test::Token;

    use super::*;

    #[test]
    fn test_version_detect() {
        glob!("fixtures/*.js", |path| {
            let input = fs::read_to_string(path).unwrap();
            let data = Version::detect(&input);
            insta::assert_yaml_snapshot!(data);
        });
    }

    #[allow(clippy::bind_instead_of_map)]
    #[test]
    fn test_load_raw() {
        glob!("fixtures/*.js", |path| {
            let input = fs::read_to_string(path).unwrap();
            let data = Version::detect(&input).and_then(|v| match v {
                #[cfg(feature = "index-v1")]
                Version::V1 => Some(v1::load_raw(&input).unwrap()),
                #[cfg(feature = "index-v2")]
                Version::V2 => Some(v2::load_raw(&input).unwrap()),
                Version::V3 => Some(load_raw(&input).unwrap()),
            });
            insta::assert_yaml_snapshot!(data);
        });
    }

    #[allow(clippy::bind_instead_of_map)]
    #[test]
    fn test_transform() {
        glob!("fixtures/*.js", |path| {
            let input = fs::read_to_string(path).unwrap();
            let data = Version::detect(&input)
                .and_then(|v| match v {
                    #[cfg(feature = "index-v1")]
                    Version::V1 => Some(v1::load_raw(&input).unwrap()),
                    #[cfg(feature = "index-v2")]
                    Version::V2 => Some(v2::load_raw(&input).unwrap()),
                    Version::V3 => Some(load_raw(&input).unwrap()),
                })
                .map(transform);
            insta::assert_yaml_snapshot!(data);
        });
    }

    #[allow(clippy::bind_instead_of_map)]
    #[test]
    fn test_generate_mapping() {
        glob!("fixtures/*.js", |path| {
            let input = fs::read_to_string(path).unwrap();
            let data = Version::detect(&input)
                .and_then(|v| match v {
                    #[cfg(feature = "index-v1")]
                    Version::V1 => Some(v1::load_raw(&input).unwrap()),
                    #[cfg(feature = "index-v2")]
                    Version::V2 => Some(v2::load_raw(&input).unwrap()),
                    Version::V3 => Some(load_raw(&input).unwrap()),
                })
                .map(transform)
                .map(generate_mapping);
            insta::assert_yaml_snapshot!(data);
        });
    }

    #[test]
    fn test_t() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Wrapper {
            #[serde(deserialize_with = "t")]
            value: Vec<ItemType>,
        }

        let wrapper = Wrapper {
            value: vec![ItemType::Module],
        };

        serde_test::assert_de_tokens(
            &wrapper,
            &[
                Token::Struct {
                    name: "Wrapper",
                    len: 1,
                },
                Token::Str("value"),
                Token::Str("A"),
                Token::StructEnd,
            ],
        );
        serde_test::assert_de_tokens(
            &wrapper,
            &[
                Token::Struct {
                    name: "Wrapper",
                    len: 1,
                },
                Token::Str("value"),
                Token::Seq { len: Some(1) },
                Token::I64(0),
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn test_q() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Wrapper {
            #[serde(deserialize_with = "q")]
            value: BTreeMap<usize, String>,
        }

        let wrapper = Wrapper {
            value: [(0, "test".to_owned()), (2, "test::two".to_owned())].into(),
        };

        serde_test::assert_de_tokens(
            &wrapper,
            &[
                Token::Struct {
                    name: "Wrapper",
                    len: 1,
                },
                Token::Str("value"),
                Token::Seq { len: Some(2) },
                Token::Seq { len: Some(2) },
                Token::I64(0),
                Token::Str("test"),
                Token::SeqEnd,
                Token::Seq { len: Some(2) },
                Token::I64(2),
                Token::Str("test::two"),
                Token::SeqEnd,
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
        serde_test::assert_de_tokens(
            &wrapper,
            &[
                Token::Struct {
                    name: "Wrapper",
                    len: 1,
                },
                Token::Str("value"),
                Token::Seq { len: Some(3) },
                Token::Str("test"),
                Token::Str(""),
                Token::Str("test::two"),
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }
}
