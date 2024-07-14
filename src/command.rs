use crate::data::SharedData;
use crate::resp::{RespIn, RespOut};
use anyhow::{bail, Result};

type Args = Vec<String>;

pub struct Handler {
    data: SharedData,
}

impl Handler {
    pub fn new(data: SharedData) -> Self {
        Self { data }
    }

    pub async fn handle(&self, value: RespIn) -> RespOut {
        match self.handle_value(value).await {
            Ok(res) => res,
            Err(e) => RespOut::Error(format!("failed to handle: {}", e)),
        }
    }

    async fn handle_value(&self, value: RespIn) -> Result<RespOut> {
        match value {
            RespIn::Array(mut arr) => {
                if arr.is_empty() {
                    bail!("empty array");
                }
                let cmd = arr.remove(0);
                let args = &arr;
                match cmd.to_uppercase().as_str() {
                    "PING" => self.ping(args),
                    "ECHO" => self.echo(args),
                    "GET" => self.get(args).await,
                    "SET" => self.set(args).await,
                    _ => bail!("unknown command: {}", cmd),
                }
            }
        }
    }

    fn ping(&self, _args: &Args) -> Result<RespOut> {
        Ok(RespOut::SimpleString("PONG".to_string()))
    }

    fn echo(&self, args: &Args) -> Result<RespOut> {
        if args.is_empty() {
            bail!("ECHO requires at least one argument")
        }
        Ok(RespOut::BulkString(args[0].clone()))
    }

    async fn get(&self, args: &Args) -> Result<RespOut> {
        if args.is_empty() {
            bail!("GET requires one argument")
        }
        let key = &args[0];

        let data = self.data.read().await;

        match data.get(key) {
            Some(value) => Ok(RespOut::BulkString(value)),
            None => Ok(RespOut::Null),
        }
    }

    async fn set(&self, args: &Args) -> Result<RespOut> {
        if args.len() != 2 {
            bail!("SET requires two arguments")
        }
        let key = args[0].clone();
        let value = args[1].clone();

        let mut data = self.data.write().await;

        data.set(key, value);

        Ok(RespOut::SimpleString("OK".to_string()))
    }
}
