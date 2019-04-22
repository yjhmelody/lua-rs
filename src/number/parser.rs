#![allow(non_upper_case_globals)]

use regex::bytes::Regex;

use crate::compiler::error::{Error, Result};

lazy_static! {
    static ref re_integer: Regex = Regex::new(r#"^[+-]?[0-9]+$|^-?0x[0-9a-f]+$"#).unwrap();
    static ref re_hex_float: Regex = Regex::new(r#"^([0-9a-f]+(\.[0-9a-f]*)?|([0-9a-f]*\.[0-9a-f]+))(p[+\-]?[0-9]+)?$"#).unwrap();
}


pub fn parse_number(num: String) -> Result<i64> {
    // todo: supports total syntax
    // trim space
    // to lower
//    let mut i = 0;
//    if re_integer.find(&bytes).is_none() {
//        Err(Error::IllegalToken)
//    } else if bytes[0] == b'+' {
//        i = 1;
//    }

    //decimal

    num.parse::<i64>().or(Err(Error::IllegalToken {
        line: 0,
    }))
}
