use serde::{Deserialize, Serialize};

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{i64, u64};
use nom::combinator::value;
use nom::multi::many_m_n;
use nom::sequence::tuple;
use nom::IResult;

use crate::resp::RESP;
use crate::resp::{parse_bulk, parse_simple};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Int(i64),
    Array(Vec<Value>),
    Null,
    SimpleString(String),
    SimpleError(String),
}

impl Value {
    pub fn string_repr(&self) -> String {
        match self {
            Value::String(s) => format!("\"{}\"", s),
            Value::Int(i) => format!("(integer) {}", i),
            Value::Array(a) => format!(
                "[{}]",
                a.iter()
                    .map(|v| v.string_repr())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Value::Null => "(nil)".to_string(),
            Value::SimpleString(s) => format!("(string) {}", s),
            Value::SimpleError(s) => format!("(error) {}", s),
        }
    }
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::SimpleString(s) => s.clone(),
            Value::String(s) => s.clone(),
            _ => panic!("Value is not a string"),
        }
    }
}

fn parse_null(input: &str) -> IResult<&str, Value> {
    value(Value::Null, tag("$-1\r\n"))(input)
}

fn parse_simple_string(input: &str) -> IResult<&str, Value> {
    let (remaining, s) = parse_simple(input, "+")?;
    Ok((remaining, Value::SimpleString(s)))
}

fn parse_simple_error(input: &str) -> IResult<&str, Value> {
    let (remaining, s) = parse_simple(input, "-")?;
    Ok((remaining, Value::SimpleError(s)))
}

fn parse_nonnull_string(input: &str) -> IResult<&str, Value> {
    let (remaining, s) = parse_bulk(input, "$")?;
    Ok((remaining, Value::String(s)))
}

fn parse_int(input: &str) -> IResult<&str, Value> {
    let (remaining, (_, i, _)) = tuple((tag(":"), i64, tag("\r\n")))(input)?;

    Ok((remaining, Value::Int(i)))
}

fn parse_array(input: &str) -> IResult<&str, Value> {
    let (remaining, (_, len)) = tuple((tag("*"), u64))(input)?;

    let (remaining, (values, _)) = tuple((
        many_m_n(len as usize, len as usize, parse_value),
        tag("\r\n"),
    ))(remaining)?;

    Ok((remaining, Value::Array(values)))
}

pub fn parse_value(input: &str) -> IResult<&str, Value> {
    alt((
        parse_simple_string,
        parse_simple_error,
        parse_null,
        parse_nonnull_string,
        parse_int,
        parse_array,
    ))(input)
}

impl RESP for Value {
    fn encode_resp(self) -> String {
        match self {
            Self::String(s) => {
                format!("${}{}\r\n", s.len(), s)
            }
            Self::Int(i) => {
                format!(":{}\r\n", i)
            }
            Self::Array(a) => {
                let contents = a
                    .iter()
                    .map(|v| Self::encode_resp(v.clone()))
                    .collect::<Vec<String>>()
                    .join("");
                format!("*{}{}\r\n", a.len(), contents)
            }
            Self::Null => "$-1\r\n".to_string(),
            Self::SimpleString(s) => format!("+{}\r\n", s),
            Self::SimpleError(s) => format!("-{}\r\n", s),
        }
    }

    fn decode_resp(s: String) -> Result<Self, String> {
        match parse_value(s.as_str()) {
            Ok((remaining, value)) => {
                if !remaining.is_empty() {
                    return Err("expected end of string".to_string());
                }
                Ok(value)
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
