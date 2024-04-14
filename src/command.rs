use crate::resp::{RespIn, RespOut};
use anyhow::{bail, Result};

pub fn handle(value: RespIn) -> RespOut {
    match handle_value(value) {
        Ok(res) => res,
        Err(e) => RespOut::Error(format!("failed to handle: {}", e)),
    }
}

fn handle_value(value: RespIn) -> Result<RespOut> {
    match value {
        RespIn::Array(mut arr) => {
            if arr.is_empty() {
                bail!("empty array");
            }
            let cmd = arr.remove(0);
            match cmd.to_uppercase().as_str() {
                "PING" => Ping::handle(&arr),
                "ECHO" => Echo::handle(&arr),
                "GET" => Get::handle(&arr),
                _ => bail!("unknown command: {}", cmd),
            }
        }
    }
}

pub trait Command {
    fn handle(args: &Vec<String>) -> Result<RespOut>;
}

struct Ping {}
struct Echo {}

struct Get {}

impl Command for Ping {
    fn handle(_args: &Vec<String>) -> Result<RespOut> {
        Ok(RespOut::SimpleString("PONG".to_string()))
    }
}

impl Command for Echo {
    fn handle(args: &Vec<String>) -> Result<RespOut> {
        if args.is_empty() {
            bail!("ECHO requires at least one argument")
        }
        Ok(RespOut::BulkString(args[0].clone()))
    }
}

impl Command for Get {
    fn handle(_args: &Vec<String>) -> Result<RespOut> {
        Ok(RespOut::Null)
    }
}
