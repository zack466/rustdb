use nom::bytes::complete::{tag, take, take_until};
use nom::character::complete::u64;
use nom::sequence::tuple;
use nom::IResult;

pub trait RESP {
    fn encode_resp(self) -> String;
    fn decode_resp(s: String) -> Result<Self, String>
    where
        Self: Sized;
}

pub fn parse_bulk<'a>(input: &'a str, prefix: &'a str) -> IResult<&'a str, String> {
    let (remaining, (_, len)) = tuple((tag(prefix), u64))(input)?;

    let (remaining, (s, _)) = tuple((take(len), tag("\r\n")))(remaining)?;
    let s = s.to_string();

    Ok((remaining, s))
}

pub fn parse_simple<'a>(input: &'a str, prefix: &'a str) -> IResult<&'a str, String> {
    let (remaining, (_, s, _)) = tuple((tag(prefix), take_until("\r\n"), tag("\r\n")))(input)?;
    Ok((remaining, s.to_string()))
}
