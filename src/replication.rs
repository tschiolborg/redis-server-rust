use crate::resp::{RespIn, RespOut};
use crate::{data::SharedData, info::SharedInfo};
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn replication_task(_data: SharedData, info: SharedInfo) -> Result<()> {
    let mut stream = TcpStream::connect(info.replication.master_addr().unwrap()).await?;

    let ping = RespIn::Array(vec!["PING".to_string()]);

    stream.write_all(&ping.serialize()).await?;
    expect_pong(&mut stream).await?;

    Ok(())
}

async fn expect_pong(stream: &mut TcpStream) -> Result<()> {
    let mut buf = [0; 512];
    let n = stream.read(&mut buf).await?;
    let res = crate::resp::parse_output(&buf[..n])?;
    match res {
        RespOut::SimpleString(s) if s == "PONG" => Ok(()),
        _ => anyhow::bail!("expected PONG"),
    }
}
