use anyhow::Result;
use tokio::net::TcpListener;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn handle_connection(mut stream: tokio::net::TcpStream) -> Result<()> {
    println!("accepted new connection");

    let mut buf = [0; 512];

    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        stream.write_all(b"+PONG\r\n").await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let _ = handle_connection(stream).await;
        });
    }
}
