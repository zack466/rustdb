use crate::command::Command;
use crate::response::Response;
use crate::value::Value;

use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::character::complete::{i64, u64};
use nom::combinator::value;
use nom::multi::many_m_n;
use nom::sequence::tuple;
use nom::IResult;

pub trait RESP {
    fn encode_resp(self) -> String;
    fn decode_resp(s: String) -> Result<Self, String>
    where
        Self: Sized;
}

fn parse_null(input: &str) -> IResult<&str, Value> {
    value(Value::Null, tag("$-1\r\n"))(input)
}

fn parse_ok(input: &str) -> IResult<&str, Value> {
    value(Value::Ok, tag("+OK\r\n"))(input)
}

fn parse_bulk<'a>(input: &'a str, prefix: &'a str) -> IResult<&'a str, String> {
    let (remaining, (_, len)) = tuple((tag(prefix), u64))(input)?;

    let (remaining, (s, _)) = tuple((take(len), tag("\r\n")))(remaining)?;
    let s = s.to_string();

    Ok((remaining, s))
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

fn parse_value(input: &str) -> IResult<&str, Value> {
    alt((parse_ok, parse_null, parse_nonnull_string, parse_int, parse_array))(input)
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
            Self::Ok => "+OK\r\n".to_string(),
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

impl RESP for Command {
    fn encode_resp(self) -> String {
        match self {
            Self::Get(key) => Value::encode_resp(Value::Array(vec![
                Value::String("GET".to_string()),
                Value::String(key),
            ])),
            Self::Set(key, value) => Value::encode_resp(Value::Array(vec![
                Value::String("SET".to_string()),
                Value::String(key),
                value,
            ])),
            Self::Inc(key) => Value::encode_resp(Value::Array(vec![
                Value::String("INC".to_string()),
                Value::String(key),
            ])),
            Self::Dec(key) => Value::encode_resp(Value::Array(vec![
                Value::String("DEC".to_string()),
                Value::String(key),
            ])),
            Self::Hello => Value::encode_resp(Value::Array(vec![
                Value::String("HELLO".to_string()),
            ])),
        }
    }

    fn decode_resp(s: String) -> Result<Self, String> {
        let value = Value::decode_resp(s)?;
        let Value::Array(a) = value else {
            return Err("expected array of strings".to_string());
        };

        let Some(Value::String(cmd)) = a.first() else {
            return Err("expected array of strings".to_string());
        };

        match cmd.as_str() {
            "GET" => {
                let key = a.get(1).unwrap();
                Ok(Self::Get(key.to_string()))
            }
            "SET" => {
                let key = a.get(1).unwrap();
                let value = a.get(2).unwrap();
                Ok(Self::Set(key.to_string(), value.clone()))
            }
            "INC" => {
                let key = a.get(1).unwrap();
                Ok(Self::Inc(key.to_string()))
            }
            "DEC" => {
                let key = a.get(1).unwrap();
                Ok(Self::Dec(key.to_string()))
            }
            "HELLO" => Ok(Self::Hello),
            _ => Err("unknown command".to_string()),
        }
    }
}

fn parse_error(input: &str) -> IResult<&str, Response> {
    let (remaining, msg) = parse_bulk(input, "!")?;
    Ok((remaining, Response::Error(msg)))
}

fn parse_success(input: &str) -> IResult<&str, Response> {
    let (remaining, value) = parse_value(input)?;
    Ok((remaining, Response::Success(value)))
}

fn parse_response(input: &str) -> IResult<&str, Response> {
    alt((parse_error, parse_success))(input)
}

impl RESP for Response {
    fn encode_resp(self) -> String {
        match self {
            Self::Success(value) => value.encode_resp(),
            Self::Error(e) => format!("!{}{}", e.len(), e),
        }
    }

    fn decode_resp(s: String) -> Result<Self, String> {
        match parse_response(s.as_str()) {
            Ok((remaining, response)) => {
                if !remaining.is_empty() {
                    return Err("expected end of string".to_string());
                }
                Ok(response)
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
