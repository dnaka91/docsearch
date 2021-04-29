use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Debug)]
struct IndexData {
    crates: HashMap<String, CrateData>,
}

#[derive(Debug)]
struct CrateData {
    doc: String,
    items: Vec<IndexItem>,
    paths: Vec<(ItemType, String)>, // aliases
}

#[derive(Debug)]
struct IndexItem {
    ty: ItemType,
    name: String,
    path: String,
    desc: String,
    parent_idx: Option<usize>,
    // search_type
}

#[derive(Clone, Copy, Debug, Deserialize_repr)]
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
}

#[derive(Debug, Deserialize)]
struct RawIndexData {
    #[serde(flatten)]
    crates: HashMap<String, RawCrateData>,
}

#[derive(Debug, Deserialize)]
struct RawCrateData {
    doc: String,
    t: Vec<ItemType>,
    n: Vec<String>,
    q: Vec<String>,
    d: Vec<String>,
    i: Vec<usize>,
    // f
    p: Vec<(ItemType, String)>,
    // a
}

pub fn load(index: &str) -> Result<HashMap<String, HashMap<String, String>>> {
    Ok(generate_mapping(transform(load_raw(index)?)))
}

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
        json.replace("\\\\\"", "\\\"")
            .replace(r"\'", "'")
            .replace(r"\\", r"\")
    };

    serde_json::from_str(&json).map_err(Into::into)
}

fn transform(raw: RawIndexData) -> IndexData {
    IndexData {
        crates: raw
            .crates
            .into_iter()
            .map(|(name, raw_data)| {
                let length = raw_data.t.len();
                let (items, _) = raw_data
                    .t
                    .into_iter()
                    .zip(raw_data.n.into_iter())
                    .zip(raw_data.q.into_iter())
                    .zip(raw_data.d.into_iter())
                    .zip(raw_data.i.into_iter())
                    .fold(
                        (Vec::with_capacity(length), String::new()),
                        |(mut items, path), ((((t, n), q), d), i)| {
                            let path = if q.is_empty() { path } else { q };
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

fn generate_mapping(data: IndexData) -> HashMap<String, HashMap<String, String>> {
    data.crates
        .into_iter()
        .map(|(name, data)| (name, generate_crate_mapping(data)))
        .collect()
}

fn generate_crate_mapping(data: CrateData) -> HashMap<String, String> {
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
