use crate::data::SharedData;
use crate::resp::RespOut;

// NOTE: the redis protocol expects the RDB file response to be on the form as a bulk string
//       except that it should not finish with a CRLF.
//       Which is bullshit, so we that is not done here and instead just return a normal bulk string.

const EMPTY_RDB: &str = "UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";

pub fn construct_rdb_file(_data: &SharedData) -> RespOut {
    RespOut::BulkString(EMPTY_RDB.to_string())
}
