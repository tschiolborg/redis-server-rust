use anyhow::{bail, Result};

/// Input is always a list of BulkStrings
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

const SIMPLE_STRING_BYTE_CODE: u8 = b'+';
const ERROR_BYTE_CODE: u8 = b'-';
const INTEGER_BYTE_CODE: u8 = b':';
const BULK_STRING_BYTE_CODE: u8 = b'$';
const ARRAY_BYTE_CODE: u8 = b'*';
const NULL_BYTE_CODE: u8 = b'_';

pub fn parse(buf: &[u8]) -> Result<RespIn> {
    println!(
        "  (DEBUG) req: {:?}",
        buf.iter().map(|b| *b as char).collect::<String>()
    );

    let mut parser = RespParser::new(buf);
    parser.parse_full()
}

pub struct RespParser<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl RespParser<'_> {
    pub fn new(buf: &[u8]) -> RespParser {
        RespParser { buf, pos: 0 }
    }

    pub fn parse_full(&mut self) -> Result<RespIn> {
        let values = self.next_array()?;
        Ok(RespIn::Array(values))
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
        self.consume_type(BULK_STRING_BYTE_CODE)?;
        let n = self.next_int()?;
        if n < 0 {
            bail!("fuck null strings")
        }
        self.next_line()
    }

    fn next_array(&mut self) -> Result<Vec<String>> {
        self.consume_type(ARRAY_BYTE_CODE)?;
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
            "  (DEBUG) res: {:?}",
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
            buf.push(SIMPLE_STRING_BYTE_CODE);
            buf.extend(s.as_bytes());
            push_crlf(buf);
        }
        RespOut::Error(e) => {
            buf.push(ERROR_BYTE_CODE);
            buf.extend(b"ERR ");
            buf.extend(e.as_bytes());
            push_crlf(buf);
        }
        RespOut::Integer(i) => {
            buf.push(INTEGER_BYTE_CODE);
            buf.extend(i.to_string().as_bytes());
            push_crlf(buf);
        }
        RespOut::BulkString(s) => {
            buf.push(BULK_STRING_BYTE_CODE);
            buf.extend(s.len().to_string().as_bytes());
            push_crlf(buf);
            buf.extend(s.as_bytes());
            push_crlf(buf);
        }
        RespOut::Null => {
            buf.push(NULL_BYTE_CODE);
            push_crlf(buf);
        }
        RespOut::Array(values) => {
            buf.push(ARRAY_BYTE_CODE);
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
