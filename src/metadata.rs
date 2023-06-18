use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::take_while,
    character::complete::{char, satisfy, space1},
    combinator::{iterator, recognize},
    sequence::preceded,
    Parser,
};

use crate::{amount, empty_line, end_of_line, string, Currency, Decimal, IResult, Span};

/// Metadata value
///
/// # Example
///
/// ```
/// # use beancount_parser_2::MetadataValue;
/// let input = r#"
/// 2023-05-27 commodity CHF
///     title: "Swiss Franc"
/// "#;
/// let beancount = beancount_parser_2::parse::<f64>(input).unwrap();
/// let directive_metadata = &beancount.directives[0].metadata;
/// assert_eq!(directive_metadata.get("title"), Some(&MetadataValue::String("Swiss Franc".into())));
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Value<D> {
    /// String value
    String(String),
    /// A number or number expression
    Number(D),
    /// A [`Currency`]
    Currency(Currency),
}

pub(crate) fn parse<D: Decimal>(input: Span<'_>) -> IResult<'_, HashMap<&str, Value<D>>> {
    let mut iter = iterator(input, alt((entry.map(Some), empty_line.map(|_| None))));
    let map: HashMap<_, _> = iter.flatten().collect();
    let (input, _) = iter.finish()?;
    Ok((input, map))
}

fn entry<D: Decimal>(input: Span<'_>) -> IResult<'_, (&str, Value<D>)> {
    let (input, _) = space1(input)?;
    let (input, key) = recognize(preceded(
        satisfy(char::is_lowercase),
        take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_'),
    ))(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = space1(input)?;
    let (input, value) = alt((
        string.map(ToOwned::to_owned).map(Value::String),
        amount::expression.map(Value::Number),
        amount::currency.map(Value::Currency),
    ))(input)?;
    let (input, _) = end_of_line(input)?;
    Ok((input, (*key.fragment(), value)))
}
