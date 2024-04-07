use anyhow::{bail, Result};

#[derive(Debug, Clone)]
pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<Vec<u8>>),
    Array(Vec<RespValue>),
}

pub fn parse_and_handle(buf: &[u8]) -> RespValue {
    let res = match parse_value(buf) {
        Ok(req) => handle_value(req),
        Err(e) => Err(e),
    };
    match res {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Failed to handle request: {}", e);
            eprintln!(
                "input: {:?}",
                buf.to_vec()
                    .into_iter()
                    .map(|b| b as char)
                    .collect::<String>()
            );
            RespValue::Error(e.to_string())
        }
    }
}

pub fn parse_value(buf: &[u8]) -> Result<RespValue> {
    let mut iter = buf.iter().copied();
    parse_value_inner(&mut iter)
}

fn parse_value_inner(iter: &mut impl Iterator<Item = u8>) -> Result<RespValue> {
    let res = match iter.next() {
        Some(b'+') => RespValue::SimpleString(parse_simple_string(iter)?),
        Some(b':') => RespValue::Integer(parse_integer(iter)?),
        Some(b'$') => RespValue::BulkString(parse_bulk_string(iter)?),
        Some(b'*') => RespValue::Array(parse_array(iter)?),
        Some(s) => bail!("unexpected byte {:?}", s as char),
        None => bail!("unexpected EOF"),
    };
    Ok(res)
}

fn parse_simple_string(iter: &mut impl Iterator<Item = u8>) -> Result<String> {
    let mut buf = Vec::new();

    loop {
        match iter.next() {
            Some(b'\r') => {
                if iter.next() != Some(b'\n') {
                    bail!("expected LF");
                }
                return Ok(String::from_utf8(buf)?);
            }
            Some(byte) => buf.push(byte),
            None => bail!("unexpected EOF"),
        }
    }
}

fn parse_integer(iter: &mut impl Iterator<Item = u8>) -> Result<i64> {
    let mut buf = Vec::new();

    loop {
        match iter.next() {
            Some(b'\r') => {
                if iter.next() != Some(b'\n') {
                    bail!("expected LF");
                }
                return Ok(String::from_utf8(buf)?.parse::<i64>()?);
            }
            Some(byte) => buf.push(byte),
            None => bail!("unexpected EOF"),
        }
    }
}

fn parse_bulk_string(iter: &mut impl Iterator<Item = u8>) -> Result<Option<Vec<u8>>> {
    let n = parse_integer(iter)?;
    if n == -1 {
        return Ok(None);
    }

    let mut buf = vec![0; n as usize];
    for i in 0..n as usize {
        match iter.next() {
            Some(byte) => buf[i] = byte,
            None => bail!("unexpected EOF"),
        }
    }

    if iter.next() != Some(b'\r') {
        bail!("expected CR");
    }
    if iter.next() != Some(b'\n') {
        bail!("expected LF");
    }

    Ok(Some(buf))
}

fn parse_array(iter: &mut impl Iterator<Item = u8>) -> Result<Vec<RespValue>> {
    let n = parse_integer(iter)?;
    let mut res = Vec::new();
    for _ in 0..n {
        res.push(parse_value_inner(iter)?);
    }
    Ok(res)
}

pub fn handle_value(value: RespValue) -> Result<RespValue> {
    match value {
        RespValue::Array(mut values) => {
            if values.is_empty() {
                bail!("empty array");
            }

            let command = match values.remove(0) {
                RespValue::SimpleString(s) => s,
                RespValue::BulkString(Some(data)) => String::from_utf8(data)?,
                _ => bail!("expected command"),
            };

            let args = values;

            match command.to_uppercase().as_str() {
                "PING" => Ok(RespValue::SimpleString("PONG".into())),
                "ECHO" => {
                    if args.len() != 1 {
                        bail!("expected 1 argument");
                    }
                    Ok(args[0].clone())
                }
                _ => bail!("unknown command"),
            }
        }
        _ => bail!("expected array"),
    }
}

pub fn write_value(value: RespValue) -> Vec<u8> {
    let mut buf = Vec::new();
    write_value_inner(&mut buf, &value);
    buf
}

fn write_value_inner(buf: &mut Vec<u8>, value: &RespValue) {
    match value {
        RespValue::SimpleString(s) => {
            buf.push(b'+');
            buf.extend(s.as_bytes());
            buf.extend(b"\r\n");
        }
        RespValue::Error(e) => {
            buf.push(b'-');
            buf.extend(b"ERR ");
            buf.extend(e.as_bytes());
            buf.extend(b"\r\n");
        }
        RespValue::Integer(i) => {
            buf.push(b':');
            buf.extend(i.to_string().as_bytes());
            buf.extend(b"\r\n");
        }
        RespValue::BulkString(Some(data)) => {
            buf.push(b'$');
            buf.extend(data.len().to_string().as_bytes());
            buf.extend(b"\r\n");
            buf.extend(data);
            buf.extend(b"\r\n");
        }
        RespValue::BulkString(None) => {
            buf.push(b'$');
            buf.extend(b"-1\r\n");
        }
        RespValue::Array(values) => {
            buf.push(b'*');
            buf.extend(values.len().to_string().as_bytes());
            buf.extend(b"\r\n");
            for value in values {
                write_value_inner(buf, value);
            }
        }
    }
}
