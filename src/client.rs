use clap::{command, Parser};
use futures_util::{SinkExt, StreamExt};
use http::Uri;
use std::io;
use std::io::Write;
use std::str::FromStr;
use tokio;
use tokio_websockets::{ClientBuilder, Error, Message};

use rustdb::command::{parse_readable_command, Command};
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

        match parse_readable_command(input.as_str()) {
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
}
