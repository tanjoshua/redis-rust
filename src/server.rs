use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::resp::{RESPData, parse_resp};

pub async fn run() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (stream, _address) = listener.accept().await?;
        tokio::spawn(async move { handle_connection(stream).await });
    }
}

async fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
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
                _ => &RESPData::SimpleString(String::from("Unimplemented")),
            },
            _ => anyhow::bail!("Invalid RESP command"),
        };

        stream.write_all(Vec::<u8>::from(res).as_slice()).await?;
    }
    stream.flush().await?;
    Ok(())
}
