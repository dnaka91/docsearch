use std::collections::HashMap;

use chumsky::{error::Cheap, prelude::*};

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
            let json = match parser(&r).parse(index) {
                Ok(json) => json,
                Err(errs) => {
                    return Err(Error::InvalidIndexJavaScript(errs.first().map_or_else(
                        || "Unknown error".to_owned(),
                        |err| {
                            let span = String::from_utf8_lossy(&index.as_bytes()[err.span()]);
                            match err.label() {
                                Some(label) => format!("{label}: {span}"),
                                None => span.into_owned(),
                            }
                        },
                    )));
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
    Invalid,
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
            JsJson::Invalid => return Err("Invalid JSON element".to_owned()),
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

fn parser(references: &[String]) -> impl Parser<char, JsJson, Error = Cheap<char>> + '_ {
    recursive(|value| {
        let number = text::int(10).from_str().unwrapped().labelled("number");

        let escape = just('\\').ignore_then(
            just('\\')
                .or(just('/'))
                .or(just('"'))
                .or(just('b').to('\x08'))
                .or(just('f').to('\x0C'))
                .or(just('n').to('\n'))
                .or(just('r').to('\r'))
                .or(just('t').to('\t'))
                .or(just('u').ignore_then(
                    filter(|c: &char| c.is_digit(16))
                        .repeated()
                        .exactly(4)
                        .collect::<String>()
                        .validate(|digits, _span, _emit| {
                            char::from_u32(u32::from_str_radix(&digits, 16).unwrap()).unwrap_or(
                                '\u{FFFD}', // unicode replacement character
                            )
                        }),
                )),
        );

        let string = just('"')
            .ignore_then(filter(|c| *c != '\\' && *c != '"').or(escape).repeated())
            .then_ignore(just('"'))
            .collect::<String>()
            .labelled("string");

        let array = value
            .clone()
            .chain(just(',').ignore_then(value.clone()).repeated())
            .or_not()
            .flatten()
            .delimited_by(just('['), just(']'))
            .map(JsJson::Array)
            .labelled("array");

        let member = string.then_ignore(just(':').padded()).then(value);
        let object = member
            .clone()
            .chain(just(',').padded().ignore_then(member).repeated())
            .or_not()
            .flatten()
            .padded()
            .delimited_by(just('{'), just('}'))
            .collect::<HashMap<String, JsJson>>()
            .map(JsJson::Object)
            .labelled("object");

        let reference = just("R")
            .ignore_then(number.delimited_by(just('['), just(']')).map(
                |v| match references.get(v) {
                    Some(s) => JsJson::Str(String::clone(s)),
                    None => JsJson::Invalid,
                },
            ))
            .labelled("R");

        just("null")
            .to(JsJson::Null)
            .labelled("null")
            .or(number.map(JsJson::Num))
            .or(string.map(JsJson::Str))
            .or(array)
            .or(object)
            .or(just("N").to(JsJson::Null).labelled("N"))
            .or(just("E").to(JsJson::Str(String::new())).labelled("E"))
            .or(just("T").to(JsJson::Str("t".to_owned())).labelled("T"))
            .or(just("U").to(JsJson::Str("u".to_owned())).labelled("U"))
            .or(reference)
            .recover_with(nested_delimiters('{', '}', [('[', ']')], |_| {
                JsJson::Invalid
            }))
            .recover_with(nested_delimiters('[', ']', [('{', '}')], |_| {
                JsJson::Invalid
            }))
            .recover_with(skip_then_retry_until(['}', ']']))
            .padded()
    })
    .then_ignore(end().recover_with(skip_then_retry_until([])))
}
