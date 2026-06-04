use std::collections::HashMap;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

use crate::resp::{RESPData, parse_resp};

type Store = Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>;

pub async fn run() -> anyhow::Result<()> {
    let data_store: Store = Arc::new(RwLock::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (stream, _address) = listener.accept().await?;
        let task_data_store = Arc::clone(&data_store);
        tokio::spawn(async move { handle_connection(stream, task_data_store).await });
    }
}

async fn handle_connection(mut stream: TcpStream, data_store: Store) -> anyhow::Result<()> {
    let mut buf = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buf).await?;
        if bytes_read == 0 {
            // client disconnected
            break;
        }
        let resp_data = parse_resp(&buf[..bytes_read])?;
        println!("{:?}", resp_data);

        let RESPData::Array(resp_commands) = resp_data else {
            anyhow::bail!("Invalid RESP command")
        };

        if resp_commands.is_empty() {
            anyhow::bail!("Empty RESP array")
        }

        let res = match &resp_commands[0] {
            RESPData::BulkString(bulk_string) => match bulk_string.as_slice() {
                b"PING" => &RESPData::SimpleString(String::from("PONG")),
                b"ECHO" => &resp_commands[1],
                b"SET" => {
                    let [_, RESPData::BulkString(key), RESPData::BulkString(value)] =
                        resp_commands.as_slice()
                    else {
                        anyhow::bail!("SET requires a key and value")
                    };

                    data_store.write().await.insert(key.clone(), value.clone());
                    &RESPData::SimpleString(String::from("OK"))
                }
                b"GET" => {
                    let [_, RESPData::BulkString(key)] = resp_commands.as_slice() else {
                        anyhow::bail!("SET requires a key and value")
                    };

                    match data_store.read().await.get(key) {
                        Some(value) => &RESPData::BulkString(value.clone()),
                        None => &RESPData::NullBulkString,
                    }
                }
                _ => &RESPData::SimpleString(String::from("Unimplemented")),
            },
            _ => anyhow::bail!("Invalid RESP command"),
        };

        stream.write_all(Vec::<u8>::from(res).as_slice()).await?;
    }
    stream.flush().await?;
    Ok(())
}
