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

pub fn int_to_float_byte(mut x: isize) -> isize {
    let mut e = 0;
    if x < 8 {
        return x;
    }

    while x >= (8 << 4) {
        x = (x + 0xf) >> 4;
        e += 4;
    }
    while x >= (8 << 1) {
        x = (x + 1) >> 1;
        e += 1;
    }

    ((e + 1) << 3) | (x - 8)
}