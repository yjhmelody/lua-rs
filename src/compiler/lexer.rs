#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::str;

use regex::bytes::Regex;

use crate::compiler::error::*;
use crate::compiler::token::Token;

/// 代码原位置，用于代码生成的信息
pub type Line = usize;

#[derive(Debug)]
pub struct Lexer {
    /// 源码
    chunk: Vec<u8>,
    /// 当前位置
    index: usize,
    /// 源文件名
    chunk_name: String,
    /// 当前行号
    line: Line,
    /// 缓存前看token
    next_tok: Result<Token>,
    next_line: Line,
}

#[derive(Debug, Clone, Copy)]
pub struct WithPosition<T> {
    pub node: T,
    pub line: usize,
}

impl<T> WithPosition<T> {
    #[inline]
    pub fn new(node: T, line: usize) -> Self {
        WithPosition { node, line }
    }
}

impl<T: Display> Display for WithPosition<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "node: {}, line: {}", self.node, self.line)
    }
}

lazy_static! {
    static ref keywords: HashMap<&'static str, Token> = {
        let mut m = HashMap::new();
        m.insert("and", Token::OpAnd);
        m.insert("or", Token::OpOr);
        m.insert("not", Token::OpNot);
        m.insert("function", Token::KwFunction);
        m.insert("break", Token::KwBreak);
        m.insert("return", Token::KwReturn);
        m.insert("local", Token::KwLocal);
        m.insert("if", Token::KwIf);
        m.insert("else", Token::KwElse);
        m.insert("elseif", Token::KwElseIf);
        m.insert("goto", Token::KwGoto);
        m.insert("do", Token::KwDo);
        m.insert("end", Token::KwEnd);
        m.insert("then", Token::KwThen);
        m.insert("until", Token::KwUntil);
        m.insert("repeat", Token::KwRepeat);
        m.insert("while", Token::KwWhile);
        m.insert("for", Token::KwFor);
        m.insert("in", Token::KwIn);
        m.insert("true", Token::KwTrue);
        m.insert("false", Token::KwFalse);
        m.insert("nil", Token::KwNil);
        m
    };
    static ref re_long_bracket: Regex = Regex::new(r##"^(?P<comment>\[=*\[(?P<string>.*?)\]=*\])"##).unwrap();
    static ref re_short_str: Regex = Regex::new(r##"(^'(\\\\|\\' | \\\n|\\z\s*|[^'\n])*')|^"(\\\\|\\' | \\\n|\\z\s*|[^'\n])*""##).unwrap();
    static ref re_number: Regex = Regex::new(r#"^0[xX][[:xdigit:]]*(\.[[:xdigit:]]*)?([pP][+\-]?[[:digit:]]+)?|^[[:digit:]]*(\.[[:digit:]]*)?([eE][+\-]?[[:digit:]]+)?"#).unwrap();
    static ref re_ident: Regex = Regex::new(r##"^[_\d\w]+"##).unwrap();
    static ref re_dec_escaped_seq: Regex = Regex::new(r##"^\\[0-9]{1,3}"##).unwrap();
    static ref re_hex_escaped_seq: Regex = Regex::new(r##"^\\[[:xdigit]]{2}"##).unwrap();
    static ref re_unicode_escaped_seq: Regex = Regex::new(r##"^\\u\{[[:xdigit]]+\}"##).unwrap();
}

impl Lexer {
    /// 从String中创建词法分析器
    #[inline]
    pub fn new(chunk: String, chunk_name: String) -> Self {
        Self {
            chunk: chunk.into_bytes(),
            index: 0,
            chunk_name,
            line: 1,
            next_tok: Err(Error::IllegalToken),
            next_line: 0,
        }
    }

    /// 从IntoIterator中创建词法分析器
    #[inline]
    pub fn from_iter<T: IntoIterator<Item=u8>>(iter: T, chunk_name: String) -> Self {
        let chunk = iter.into_iter().collect();
        Self {
            chunk,
            index: 0,
            chunk_name,
            line: 1,
            next_tok: Err(Error::IllegalToken),
            next_line: 0,
        }
    }

    /// 返回当前token的行号
    #[inline]
    pub fn current_line(&self) -> Line {
        self.line
    }

    /// 向前查看1个token
    pub fn look_ahead(&mut self) -> Result<Token> {
        // 检查是否已经缓存
        if self.next_line > 0 {
            self.next_tok.clone()
        } else {
            let cur_line = self.current_line();
            self.next_tok = self.next_token();
            self.next_line = self.current_line();
            self.line = cur_line;
            self.next_tok.clone()
        }
    }

    /// 返回当前单个字符的token
    #[inline]
    fn simple_token(&mut self, token: Token) -> Result<Token> {
        self.next(1);
        Ok(token)
    }
    /// 返回下一个token
    pub fn next_token(&mut self) -> Result<Token> {
        if self.next_line > 0 {
            self.line = self.next_line;
            self.next_line = 0;
            return self.next_tok.clone();
        }

        self.skip_whitespaces()?;
        let ch = match self.current() {
            Some(ch) => ch,
            None => {
                return Err(Error::EOF);
            }
        };

        match ch {
            b';' => self.simple_token(Token::SepSemi),
            b',' => self.simple_token(Token::SepComma),
            b'(' => self.simple_token(Token::SepLparen),
            b')' => self.simple_token(Token::SepRparen),
            b']' => self.simple_token(Token::SepRbrack),
            b'{' => self.simple_token(Token::SepLcurly),
            b'}' => self.simple_token(Token::SepRcurly),
            b'+' => self.simple_token(Token::OpAdd),
            b'-' => self.simple_token(Token::OpMinus),
            b'*' => self.simple_token(Token::OpMul),
            b'^' => self.simple_token(Token::OpPow),
            b'%' => self.simple_token(Token::OpMod),
            b'&' => self.simple_token(Token::OpBitAnd),
            b'|' => self.simple_token(Token::OpBitOr),
            b'#' => self.simple_token(Token::OpLen),
            b':' => {
                if self.test("::") {
                    self.next(2);
                    Ok(Token::SepLabel)
                } else {
                    self.simple_token(Token::SepColon)
                }
            }
            b'/' => {
                if self.test("//") {
                    self.next(2);
                    Ok(Token::OpIDiv)
                } else {
                    self.simple_token(Token::OpDiv)
                }
            }
            b'~' => {
                if self.test("~=") {
                    self.next(2);
                    Ok(Token::OpNe)
                } else {
                    self.simple_token(Token::OpWave)
                }
            }
            b'=' => {
                if self.test("==") {
                    self.next(2);
                    Ok(Token::OPEq)
                } else {
                    self.simple_token(Token::OpAssign)
                }
            }
            b'<' => {
                if self.test("<<") {
                    self.next(2);
                    Ok(Token::OpShl)
                } else if self.test("<=") {
                    self.next(2);
                    Ok(Token::OpLe)
                } else {
                    self.simple_token(Token::OpLt)
                }
            }
            b'>' => {
                if self.test(">>") {
                    self.next(2);
                    Ok(Token::OpShr)
                } else if self.test(">=") {
                    self.next(2);
                    Ok(Token::OpGe)
                } else {
                    self.simple_token(Token::OpGt)
                }
            }
            b'.' if self.test("...") => {
                self.next(3);
                Ok(Token::VarArg)
            }
            b'.' if self.test("..") => {
                self.next(2);
                Ok(Token::OpConcat)
            }
            b'.' if self.index + 1 == self.chunk.len()
                || !self.chunk[self.index + 1].is_ascii_digit() =>
            {
                self.simple_token(Token::SepDot)
            }
            b'[' => {
                if self.test("[[") || self.test("[=") {
                    Ok(Token::String(self.scan_long_string()?))
                } else {
                    self.simple_token(Token::SepLbrack)
                }
            }
            b'\'' | b'"' => Ok(Token::String(self.scan_short_string()?)),

            _ => {
                if ch == b'.' || ch.is_ascii_digit() {
                    Ok(Token::Number(self.scan_number()?))
                } else if ch == b'_' || ch.is_ascii_alphabetic() {
                    let s = self.scan_identifier()?;
                    match keywords.get(s.as_str()) {
                        None => Ok(Token::Identifier(s)),
                        Some(tok) => Ok(tok.clone()),
                    }
                } else {
                    Err(Error::IllegalToken)
                }
            }
        }
    }

    /// 返回 identifier string，若不是则Err
    pub fn next_ident(&mut self) -> Result<String> {
        let tok = self.next_token();
        match tok {
            Ok(Token::Identifier(s)) => Ok(s),
            _ => Err(Error::NotIdentifier),
        }
    }

    pub fn check_next_token(&mut self, tok: Token) -> bool {
        match self.next_token() {
            Ok(ref token) if tok == *token => true,
            _ => false,
        }
    }

    /// 转移字符串
    fn escape_string(&self, s: &[u8]) -> Result<String> {
        // todo: 转义字符串
        let mut ret: Vec<u8> = vec![];
        let mut i = 0;
        while i < s.len() {
            if s[i] != b'\\' {
                ret.push(s[i]);
                i += 1;
                continue;
            }

            if i + 1 == s.len() {
                return Err(Error::IllegalEscape);
            } else {
                match s[i + 1] {
                    b'a' => {
                        ret.push(0x07u8);
                        i += 2;
                    }
                    b'b' => {
                        ret.push(0x08u8);
                        i += 2;
                    }
                    b'f' => {
                        ret.push(0x0cu8);
                        i += 2;
                    }
                    b'n' | b'\n' => {
                        ret.push(b'\n');
                        i += 2;
                    }
                    b'r' => {
                        ret.push(b'\r');
                        i += 2;
                    }
                    b't' => {
                        ret.push(b'\t');
                        i += 2;
                    }
                    b'v' => {
                        ret.push(0x0bu8);
                        i += 2;
                    }
                    ch @ b'"' | ch @ b'\'' | ch @ b'\\' => {
                        ret.push(ch);
                        i += 2;
                    }
                    // todo: fix it
                    // \ddd
                    ch if ch.is_ascii_digit() => {
                        let num = re_dec_escaped_seq
                            .find(&s[i + 1..])
                            .ok_or(Error::IllegalEscape)?
                            .as_bytes();
                        let num = str::from_utf8(num).or(Err(Error::IllegalEscape))?;
                        let len = num.len() + 1;
                        let num = num.parse::<u8>().or(Err(Error::IllegalEscape))?;
                        ret.push(num);
                        i += len;
                    }
                    // todo: fix it
                    // \xXX
                    b'x' => {
                        let num = re_hex_escaped_seq
                            .find(&s[i + 2..])
                            .ok_or(Error::IllegalEscape)?
                            .as_bytes();
                        let num = str::from_utf8(num).or(Err(Error::IllegalEscape))?;
                        let num = num.parse::<u8>().or(Err(Error::IllegalEscape))?;
                        ret.push(num);
                        i += 3;
                    }
                    // \u{XXX}
                    b'u' => {
                        let num = re_unicode_escaped_seq
                            .find(&s[i + 3..])
                            .ok_or(Error::IllegalEscape)?
                            .as_bytes();
                        let num = str::from_utf8(num).or(Err(Error::IllegalEscape))?;
                        let len = num.len();
                        let num = num.parse::<u8>().or(Err(Error::IllegalEscape))?;
                        ret.push(num);
                        i += len;
                    }
                    // \z
                    b'z' => {}
                    _ => {
                        i += 1;
                    }
                };
            }
        }

        unsafe { Ok(String::from_utf8_unchecked(ret)) }
    }

    /// 扫描长字符串
    fn scan_long_string(&mut self) -> Result<String> {
        // long comment: -- [===[ ... ]===]
        let text = &self.chunk[self.index..];
        let caps = &re_long_bracket.captures(text).ok_or(Error::IllegalToken)?;
        // todo: trim string
        self.index += caps["comment"].len();
        unsafe { Ok(String::from_utf8_unchecked(caps["string"].to_vec())) }
    }

    /// 扫描短字符串
    fn scan_short_string(&mut self) -> Result<String> {
        // todo: escape
        let text = &self.chunk[self.index..];
        let s = re_short_str
            .find(text)
            .ok_or(Error::IllegalToken)?
            .as_bytes();
        self.index += s.len();
        let s = &s[1..s.len() - 1];
        self.escape_string(s)
    }

    /// 扫描数字
    fn scan_number(&mut self) -> Result<String> {
        use std::str;

        let text = &self.chunk[self.index..];
        let s = re_number.find(text).ok_or(Error::IllegalToken)?.as_bytes();
        self.index += s.len();
        unsafe { Ok(String::from_utf8_unchecked(s.to_vec())) }
    }

    /// 扫描标识符
    fn scan_identifier(&mut self) -> Result<String> {
        let text = &self.chunk[self.index..];
        let s = re_ident.find(text).ok_or(Error::IllegalToken)?.as_bytes();
        self.index += s.len();
        unsafe { Ok(String::from_utf8_unchecked(s.to_vec())) }
    }

    /// 跳过空白符(总是跳过注释)
    fn skip_whitespaces(&mut self) -> Result<()> {
        while let Some(ch) = self.current() {
            if self.test("--") {
                self.skip_comment()?;
            } else if self.test("\r\n") || self.test("\n\r") {
                self.next(2);
                self.line += 1;
            } else if is_new_line(ch) {
                self.next(1);
                self.line += 1;
            } else if ch.is_ascii_whitespace() {
                self.next(1);
            } else {
                break;
            }
        }
        Ok(())
    }

    /// 判断当前源码是否以一串字符串开头
    fn test(&self, s: &str) -> bool {
        for (i, ch) in s.bytes().enumerate() {
            if self.is_eof() {
                return false;
            } else if self.chunk[self.index + i] != ch {
                return false;
            }
        }
        return true;
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.chunk.len() <= self.index
    }

    /// 跳过n个字符
    #[inline]
    fn next(&mut self, n: usize) {
        self.index += n;
    }

    /// 返回当前字符
    #[inline]
    fn current(&self) -> Option<u8> {
        if self.is_eof() {
            None
        } else {
            Some(self.chunk[self.index])
        }
    }

    /// 跳过注释
    fn skip_comment(&mut self) -> Result<()> {
        self.next(2);
        // long comment: --[[ ...... --]]
        match self.current() {
            Some(b'[') => {
                self.scan_long_string()?;
                return Ok(());
            }
            _ => {}
        }

        // short comment: --
        while let Some(ch) = self.current() {
            self.next(1);
            if is_new_line(ch) {
                break;
            }
        }

        Ok(())
    }
}

/// 判断是否开始新一行
#[inline]
fn is_new_line(c: u8) -> bool {
    c == b'\r' || c == b'\n'
}

/// 判断字符是否符合16进制
#[inline]
fn is_hexadecimal(c: u8) -> bool {
    (b'0' <= c && c <= b'9') || (b'a' <= c && c <= b'f') || (b'A' <= c && c <= b'F')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let s = r##"
            +
            -
            >>
            ==
            [==[ 世界 ]=]
            'string'
            "string"
            12.34E-56
            0x12.abp-10
            break
            name
        "##
        .to_string();

        let mut lexer = Lexer::from_iter(s.bytes(), "test".to_string());

        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::OpAdd);
        assert_eq!(lexer.current_line(), 2);

        let res = lexer.look_ahead();
        assert_eq!(res.unwrap(), Token::OpMinus);
        assert_eq!(lexer.current_line(), 2);

        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::OpMinus);
        assert_eq!(lexer.current_line(), 3);

        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::OpShr);
        assert_eq!(lexer.current_line(), 4);

        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::OPEq);
        assert_eq!(lexer.current_line(), 5);

        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::String(" 世界 ".to_string()));
        assert_eq!(lexer.current_line(), 6);

        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::String("string".to_string()));
        assert_eq!(lexer.current_line(), 7);

        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::String("string".to_string()));
        assert_eq!(lexer.current_line(), 8);

        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::Number("12.34E-56".to_string()));
        assert_eq!(lexer.current_line(), 9);

        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::Number("0x12.abp-10".to_string()));
        assert_eq!(lexer.current_line(), 10);

        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::KwBreak);
        assert_eq!(lexer.current_line(), 11);

        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::Identifier("name".to_string()));
        assert_eq!(lexer.current_line(), 12);

        assert_eq!(lexer.next_token(), Err(Error::EOF));
        assert_eq!(lexer.current_line(), 13);
    }
}
