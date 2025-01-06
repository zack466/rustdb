use crate::resp::RESP;
use crate::value::Value;

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_till1};
use nom::character::complete::{anychar, i64, multispace0, multispace1, u64};
use nom::combinator::value;
use nom::multi::{many0, many1, many_m_n, many_till, separated_list0};
use nom::sequence::{delimited, tuple};
use nom::IResult;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Get(String),
    Set(String, Value),
    Inc(String),
    Dec(String),
    Hello,
    Save,
}

// TODO: Yes, this is super boilerplate-y. There's probably some way to generate all of these functions
// from the enum definition using a macro, but I haven't written it yet.

fn parse_readable_int(input: &str) -> IResult<&str, Value> {
    let (remaining, i) = i64(input)?;
    Ok((remaining, Value::Int(i)))
}

fn parse_readable_array(input: &str) -> IResult<&str, Value> {
    let (remaining, a) = alt((
        // empty array literal to prevent being parsed as the string "[]"
        value(vec![], tag("[]")),
        delimited(
            tag("["),
            separated_list0(tag(","), parse_readable_value),
            tag("]"),
        ),
    ))(input)?;
    Ok((remaining, Value::Array(a)))
}

fn parse_literal_string(input: &str) -> IResult<&str, Value> {
    let (remaining, (_, (s, _))) = tuple((tag("\""), many_till(anychar, tag("\""))))(input)?;
    Ok((remaining, Value::String(s.iter().collect())))
}

fn parse_unquoted_string(input: &str) -> IResult<&str, Value> {
    let (remaining, s) = take_till1(|c| c == ' ')(input)?;
    Ok((remaining, Value::String(s.to_string())))
}

fn parse_readable_string(input: &str) -> IResult<&str, Value> {
    alt((parse_literal_string, parse_unquoted_string))(input)
}

fn parse_readable_value(input: &str) -> IResult<&str, Value> {
    delimited(
        multispace0,
        alt((
            parse_readable_int,
            parse_readable_array,
            parse_readable_string,
        )),
        multispace0,
    )(input)
}

fn parse_command_name(input: &str) -> IResult<&str, String> {
    alt((
        value("GET".to_string(), tag_no_case("GET")),
        value("SET".to_string(), tag_no_case("SET")),
        value("INC".to_string(), tag_no_case("INC")),
        value("DEC".to_string(), tag_no_case("DEC")),
        value("SAVE".to_string(), tag_no_case("SAVE")),
        value("HELLO".to_string(), tag_no_case("HELLO")),
    ))(input)
}

pub fn parse_readable_command(input: &str) -> Result<Command, String> {
    if let Ok((remaining, (cmd, args))) = tuple((
        delimited(multispace0, parse_command_name, multispace0),
        many0(parse_readable_value),
    ))(input)
    {
        assert!(remaining.is_empty());
        match cmd.as_str() {
            "GET" => {
                if let [Value::String(key)] = args.as_slice() {
                    Ok(Command::Get(key.clone()))
                } else {
                    Err("Invalid usage: GET <key>".to_string())
                }
            }
            "SET" => {
                if let [Value::String(key), value] = args.as_slice() {
                    Ok(Command::Set(key.clone(), value.clone()))
                } else {
                    Err("Invalid usage: SET <key> <value>".to_string())
                }
            }
            "INC" => {
                if let [Value::String(key)] = args.as_slice() {
                    Ok(Command::Inc(key.clone()))
                } else {
                    Err("Invalid usage: INC <key>".to_string())
                }
            }
            "DEC" => {
                if let [Value::String(key)] = args.as_slice() {
                    Ok(Command::Dec(key.clone()))
                } else {
                    Err("Invalid usage: DEC <key>".to_string())
                }
            }
            "SAVE" => {
                if args.is_empty() {
                    Ok(Command::Save)
                } else {
                    Err("Invalid usage: SAVE".to_string())
                }
            }
            "HELLO" => {
                if args.is_empty() {
                    Ok(Command::Hello)
                } else {
                    Err("Invalid usage: HELLO".to_string())
                }
            }
            _ => Err(format!("Unknown command: {}", cmd)),
        }
    } else {
        Err("Invalid command".to_string())
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
            Self::Hello => {
                Value::encode_resp(Value::Array(vec![Value::String("HELLO".to_string())]))
            }
            Self::Save => Value::encode_resp(Value::Array(vec![Value::String("SAVE".to_string())])),
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
            "SAVE" => Ok(Self::Save),
            _ => Err("unknown command".to_string()),
        }
    }
}
