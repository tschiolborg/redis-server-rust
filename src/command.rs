use crate::data::SharedData;
use crate::resp::{RespIn, RespOut};
use anyhow::{bail, Result};

pub async fn handle(value: RespIn, data: &SharedData) -> RespOut {
    match handle_value(value, data).await {
        Ok(res) => res,
        Err(e) => RespOut::Error(format!("failed to handle: {}", e)),
    }
}

async fn handle_value(value: RespIn, data: &SharedData) -> Result<RespOut> {
    match value {
        RespIn::Array(mut arr) => {
            if arr.is_empty() {
                bail!("empty array");
            }
            let cmd = arr.remove(0); // not optimal with a lot of args

            let commands = Commands::new(data, &arr);

            match cmd.to_uppercase().as_str() {
                "PING" => commands.ping(),
                "ECHO" => commands.echo(),
                "GET" => commands.get().await,
                "SET" => commands.set().await,
                _ => bail!("unknown command: {}", cmd),
            }
        }
    }
}

struct Commands<'a> {
    data: &'a SharedData,
    args: &'a Vec<String>,
}

impl<'a> Commands<'a> {
    pub fn new(data: &'a SharedData, args: &'a Vec<String>) -> Commands<'a> {
        Self { data, args }
    }

    fn ping(&self) -> Result<RespOut> {
        Ok(RespOut::SimpleString("PONG".to_string()))
    }

    fn echo(&self) -> Result<RespOut> {
        if self.args.is_empty() {
            bail!("ECHO requires at least one argument")
        }
        Ok(RespOut::BulkString(self.args[0].clone()))
    }

    async fn get(&self) -> Result<RespOut> {
        if self.args.is_empty() {
            bail!("GET requires one argument")
        }
        let key = &self.args[0];

        let data = self.data.read().await;

        match data.get(key) {
            Some(value) => Ok(RespOut::BulkString(value)),
            None => Ok(RespOut::Null),
        }
    }

    async fn set(&self) -> Result<RespOut> {
        if self.args.len() != 2 {
            bail!("SET requires two arguments")
        }
        let key = self.args[0].clone();
        let value = self.args[1].clone();

        let mut data = self.data.write().await;

        data.set(key, value);

        Ok(RespOut::SimpleString("OK".to_string()))
    }
}
