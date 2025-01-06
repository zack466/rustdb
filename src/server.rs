use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio;
use tokio::net::TcpListener;
use tokio_websockets::{Error, Message, ServerBuilder};

use rustdb::command::Command;
use rustdb::resp::RESP;
use rustdb::table::Table;
use rustdb::value::Value;

fn dispatch(command: Command, shared: Arc<Mutex<Table>>) -> Result<Value, String> {
    let mut table = shared.lock().unwrap();

    match command {
        Command::Get(key) => Ok(table.get(&key).unwrap_or(Value::Null)),
        Command::Set(key, value) => {
            table.set(key, value);
            Ok(Value::SimpleString("OK".to_string()))
        }
        Command::Inc(key) => match table.get(&key) {
            Some(Value::Int(i)) => {
                table.set(key, Value::Int(i + 1));
                Ok(Value::SimpleString("OK".to_string()))
            }
            _ => Err("Tried to increment a non-integer".to_string()),
        },
        Command::Dec(key) => match table.get(&key) {
            Some(Value::Int(i)) => {
                table.set(key, Value::Int(i - 1));
                Ok(Value::SimpleString("OK".to_string()))
            }
            _ => Err("Tried to decrement a non-integer".to_string()),
        },
        Command::Hello => {
            println!("Received HELLO");
            Ok(Value::String("WORLD".to_string()))
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    // TODO: load table from disk if passed in as argument
    let table = Table::new();
    let shared = Arc::new(Mutex::new(table));

    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    while let Ok((stream, _)) = listener.accept().await {
        let (_request, mut ws_stream) = ServerBuilder::new().accept(stream).await?;
        let shared = shared.clone();

        tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_stream.next().await {
                if msg.is_text() || msg.is_binary() {
                    let command = Command::decode_resp(msg.as_text().unwrap().to_string()).unwrap();
                    let result = dispatch(command, shared.clone());
                    match result {
                        Ok(value) => ws_stream.send(Message::text(value.encode_resp())).await?,
                        Err(e) => ws_stream.send(Message::text(e.to_string())).await?,
                    }
                }
            }

            Ok::<_, Error>(())
        });
    }

    Ok(())
}
