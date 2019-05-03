#![allow(non_upper_case_globals)]

use regex::bytes::Regex;

use crate::compiler::error::{Error, Result};

lazy_static! {
    static ref re_integer: Regex = Regex::new(r#"^[+-]?[0-9]+$|^-?0x[0-9a-f]+$"#).unwrap();
    static ref re_hex_float: Regex = Regex::new(r#"^([0-9a-f]+(\.[0-9a-f]*)?|([0-9a-f]*\.[0-9a-f]+))(p[+\-]?[0-9]+)?$"#).unwrap();
}

pub fn parse_float(num: String) -> Result<f64> {
    // todo: supports total syntax
    num.parse::<f64>().or(Err(Error::IllegalToken {
        line: 0,
    }))
}

pub fn parse_integer(num: String) -> Result<i64> {
    num.parse::<i64>().or(Err(Error::IllegalToken {
        line: 0,
    }))
}