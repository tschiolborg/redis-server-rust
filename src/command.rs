use crate::data::SharedData;
use crate::info::SharedInfo;
use crate::resp::{RespIn, RespOut};
use anyhow::{bail, Result};
use std::cell::Cell;

struct Args<'b> {
    items: &'b Vec<String>,
    pos: Cell<usize>,
}

// I think there is something wrong with lifetimes
unsafe impl Send for Args<'_> {}
unsafe impl Sync for Args<'_> {}

struct Handler<'a, 'b, 'c> {
    data: &'a SharedData,
    info: &'b SharedInfo,
    args: Args<'c>,
}

pub async fn handle(value: RespIn, data: &SharedData, info: &SharedInfo) -> Vec<RespOut> {
    match handle_value(value, data, info).await {
        Ok(res) => res,
        Err(e) => vec![RespOut::Error(format!("failed to handle: {}", e))],
    }
}

async fn handle_value(value: RespIn, data: &SharedData, info: &SharedInfo) -> Result<Vec<RespOut>> {
    match value {
        RespIn::Array(arr) => {
            let handler = Handler::new(data, info, Args::new(&arr));
            handler.handle().await
        }
    }
}

impl<'b> Args<'b> {
    fn new(items: &'b Vec<String>) -> Self {
        Self {
            items,
            pos: Cell::new(0),
        }
    }

    fn next(&self) -> Result<&String> {
        let pos = self.pos.get();
        if !self.has_next() {
            bail!("Missing argument number {}", pos + 1);
        }
        let res = &self.items[pos];
        self.pos.set(pos + 1);
        Ok(res)
    }

    fn has_next(&self) -> bool {
        self.items.len() > self.pos.get()
    }
}

type Resp = Result<Vec<RespOut>>;

impl<'a, 'b, 'c> Handler<'a, 'b, 'c> {
    fn new(data: &'a SharedData, info: &'b SharedInfo, args: Args<'c>) -> Handler<'a, 'b, 'c> {
        Self { data, info, args }
    }

    async fn handle(&self) -> Resp {
        let cmd = self.args.next()?;

        match cmd.to_uppercase().as_str() {
            "PING" => self.ping(),
            "ECHO" => self.echo(),
            "GET" => self.get().await,
            "SET" => self.set().await,
            "INFO" => self.info().await,
            "REPLCONF" => self.replconf(),
            "PSYNC" => self.psync(),
            _ => bail!("unknown command: {}", cmd),
        }
    }

    fn ping(&self) -> Resp {
        Ok(vec![RespOut::SimpleString("PONG".to_string())])
    }

    fn echo(&self) -> Resp {
        Ok(vec![RespOut::BulkString(self.args.next()?.clone())])
    }

    async fn get(&self) -> Resp {
        let data = self.data.read().await;

        let key = self.args.next()?.as_str();

        let res = match data.get(key) {
            Some(value) => RespOut::BulkString(value),
            None => RespOut::Null,
        };
        Ok(vec![res])
    }

    async fn set(&self) -> Resp {
        let key = self.args.next()?.clone();
        let value = self.args.next()?.clone();

        let mut px: Option<u128> = None;

        while self.args.has_next() {
            let arg = self.args.next()?;
            match arg.to_uppercase().as_str() {
                "PX" => px = Some(self.args.next()?.parse()?),
                s => bail!("Unknown argument {}", s),
            }
        }

        let mut data = self.data.write().await;

        data.set(key, value, px);

        Ok(vec![RespOut::SimpleString("OK".to_string())])
    }

    async fn info(&self) -> Resp {
        let res = match self.args.has_next() {
            false => self.info.get_all(),
            true => {
                let mut res = Vec::new();
                while self.args.has_next() {
                    let arg = self.args.next()?;
                    if let Some(s) = self.info.get_section(arg.as_str()) {
                        res.push(s);
                    }
                }
                res.join("\n")
            }
        };

        return Ok(vec![RespOut::BulkString(res)]);
    }

    fn replconf(&self) -> Resp {
        Ok(vec![RespOut::SimpleString("OK".to_string())])
    }

    fn psync(&self) -> Resp {
        Ok(vec![
            RespOut::SimpleString(
                format!(
                    "FULLRESYNC {} {}",
                    self.info.replication.master_replid(),
                    self.info.replication.master_repl_offset()
                )
                .to_string(),
            ),
            crate::file::construct_rdb_file(&self.data),
        ])
    }
}
