use crate::resp::{RespIn, RespOut};
use crate::{data::SharedData, info::SharedInfo};
use anyhow::{bail, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn replication_task(data: SharedData, info: SharedInfo) {
    match handshake(data, info).await {
        Ok(_) => {
            println!("(INFO) Handshake completed");
        }
        Err(e) => {
            eprintln!("(ERROR) Handshake failed: {}", e);
            std::process::exit(1);
        }
    }
}

pub async fn handshake(_data: SharedData, info: SharedInfo) -> Result<()> {
    let mut stream = TcpStream::connect(info.replication.master_addr()).await?;

    let ping = RespIn::Array(vec!["PING".to_string()]);
    stream.write_all(&ping.serialize()).await?;
    expect_simple(&mut stream, "PONG").await?;

    let replconf = RespIn::Array(vec![
        "REPLCONF".to_string(),
        "listening-port".to_string(),
        info.server.port().to_string(),
    ]);
    stream.write_all(&replconf.serialize()).await?;
    expect_simple(&mut stream, "OK").await?;

    let replconf = RespIn::Array(vec![
        "REPLCONF".to_string(),
        "capa".to_string(),
        "psync2".to_string(),
    ]);
    stream.write_all(&replconf.serialize()).await?;
    expect_simple(&mut stream, "OK").await?;

    let psync = RespIn::Array(vec!["PSYNC".to_string(), "?".to_string(), "-1".to_string()]);
    stream.write_all(&psync.serialize()).await?;
    expect_full_resync(&mut stream).await?;

    Ok(())
}

async fn next_response(stream: &mut TcpStream) -> Result<RespOut> {
    let mut buf = [0; 512];
    let n = stream.read(&mut buf).await?;
    Ok(crate::resp::parse_output(&buf[..n])?)
}

async fn expect_simple(stream: &mut TcpStream, expected: &str) -> Result<()> {
    let res = next_response(stream).await?;
    match res {
        RespOut::SimpleString(s) if s.to_lowercase() == expected.to_lowercase() => Ok(()),
        _ => bail!("expected '{}'", expected),
    }
}

async fn expect_full_resync(stream: &mut TcpStream) -> Result<()> {
    let res = next_response(stream).await?;
    let (id, offset) = match res {
        RespOut::SimpleString(s) => {
            let mut iter = s.split_whitespace();
            match iter.next() {
                Some("FULLRESYNC") => {}
                _ => bail!("expected FULLRESYNC"),
            }
            let id = match iter.next() {
                Some(id) => id.to_string(),
                None => bail!("expected id"),
            };
            let offset = match iter.next() {
                Some(offset) => offset.to_string(),
                None => bail!("expected offset"),
            };
            (id, offset)
        }
        _ => bail!("expected Simple String"),
    };

    println!("(INFO) FULLRESYNC id={} offset={}", id, offset);

    Ok(())
}
