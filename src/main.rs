use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::RwLock;

pub mod background;
pub mod command;
pub mod data;
pub mod file;
pub mod info;
pub mod replication;
pub mod resp;
pub mod utils;

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

        let res = match resp::parse_input(&buf[..n]) {
            Ok(req) => command::handle(req, &data, &info).await,
            Err(e) => resp::RespOut::Error(format!("failed to parse: {}", e)),
        };

        stream.write_all(&res.serialize()).await?;

        // TODO: do this somewhere else
        match res {
            resp::RespOut::SimpleString(s) if s.to_uppercase().starts_with("FULLRESYNC") => {
                println!("LOLOL");
                let res = crate::file::construct_rdb_file(&data);
                stream.write_all(&res.serialize()).await?;
            }
            _ => {}
        }
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

    let role;
    let master_host;
    let master_port;

    if let Some(addr) = &args.replicaof {
        println!("(INFO) Replicating from {}", addr);
        role = info::ReplicaRole::SLAVE;
        master_host = Some(addr.split(" ").next().unwrap().to_string());
        master_port = Some(addr.split(" ").nth(1).unwrap().parse().unwrap());
    } else {
        role = info::ReplicaRole::MASTER;
        master_host = None;
        master_port = None;
    }

    // read-only to no mutex is needed
    let info = Arc::new(info::create_info(args.port, role, master_host, master_port));

    // Start background task
    if role == info::ReplicaRole::MASTER {
        let data = Arc::clone(&data);
        tokio::spawn(background::delete_expired(data));
    }

    // Replica task
    if role == info::ReplicaRole::SLAVE {
        let data = Arc::clone(&data);
        let info = Arc::clone(&info);
        tokio::spawn(replication::replication_task(data, info));
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
