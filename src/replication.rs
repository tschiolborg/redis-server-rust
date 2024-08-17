use crate::resp::{RespIn, RespOut};
use crate::{data::SharedData, info::SharedInfo};
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn replication_task(_data: SharedData, info: SharedInfo) -> Result<()> {
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
    expect_simple(&mut stream, "FULLRESYNC").await?; // TODO: get id and offset

    Ok(())
}

async fn expect_simple(stream: &mut TcpStream, expected: &str) -> Result<()> {
    let mut buf = [0; 512];
    let n = stream.read(&mut buf).await?;
    let res = crate::resp::parse_output(&buf[..n])?;
    match res {
        RespOut::SimpleString(s) if s.to_lowercase() == expected.to_lowercase() => Ok(()),
        _ => {
            eprintln!("(ERROR) expected '{}' ... exiting", expected);
            std::process::exit(0);
        }
    }
}
