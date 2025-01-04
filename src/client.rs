use futures_util::{SinkExt, StreamExt};
use http::Uri;
use std::sync::MutexGuard;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::net::TcpListener;
use tokio_websockets::{ClientBuilder, Error, Message, ServerBuilder};

use crate::command::Command;
use crate::resp::RESP;
use crate::response::Response;
use crate::table::Table;
use crate::value::Value;

mod command;
mod resp;
mod response;
mod table;
mod value;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    let uri = Uri::from_static("ws://127.0.0.1:3000");
    let (mut client, _) = ClientBuilder::from_uri(uri).connect().await?;

    let command = Command::Hello;

    client.send(Message::text(command.encode_resp())).await?;

    while let Some(Ok(msg)) = client.next().await {
        if let Some(text) = msg.as_text() {
            let response = Response::decode_resp(text.to_string()).unwrap();
            println!("Response: {:?}", response);
            client.close().await?;
        }
    }

    Ok(())
}
