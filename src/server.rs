use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::time::Instant;

use crate::resp::{RESPData, parse_resp};
use crate::store::{RedisData, Store};

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
                    if resp_commands.len() < 3 {
                        anyhow::bail!("Invalid SET command")
                    }
                    let (RESPData::BulkString(key), RESPData::BulkString(value)) =
                        (&resp_commands[1], &resp_commands[2])
                    else {
                        anyhow::bail!("SET requires a key and value")
                    };

                    let mut i = 3;
                    let mut expiry = None;
                    while i < resp_commands.len() {
                        let RESPData::BulkString(option) = &resp_commands[i] else {
                            anyhow::bail!("Invalid option")
                        };
                        let RESPData::BulkString(option_value) = &resp_commands[i + 1] else {
                            anyhow::bail!("Invalid option value")
                        };

                        match option.as_slice() {
                            b"EX" => {}
                            b"PX" => {
                                let validity_str = str::from_utf8(option_value)?;
                                let validity_in_ms = validity_str.parse::<u64>()?;

                                let validity_duration = Duration::from_millis(validity_in_ms);
                                expiry = Some(Instant::now() + validity_duration);
                            }
                            _ => {
                                anyhow::bail!("Unrecognized option")
                            }
                        }

                        i += 2;
                    }

                    data_store.write().await.insert(
                        key.clone(),
                        RedisData {
                            value: value.clone(),
                            expiry,
                        },
                    );
                    &RESPData::SimpleString(String::from("OK"))
                }
                b"GET" => {
                    let [_, RESPData::BulkString(key)] = resp_commands.as_slice() else {
                        anyhow::bail!("SET requires a key and value")
                    };

                    let store = data_store.write().await;
                    match store.get(key) {
                        Some(rdata) => {
                            let is_expired = rdata.expiry.is_some_and(|e| e <= Instant::now());
                            if is_expired {
                                &RESPData::NullBulkString
                            } else {
                                &RESPData::BulkString(rdata.value.clone())
                            }
                        }
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
