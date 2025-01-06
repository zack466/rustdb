use futures_util::{SinkExt, StreamExt};
use http::Uri;
use tokio;
use tokio_websockets::{ClientBuilder, Error, Message};

use rustdb::command::Command;
use rustdb::resp::RESP;
use rustdb::response::Response;

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
