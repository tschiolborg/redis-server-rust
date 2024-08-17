use anyhow::{bail, Result};
use std::cell::Cell;

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

pub fn parse_input(buf: &[u8]) -> Result<RespIn> {
    crate::utils::print_buf(buf, " in req");
    let parser = RespParser::new(buf);
    parser.parse_request()
}

pub fn parse_output(buf: &[u8]) -> Result<RespOut> {
    crate::utils::print_buf(buf, "out req");
    let parser = RespParser::new(buf);
    parser.parse_response()
}

pub struct RespParser<'a> {
    buf: &'a [u8],
    pos: Cell<usize>,
}

impl RespParser<'_> {
    fn new(buf: &[u8]) -> RespParser {
        RespParser {
            buf,
            pos: Cell::new(0),
        }
    }

    fn parse_request(&self) -> Result<RespIn> {
        let values = self.next_array_of_strings()?;
        Ok(RespIn::Array(values))
    }

    fn parse_response(&self) -> Result<RespOut> {
        Ok(self.next_item()?)
    }

    fn next(&self) -> Result<u8> {
        let pos = self.pos.get();
        if self.buf.len() <= pos {
            bail!("unexpected EOF");
        }
        let res = self.buf[pos];
        self.pos.set(pos + 1);
        Ok(res)
    }

    fn next_line(&self) -> Result<String> {
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

    fn next_item(&self) -> Result<RespOut> {
        let item = match self.next()? {
            SIMPLE_STRING_BYTE_CODE => RespOut::SimpleString(self.next_line()?),
            ERROR_BYTE_CODE => RespOut::Error(self.next_line()?),
            INTEGER_BYTE_CODE => RespOut::Integer(self.next_int()?),
            BULK_STRING_BYTE_CODE => RespOut::BulkString(self.next_string()?),
            ARRAY_BYTE_CODE => RespOut::Array(self.next_array()?),
            NULL_BYTE_CODE => RespOut::Null,
            byte => bail!("unexpected data type {:?}", byte),
        };
        Ok(item)
    }

    fn next_int(&self) -> Result<i64> {
        self.next_line()?.parse::<i64>().map_err(Into::into)
    }

    fn next_string(&self) -> Result<String> {
        let n = self.next_int()?;
        if n < 0 {
            bail!("fuck null strings")
        }
        self.next_line()
    }

    fn next_array(&self) -> Result<Vec<RespOut>> {
        let n = self.next_int()?;
        let mut res = Vec::new();
        for _ in 0..n {
            res.push(self.next_item()?);
        }
        Ok(res)
    }

    fn next_array_of_strings(&self) -> Result<Vec<String>> {
        self.consume_type(ARRAY_BYTE_CODE)?;
        let n = self.next_int()?;
        let mut res = Vec::new();
        for _ in 0..n {
            self.consume_type(BULK_STRING_BYTE_CODE)?;
            res.push(self.next_string()?);
        }
        Ok(res)
    }

    fn consume_type(&self, expected: u8) -> Result<()> {
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

        crate::utils::print_buf(&buf, "out res");
        buf
    }
}

impl RespIn {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            RespIn::Array(values) => {
                buf.push(ARRAY_BYTE_CODE);
                buf.extend(values.len().to_string().as_bytes());
                push_crlf(&mut buf);
                for value in values {
                    buf.push(BULK_STRING_BYTE_CODE);
                    buf.extend(value.len().to_string().as_bytes());
                    push_crlf(&mut buf);
                    buf.extend(value.as_bytes());
                    push_crlf(&mut buf);
                }
            }
        }

        crate::utils::print_buf(&buf, " in res");
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
