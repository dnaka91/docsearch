#![allow(clippy::cast_possible_truncation)]

use std::collections::HashMap;

use winnow::{
    ascii::dec_uint,
    combinator::{
        cut_err, delimited, fail, fold_repeat, peek, preceded, separated0, separated_pair, success,
        terminated,
    },
    dispatch,
    error::StrContext,
    stream::AsChar,
    token::{any, none_of, take_while},
    PResult, Parser, Stateful,
};

use super::{v2::RawCrate, RawIndexData};
use crate::error::IndexV1Error as Error;

pub(super) fn load_raw(index: &str) -> Result<RawIndexData, Error> {
    let r = {
        let r = index
            .lines()
            .find_map(|line| line.strip_prefix("var R="))
            .and_then(|line| line.strip_suffix(';'))
            .ok_or(Error::MissingReference)?;
        serde_json::from_str::<Vec<String>>(r).map_err(Error::InvalidReferenceJson)?
    };

    let entries = index.lines().filter_map(|line| {
        line.strip_prefix("searchIndex[\"")
            .and_then(|line| line.strip_suffix(';'))
            .and_then(|line| line.split_once("\"]="))
    });

    let crates = entries
        .map(|(name, index)| {
            let json = match json.parse(Stateful {
                input: index,
                state: r.as_slice(),
            }) {
                Ok(json) => json,
                Err(err) => {
                    return Err(Error::InvalidIndexJavaScript(format!("Parse error: {err}")));
                }
            };
            let json = serde_json::Value::try_from(json).map_err(Error::InvalidIndexJavaScript)?;
            let data = serde_json::from_value::<RawCrate>(json)
                .map_err(Error::InvalidIndexJson)?
                .into();

            Ok((name.to_owned(), data))
        })
        .collect::<Result<_, _>>()?;

    Ok(RawIndexData { crates })
}

#[derive(Clone, Debug)]
enum JsJson {
    Null,
    Str(String),
    Num(usize),
    Array(Vec<JsJson>),
    Object(HashMap<String, JsJson>),
}

impl TryFrom<JsJson> for serde_json::Value {
    type Error = String;

    fn try_from(value: JsJson) -> Result<Self, Self::Error> {
        Ok(match value {
            JsJson::Null => serde_json::Value::Null,
            JsJson::Str(string) => serde_json::Value::String(string),
            JsJson::Num(num) => serde_json::Value::Number(num.into()),
            JsJson::Array(array) => serde_json::Value::Array(
                array
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, String>>()?,
            ),
            JsJson::Object(object) => serde_json::Value::Object(
                object
                    .into_iter()
                    .map(|(k, v)| Ok((k, v.try_into()?)))
                    .collect::<Result<_, String>>()?,
            ),
        })
    }
}

type Stream<'i> = Stateful<&'i str, &'i [String]>;

fn json(input: &mut Stream<'_>) -> PResult<JsJson> {
    delimited(ws, json_value, ws).parse_next(input)
}

fn json_value(input: &mut Stream<'_>) -> PResult<JsJson> {
    dispatch!(
        peek(any);
        'n' => null.value(JsJson::Null),
        '0'..='9' => number.map(JsJson::Num),
        '"' => string.map(JsJson::Str),
        '[' => array.map(JsJson::Array),
        '{' => object.map(JsJson::Object),
        'N' => "N".value(JsJson::Null),
        'E' => "E".map(|_| JsJson::Str(String::new())),
        'T' => "T".map(|_| JsJson::Str("t".to_owned())),
        'U' => "U".map(|_| JsJson::Str("u".to_owned())),
        'R' => reference,
        _ => fail,
    )
    .context(StrContext::Label("value"))
    .parse_next(input)
}

fn null<'i>(input: &mut Stream<'i>) -> PResult<&'i str> {
    "null".context(StrContext::Label("null")).parse_next(input)
}

fn number(input: &mut Stream<'_>) -> PResult<usize> {
    dec_uint
        .map(|v: u64| v as usize)
        .context(StrContext::Label("number"))
        .parse_next(input)
}

fn string(input: &mut Stream<'_>) -> PResult<String> {
    preceded(
        '\"',
        cut_err(terminated(
            fold_repeat(0.., character, String::new, |mut string, c| {
                string.push(c);
                string
            }),
            '\"',
        )),
    )
    .context(StrContext::Label("string"))
    .parse_next(input)
}

fn character(input: &mut Stream<'_>) -> PResult<char> {
    let c = none_of('\"').parse_next(input)?;
    if c == '\\' {
        dispatch!(
            any;
            '"' => success('"'),
            '\\' => success('\\'),
            '/'  => success('/'),
            'b' => success('\x08'),
            'f' => success('\x0C'),
            'n' => success('\n'),
            'r' => success('\r'),
            't' => success('\t'),
            'u' => unicode_escape,
            _ => fail,
        )
        .parse_next(input)
    } else {
        Ok(c)
    }
}

fn unicode_escape(input: &mut Stream<'_>) -> PResult<char> {
    take_while(4, AsChar::is_hex_digit)
        .map(|hex| char::from_u32(u32::from_str_radix(hex, 16).unwrap()).unwrap_or('\u{FFFD}'))
        .parse_next(input)
}

fn array(input: &mut Stream<'_>) -> PResult<Vec<JsJson>> {
    preceded(
        ('[', ws),
        cut_err(terminated(separated0(json_value, (ws, ',', ws)), (ws, ']'))),
    )
    .context(StrContext::Label("array"))
    .parse_next(input)
}

fn object(input: &mut Stream<'_>) -> PResult<HashMap<String, JsJson>> {
    preceded(
        ('{', ws),
        cut_err(terminated(separated0(key_value, (ws, ',', ws)), (ws, '}'))),
    )
    .context(StrContext::Label("object"))
    .parse_next(input)
}

fn key_value(input: &mut Stream<'_>) -> PResult<(String, JsJson)> {
    separated_pair(string, cut_err((ws, ':', ws)), json_value).parse_next(input)
}

fn reference(input: &mut Stream<'_>) -> PResult<JsJson> {
    preceded(
        "R[",
        cut_err(terminated(
            dec_uint.try_map(|v: u64| {
                input
                    .state
                    .get(v as usize)
                    .map(|r| JsJson::Str(r.clone()))
                    .ok_or(Error::MissingReference)
            }),
            ']',
        )),
    )
    .context(StrContext::Label("reference"))
    .parse_next(input)
}

fn ws<'a>(input: &mut Stream<'a>) -> PResult<&'a str> {
    take_while(0.., &[' ', '\t', '\r', '\n']).parse_next(input)
}
