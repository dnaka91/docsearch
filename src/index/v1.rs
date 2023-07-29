use std::collections::HashMap;

use chumsky::prelude::*;

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
            let json = match parser(&r).parse(index).into_result() {
                Ok(json) => json,
                Err(errs) => {
                    return Err(Error::InvalidIndexJavaScript(errs.first().map_or_else(
                        || "Unknown error".to_owned(),
                        |err| format!("Parse error: {err}"),
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

#[allow(clippy::too_many_lines)]
fn parser(references: &[String]) -> impl Parser<'_, &str, JsJson> {
    recursive(|value| {
        let number = text::int(10).from_str().unwrapped().boxed();

        let escape = just('\\')
            .then(choice((
                just('\\'),
                just('/'),
                just('"'),
                just('b').to('\x08'),
                just('f').to('\x0C'),
                just('n').to('\n'),
                just('r').to('\r'),
                just('t').to('\t'),
                just('u').ignore_then(text::digits(16).exactly(4).slice().validate(
                    |digits, _span, _emit| {
                        char::from_u32(u32::from_str_radix(digits, 16).unwrap()).unwrap_or(
                            '\u{FFFD}', // unicode replacement character
                        )
                    },
                )),
            )))
            .ignored()
            .boxed();

        let string = none_of("\\\"")
            .ignored()
            .or(escape)
            .repeated()
            .slice()
            .map(ToString::to_string)
            .delimited_by(just('"'), just('"'))
            .boxed();

        let array = value
            .clone()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect()
            .padded()
            .delimited_by(
                just('['),
                just(']').ignored().recover_with(via_parser(end())),
            )
            .boxed();

        let member = string.clone().then_ignore(just(':').padded()).then(value);
        let object = member
            .clone()
            .separated_by(just(',').padded())
            .collect()
            .padded()
            .delimited_by(
                just('{'),
                just('}').ignored().recover_with(via_parser(end())),
            )
            .boxed();

        let reference =
            just("R").ignore_then(number.clone().delimited_by(just('['), just(']')).map(|v| {
                match references.get(v) {
                    Some(s) => JsJson::Str(String::clone(s)),
                    None => JsJson::Invalid,
                }
            }));

        choice((
            just("null").to(JsJson::Null),
            number.map(JsJson::Num),
            string.map(JsJson::Str),
            array.map(JsJson::Array),
            object.map(JsJson::Object),
            just("N").to(JsJson::Null),
            just("E").to(JsJson::Str(String::new())),
            just("T").to(JsJson::Str("t".to_owned())),
            just("U").to(JsJson::Str("u".to_owned())),
            reference,
        ))
        .recover_with(via_parser(nested_delimiters(
            '{',
            '}',
            [('[', ']')],
            |_| JsJson::Invalid,
        )))
        .recover_with(via_parser(nested_delimiters(
            '[',
            ']',
            [('{', '}')],
            |_| JsJson::Invalid,
        )))
        .recover_with(skip_then_retry_until(
            any().ignored(),
            one_of(",]}").ignored()
        ))
        .padded()
    })
}
