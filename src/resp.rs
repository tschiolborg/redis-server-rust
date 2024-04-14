use anyhow::{bail, Result};

pub enum RespIn {
    Array(Vec<String>),
}

pub enum RespOut {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<RespOut>),
    Null,
}

pub fn parse(buf: &[u8]) -> Result<RespIn> {
    println!(
        "req: {:?}",
        buf.iter().map(|b| *b as char).collect::<String>()
    );

    let mut parser = RespParser::new(buf);
    let values = parser.parse_full()?;
    Ok(RespIn::Array(values))
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

impl RespOut {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        serialize(&mut buf, &self);

        println!(
            "res: {:?}",
            buf.to_vec()
                .into_iter()
                .map(|b| b as char)
                .collect::<String>()
        );
        buf
    }
}

fn serialize(buf: &mut Vec<u8>, value: &RespOut) {
    match value {
        RespOut::SimpleString(s) => {
            buf.push(b'+');
            buf.extend(s.as_bytes());
            push_crlf(buf);
        }
        RespOut::Error(e) => {
            buf.push(b'-');
            buf.extend(b"ERR ");
            buf.extend(e.as_bytes());
            push_crlf(buf);
        }
        RespOut::Integer(i) => {
            buf.push(b':');
            buf.extend(i.to_string().as_bytes());
            push_crlf(buf);
        }
        RespOut::BulkString(s) => {
            buf.push(b'$');
            buf.extend(s.len().to_string().as_bytes());
            push_crlf(buf);
            buf.extend(s.as_bytes());
            push_crlf(buf);
        }
        RespOut::Null => {
            buf.push(b'$');
            buf.extend(b"-1");
            push_crlf(buf);
        }
        RespOut::Array(values) => {
            buf.push(b'*');
            buf.extend(values.len().to_string().as_bytes());
            push_crlf(buf);
            for value in values {
                serialize(buf, value);
            }
        }
    }
}

fn push_crlf(buf: &mut Vec<u8>) {
    buf.extend(b"\r\n");
}
