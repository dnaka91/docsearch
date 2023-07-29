use std::collections::HashMap;

use serde::Deserialize;
use serde_tuple::Deserialize_tuple;

use super::{ItemType, RawCrateData, RawIndexData};
use crate::error::{Error, Result};

#[derive(Deserialize)]
struct RawIndex {
    #[serde(flatten)]
    crates: HashMap<String, RawCrate>,
}

#[derive(Deserialize)]
pub(super) struct RawCrate {
    doc: String,
    i: Vec<Entry>,
    p: Vec<(ItemType, String)>,
}

impl From<RawCrate> for RawCrateData {
    fn from(mut raw: RawCrate) -> Self {
        RawCrateData {
            doc: raw.doc,
            t: raw.i.iter().map(|entry| entry.t).collect(),
            n: raw
                .i
                .iter_mut()
                .map(|entry| entry.n.take().unwrap_or_default())
                .collect(),
            q: raw
                .i
                .iter_mut()
                .enumerate()
                .filter_map(|(i, entry)| {
                    let q = entry.q.take().unwrap_or_default();
                    (!q.is_empty()).then_some((i, q))
                })
                .collect(),
            d: raw
                .i
                .iter_mut()
                .map(|entry| entry.d.take().unwrap_or_default())
                .collect(),
            i: raw
                .i
                .iter_mut()
                .map(|entry| entry.i.unwrap_or_default())
                .collect(),
            p: raw.p,
        }
    }
}

#[derive(Deserialize_tuple)]
struct Entry {
    t: ItemType,
    n: Option<String>,
    q: Option<String>,
    d: Option<String>,
    i: Option<usize>,
    #[allow(dead_code)]
    f: Option<Vec<serde_json::Value>>,
}

pub(super) fn load_raw(index: &str) -> Result<RawIndexData> {
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

    let raw = serde_json::from_str::<RawIndex>(&json).map_err(Error::from)?;

    Ok(RawIndexData {
        crates: raw
            .crates
            .into_iter()
            .map(|(name, raw)| (name, raw.into()))
            .collect(),
    })
}
