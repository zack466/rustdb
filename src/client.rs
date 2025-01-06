use clap::{command, Parser};
use futures_util::{SinkExt, StreamExt};
use http::Uri;
use std::io;
use std::io::Write;
use std::str::FromStr;
use tokio;
use tokio_websockets::{ClientBuilder, Error, Message};

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_till1};
use nom::character::complete::{anychar, i64, multispace0};
use nom::combinator::value;
use nom::multi::{many0, many_till, separated_list0};
use nom::sequence::{delimited, tuple};
use nom::IResult;


use rustdb::command::Command;
use rustdb::resp::RESP;
use rustdb::value::Value;

#[derive(Parser)]
#[command(name = "rustdb-client")]
#[command(version = "0.1.0")]
#[command(about = "A simple client for rustdb", long_about = None)]
#[command(arg_required_else_help(true))]
struct Cli {
    #[arg(long, default_value = "ws://127.0.0.1:3000")]
    uri: String,
}

// Logic for parsing commands.

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
        value("EXIT".to_string(), tag_no_case("EXIT")),
        value("HELP".to_string(), tag_no_case("HELP")),
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
            "EXIT" => {
                if args.is_empty() {
                    Ok(Command::Exit)
                } else {
                    Err("Invalid usage: EXIT".to_string())
                }
            }
            "HELP" => {
                if args.is_empty() {
                    Ok(Command::Help)
                } else {
                    Err("Invalid usage: HELP".to_string())
                }
            }
            _ => Err(format!("Unknown command: {}", cmd)),
        }
    } else {
        Err("Invalid command".to_string())
    }
}


#[tokio::main]
pub async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let uri = Uri::from_str(cli.uri.as_str()).unwrap();
    let (mut client, _) = ClientBuilder::from_uri(uri).connect().await?;

    loop {
        let mut input = String::new();
        print!("> ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        if input.is_empty() {
            println!("Exiting...");
            break;
        }

        match parse_readable_command(input.as_str()) {
            Ok(Command::Help) => {
                println!("Available commands:");
                println!("  GET <key>");
                println!("  SET <key> <value>");
                println!("  INC <key>");
                println!("  DEC <key>");
                println!("  SAVE");
                println!("  EXIT");
                println!("  HELP");
            }
            Ok(Command::Exit) => {
                println!("Exiting...");
                break;
            }
            Ok(command) => {
                client
                    .send(Message::text(Command::encode_resp(command)))
                    .await?;

                while let Some(Ok(msg)) = client.next().await {
                    if let Some(text) = msg.as_text() {
                        let response = Value::decode_resp(text.to_string()).unwrap();
                        println!("{}", response.string_repr());
                        break;
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        }
    }

    client.close().await?;
    Ok(())
}
