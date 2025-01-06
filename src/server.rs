use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio;
use tokio::net::TcpListener;
use tokio_websockets::{Error, Message, ServerBuilder};
use clap::{command, Parser};
use std::path::Path;

use rustdb::command::Command;
use rustdb::resp::RESP;
use rustdb::table::Table;
use rustdb::value::Value;

#[derive(Parser)]
#[command(name = "rustdb")]
#[command(version = "0.1.0")]
#[command(about = "A simple key-value store", long_about = None)]
struct Cli {
    #[arg(long, default_value_t = String::new())]
    path: String,
    #[arg(long, default_value_t = 3000)]
    port: u16,
    #[arg(long, default_value_t = true)]
    autosave: bool,
}

struct Db {
    table: Table,
    path: Option<String>,
}

fn dispatch(command: Command, shared: Arc<Mutex<Db>>) -> Result<Value, String> {
    let mut db = shared.lock().unwrap();

    match command {
        Command::Get(key) => Ok(db.table.get(&key).unwrap_or(Value::Null)),
        Command::Set(key, value) => {
            db.table.set(key, value);
            Ok(Value::SimpleString("OK".to_string()))
        }
        Command::Inc(key) => match db.table.get(&key) {
            Some(Value::Int(i)) => {
                db.table.set(key, Value::Int(i + 1));
                Ok(Value::SimpleString("OK".to_string()))
            }
            _ => Ok(Value::SimpleError("cannot increment non-integer".to_string())),
        },
        Command::Dec(key) => match db.table.get(&key) {
            Some(Value::Int(i)) => {
                db.table.set(key, Value::Int(i - 1));
                Ok(Value::SimpleString("OK".to_string()))
            }
            _ => Ok(Value::SimpleError("cannot decrement non-integer".to_string())),
        },
        Command::Hello => {
            Ok(Value::SimpleString("WORLD".to_string()))
        }
        Command::Save => {
            if let Some(path) = &db.path {
                if !path.is_empty() {
                    db.table.to_disk(path.as_str()).unwrap();
                }
            }
            Ok(Value::SimpleString("OK".to_string()))
        }
        _ => Ok(Value::SimpleString("OK".to_string()))
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let table = if cli.path.is_empty() {
        println!("No database file provided, running in-memory mode");
        Table::new()
    } else if Path::new(&cli.path).exists() {
        println!("Loading database from {}", cli.path);
        Table::from_disk(&cli.path).unwrap()
    } else {
        println!("Database file not found, creating new database");
        Table::new()
    };

    if !cli.path.is_empty() {
        if cli.autosave {
            println!("Note: autosave is enabled, database will be saved to disk every hour.");
        } else {
            println!("Note: autosave is disabled, database must be saved manually using the SAVE command.");
        }
    } else if cli.autosave {
        println!("Note: autosave is enabled, but no database file was provided. Data will be lost on exit.");
    }

    let db = Db {
        table,
        path: Some(cli.path),
    };

    let shared = Arc::new(Mutex::new(db));

    let listener = TcpListener::bind(format!("127.0.0.1:{}", cli.port)).await?;

    while let Ok((stream, _)) = listener.accept().await {
        let (_request, mut ws_stream) = ServerBuilder::new().accept(stream).await?;
        let shared = shared.clone();

        println!("Accepting connection from {}", ws_stream.get_ref().peer_addr().unwrap());

        tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_stream.next().await {
                if msg.is_text() || msg.is_binary() {
                    let msg = msg.as_text().unwrap().to_string();
                    println!("Received message: {:?}", msg);
                    let command = Command::decode_resp(msg).unwrap();
                    let result = dispatch(command, shared.clone());
                    match result {
                        Ok(value) => ws_stream.send(Message::text(value.encode_resp())).await?,
                        Err(e) => {
                            println!("Error: {}", e);
                            ws_stream.send(Message::text(Value::SimpleError("SERVER ERROR".to_string()).encode_resp())).await?;
                        }
                    }
                }
            }

            println!("Connection from {} closed", ws_stream.get_ref().peer_addr().unwrap());
            Ok::<_, Error>(())
        });
    }

    Ok(())
}
