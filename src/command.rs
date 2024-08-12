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

pub async fn handle(value: RespIn, data: &SharedData, info: &SharedInfo) -> RespOut {
    match handle_value(value, data, info).await {
        Ok(res) => res,
        Err(e) => RespOut::Error(format!("failed to handle: {}", e)),
    }
}

async fn handle_value(value: RespIn, data: &SharedData, info: &SharedInfo) -> Result<RespOut> {
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

impl<'a, 'b, 'c> Handler<'a, 'b, 'c> {
    fn new(data: &'a SharedData, info: &'b SharedInfo, args: Args<'c>) -> Handler<'a, 'b, 'c> {
        Self { data, info, args }
    }

    async fn handle(&self) -> Result<RespOut> {
        let cmd = self.args.next()?;

        match cmd.to_uppercase().as_str() {
            "PING" => self.ping(),
            "ECHO" => self.echo(),
            "GET" => self.get().await,
            "SET" => self.set().await,
            "INFO" => self.info().await,
            _ => bail!("unknown command: {}", cmd),
        }
    }

    fn ping(&self) -> Result<RespOut> {
        Ok(RespOut::SimpleString("PONG".to_string()))
    }

    fn echo(&self) -> Result<RespOut> {
        Ok(RespOut::BulkString(self.args.next()?.clone()))
    }

    async fn get(&self) -> Result<RespOut> {
        let data = self.data.read().await;

        let key = self.args.next()?.as_str();

        match data.get(key) {
            Some(value) => Ok(RespOut::BulkString(value)),
            None => Ok(RespOut::Null),
        }
    }

    async fn set(&self) -> Result<RespOut> {
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

        Ok(RespOut::SimpleString("OK".to_string()))
    }

    async fn info(&self) -> Result<RespOut> {
        let info = self.info;

        let mut res = Vec::new();

        let mut sections = std::collections::HashSet::new();

        while self.args.has_next() {
            let arg = self.args.next()?;
            sections.insert(arg.as_str());
        }

        for (k, v) in info.iter() {
            if !sections.is_empty() && !sections.contains(&k.as_str()) {
                continue;
            }
            res.push(format!("# {}\n", k));
            for (k, v) in v.iter() {
                res.push(format!("{}:{}\n", k, v));
            }
            res.push("\n".to_string());
        }

        return Ok(RespOut::BulkString(res.join("")));
    }
}
