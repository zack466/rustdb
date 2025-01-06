use crate::value::Value;
use crate::resp::RESP;

use nom::branch::alt;
use nom::IResult;
use crate::resp::parse_bulk;
use crate::value::parse_value;

#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    Success(Value),
    Error(String),
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