use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

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
        let request = str::from_utf8(&buf).unwrap();
        println!("{:?}", request);
        stream.write_all(b"+PONG\r\n").await?;
    }
    stream.flush().await?;
    Ok(())
}
