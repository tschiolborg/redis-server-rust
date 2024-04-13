use anyhow::{bail, Result};

pub enum RespIn {
    Array(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum RespOut {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Vec<RespOut>),
}

pub struct RespParser<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl RespParser<'_> {
    pub fn new(buf: &[u8]) -> RespParser {
        RespParser { buf, pos: 0 }
    }

    pub fn parse_full(&mut self) -> Result<Vec<String>> {
        self.next_array()
    }

    fn next(&mut self) -> Result<u8> {
        if self.buf.len() <= self.pos {
            bail!("unexpected EOF");
        }
        let res = self.buf[self.pos];
        self.pos += 1;
        Ok(res)
    }

    fn next_line(&mut self) -> Result<String> {
        let mut buf = Vec::new();

        loop {
            match self.next()? {
                b'\r' => {
                    if self.next()? != b'\n' {
                        bail!("expected LF");
                    }
                    return Ok(String::from_utf8(buf)?);
                }
                byte => buf.push(byte),
            }
        }
    }

    fn next_int(&mut self) -> Result<i64> {
        self.next_line()?.parse::<i64>().map_err(Into::into)
    }

    fn next_string(&mut self) -> Result<String> {
        self.consume_type(b'$')?;
        let n = self.next_int()?;
        if n < 1 {
            bail!("fuck null strings")
        }
        self.next_line().map_err(Into::into)
    }

    fn next_array(&mut self) -> Result<Vec<String>> {
        self.consume_type(b'*')?;
        let n = self.next_int()?;
        let mut res = Vec::new();
        for _ in 0..n {
            res.push(self.next_string()?);
        }
        Ok(res)
    }

    fn consume_type(&mut self, expected: u8) -> Result<()> {
        match self.next()? {
            s if s == expected => return Ok(()),
            s => bail!("unexpected data type {:?}", s),
        }
    }
}

pub fn handle_value(value: RespIn) -> Result<RespOut> {
    match value {
        RespIn::Array(mut values) => {
            if values.is_empty() {
                bail!("empty array");
            }

            let command = values.remove(0);

            let args = values;

            match command.to_uppercase().as_str() {
                "PING" => Ok(RespOut::SimpleString("PONG".into())),
                "ECHO" => {
                    if args.len() != 1 {
                        bail!("expected 1 argument");
                    }
                    Ok(RespOut::BulkString(Some(args[0].clone())))
                }
                _ => bail!("unknown command"),
            }
        }
    }
}

pub fn write_value(value: RespOut) -> Vec<u8> {
    let mut buf = Vec::new();
    write_value_inner(&mut buf, &value);
    buf
}

fn write_value_inner(buf: &mut Vec<u8>, value: &RespOut) {
    match value {
        RespOut::SimpleString(s) => {
            buf.push(b'+');
            buf.extend(s.as_bytes());
            buf.extend(b"\r\n");
        }
        RespOut::Error(e) => {
            buf.push(b'-');
            buf.extend(b"ERR ");
            buf.extend(e.as_bytes());
            buf.extend(b"\r\n");
        }
        RespOut::Integer(i) => {
            buf.push(b':');
            buf.extend(i.to_string().as_bytes());
            buf.extend(b"\r\n");
        }
        RespOut::BulkString(Some(data)) => {
            buf.push(b'$');
            buf.extend(data.len().to_string().as_bytes());
            buf.extend(b"\r\n");
            buf.extend(data.as_bytes());
            buf.extend(b"\r\n");
        }
        RespOut::BulkString(None) => {
            buf.push(b'$');
            buf.extend(b"-1\r\n");
        }
        RespOut::Array(values) => {
            buf.push(b'*');
            buf.extend(values.len().to_string().as_bytes());
            buf.extend(b"\r\n");
            for value in values {
                write_value_inner(buf, value);
            }
        }
    }
}
