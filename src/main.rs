use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub mod command;
pub mod resp;

async fn handle_connection(mut stream: tokio::net::TcpStream) -> Result<()> {
    println!("accepted new connection");

    let mut buf = [0; 512];

    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        println!(
            "req: {:?}",
            buf[..n].iter().map(|b| *b as char).collect::<String>()
        );

        let res = match resp::parse(&buf[..n]) {
            Ok(req) => command::handle(req),
            Err(e) => resp::RespOut::Error(format!("failed to parse: {}", e)),
        };

        let res = resp::write_value(res);

        println!(
            "res: {:?}",
            res.to_vec()
                .into_iter()
                .map(|b| b as char)
                .collect::<String>()
        );

        stream.write_all(&res).await?;
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
