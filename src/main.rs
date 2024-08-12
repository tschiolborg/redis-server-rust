use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::RwLock;

pub mod background;
pub mod command;
pub mod data;
pub mod info;
pub mod resp;

async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    data: data::SharedData,
    info: info::SharedInfo,
) -> Result<()> {
    println!("(INFO) Accepted new connection");

    let mut buf = [0; 512];

    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        let res = match resp::parse(&buf[..n]) {
            Ok(req) => command::handle(req, &data, &info).await,
            Err(e) => resp::RespOut::Error(format!("failed to parse: {}", e)),
        };

        stream.write_all(&res.serialize()).await?;
    }

    Ok(())
}

/// Start a Redis server
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 6379)]
    port: u16,

    /// Config for replication
    #[arg(long)]
    replicaof: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let addr = format!("127.0.0.1:{}", args.port);

    println!("(INFO) Listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;

    let data = Arc::new(RwLock::new(data::InMemoryData::new()));

    let role = match args.replicaof {
        Some(addr) => {
            println!("(INFO) Replicating from {}", addr);
            info::ReplicaRole::SLAVE
        }
        None => {
            println!("(INFO) Starting as master");
            info::ReplicaRole::MASTER
        }
    };

    let info = Arc::new(RwLock::new(info::create_info(role)));

    // Start background tasks
    {
        let data = Arc::clone(&data);
        tokio::spawn(background::delete_expired(data));
    }

    loop {
        let (stream, _) = listener.accept().await?;
        let data = Arc::clone(&data);
        let info = Arc::clone(&info);
        tokio::spawn(async move {
            let _ = handle_connection(stream, data, info).await;
        });
    }
}
