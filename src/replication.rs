use crate::resp::RespOut;
use crate::{data::SharedData, info::SharedInfo};
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn replication_task(data: SharedData, info: SharedInfo) -> Result<()> {
    let mut stream = TcpStream::connect(info.replication.master_addr().unwrap()).await?;

    let handshake = RespOut::Array(vec![
        RespOut::BulkString("PING".to_string()),
        RespOut::BulkString("PING".to_string()),
    ]);

    stream.write_all(&handshake.serialize()).await?;

    Ok(())
}
