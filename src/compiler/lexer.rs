use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::result;

use crate::compiler::error::Error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Lexer {
    /// 源码
    chunk: Vec<u8>,
    /// 当前位置
    index: usize,
    /// 源文件名
    chunk_name: String,
    /// 当前位置
    line: usize,
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

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// ...
    Vararg,
    /// ;
    SepSemi,
    /// ,
    SepComma,
    /// .
    SepDot,
    /// :
    SepColon,
    /// ::
    SepLabel,
    /// (
    SepLparen,
    /// )
    SepRparen,
    /// [
    SepLbrack,
    /// ]
    SepRbrack,
    /// {
    SepLcurly,
    /// }
    SepRcurly,
    /// =
    OpAssign,
    /// - (sub or unm)
    OpMinus,
    /// ~ (bnot or bxor)
    OpWave,
    /// +
    OpAdd,
    /// *
    OpMul,
    /// /
    OpDiv,
    /// //
    OpIDiv,
    /// ^
    OpPow,
    /// %
    OpMod,
    /// &
    OpBitAnd,
    /// |
    OpBitOr,
    /// >>
    OpShr,
    /// <<
    OpShl,
    /// ..
    OpConcat,
    /// <
    OpLt,
    /// <=
    OpLe,
    /// >
    OpGt,
    /// >=
    OpGe,
    /// ==
    OPEq,
    /// ~=
    OpNe,
    /// #
    OpLen,
    /// and
    OpAnd,
    /// or
    OpOr,
    /// not
    OpNot,
    /// break
    KwBreak,
    /// do
    KwDo,
    /// else
    KwElse,
    /// elseif
    KwElseIf,
    /// end
    KwEnd,
    /// false
    KwFalse,
    /// for
    KwFor,
    /// function
    KwFunction,
    /// goto
    KwGoto,
    /// if
    KwIf,
    /// in
    KwIn,
    /// local
    KwLocal,
    /// nil
    KwNil,
    /// repeat
    KwRepeat,
    /// return
    KwReturn,
    /// then
    KwThen,
    /// true
    KwTrue,
    /// until
    KwUntil,
    /// while
    KwWhile,
    /// `id`
    Identifier(String),
    /// `number`
    Number(String),
    /// `string`
    String(String),
}

lazy_static! {
    static ref keywords: HashMap<&'static str, Token> = {
        let mut m = HashMap::new();
        m.insert("and", Token::OpAnd);
        m.insert("break", Token::KwBreak);
        m
    };
}

impl Lexer {
    /// 创建词法分析
    #[inline]
    pub fn new(chunk: String, chunk_name: String) -> Self {
        Self {
            chunk: chunk.into_bytes(),
            index: 0,
            chunk_name,
            line: 1,
        }
    }
    /// 返回下一个token
    pub fn next_token<'a>(&mut self) -> Result<Token> {
        self.skip_whitespaces();
        let ch = self.current()?;
        let tok = match ch {
            b';' => {
                self.next(1);
                Token::SepSemi
            }
            b',' => {
                self.next(1);
                Token::SepComma
            }
            b'(' => {
                self.next(1);
                Token::SepLparen
            }
            b')' => {
                self.next(1);
                Token::SepRparen
            }
            b']' => {
                self.next(1);
                Token::SepRbrack
            }
            b'{' => {
                self.next(1);
                Token::SepLcurly
            }
            b'}' => {
                self.next(1);
                Token::SepRcurly
            }
            b'+' => {
                self.next(1);
                Token::OpAdd
            }
            b'-' => {
                self.next(1);
                Token::OpMinus
            }
            b'*' => {
                self.next(1);
                Token::OpMinus
            }
            b'^' => {
                self.next(1);
                Token::OpPow
            }
            b'%' => {
                self.next(1);
                Token::OpMod
            }
            b'&' => {
                self.next(1);
                Token::OpBitAnd
            }
            b'|' => {
                self.next(1);
                Token::OpBitOr
            }
            b'#' => {
                self.next(1);
                Token::OpLen
            }
            b':' => {
                if self.test("::") {
                    self.next(2);
                    Token::SepLabel
                } else {
                    self.next(1);
                    Token::SepColon
                }
            }
            b'/' => {
                if self.test("//") {
                    self.next(2);
                    Token::OpIDiv
                } else {
                    self.next(1);
                    Token::OpDiv
                }
            }
            b'~' => {
                if self.test("~=") {
                    self.next(2);
                    Token::OpNe
                } else {
                    self.next(1);
                    Token::OpWave
                }
            }
            b'=' => {
                if self.test("==") {
                    self.next(2);
                    Token::OPEq
                } else {
                    self.next(1);
                    Token::OpAssign
                }
            }
            b'<' => {
                if self.test("<<") {
                    self.next(2);
                    Token::OpShl
                } else if self.test("<=") {
                    self.next(2);
                    Token::OpLe
                } else {
                    self.next(1);
                    Token::OpLt
                }
            }
            b'>' => {
                if self.test(">>") {
                    self.next(2);
                    Token::OpShr
                } else if self.test(">=") {
                    self.next(2);
                    Token::OpGe
                } else {
                    self.next(1);
                    Token::OpGt
                }
            }
            b'.' if self.test("...") => {
                self.next(3);
                Token::Vararg
            }
            b'.' if self.test("..") => {
                self.next(2);
                Token::OpConcat
            }
            b'.' if self.index + 1 == self.chunk.len()
                || !self.chunk[self.index + 1].is_ascii_digit() =>
            {
                self.next(1);
                Token::SepDot
            }

            b'[' => {
                if self.test("[[") || self.test("[=") {
                    Token::String(self.scan_long_string())
                } else {
                    self.next(1);
                    Token::SepLbrack
                }
            }
            b'\'' | b'"' => Token::String(self.scan_short_string()),
            _ => {
                if ch == b'.' || ch.is_ascii_digit() {
                    Token::Number(self.scan_number())
                } else if ch == b'_' || ch.is_ascii_alphabetic() {
                    Token::Identifier(self.scan_identifier())
                } else {
                    unreachable!()
                }
            }
        };

        Ok(tok)
    }

    /// 扫描长字符串
    fn scan_long_string(&mut self) -> String {
        unimplemented!()
    }

    /// 扫描短字符串
    fn scan_short_string(&mut self) -> String {
        unimplemented!()
    }

    /// 扫描数字
    fn scan_number(&mut self) -> String {
        unimplemented!()
    }

    /// 扫描标识符
    fn scan_identifier(&mut self) -> String {
        unimplemented!()
    }

    /// 跳过空白符(总是跳过注释)
    fn skip_whitespaces(&mut self) {
        while let Ok(ch) = self.current() {
            if self.test("--") {
                self.skip_comment();
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
    }

    /// 判断当前源码是否以一串字符串开头
    fn test(&self, s: &str) -> bool {
        for (i, ch) in s.bytes().enumerate() {
            if self.chunk[self.index + i] != ch {
                return false;
            }
        }
        return true;
    }

    /// 跳过n个字符
    #[inline]
    fn next(&mut self, n: usize) -> Result<()> {
        self.index += n;
        if self.index < self.chunk.len() {
            Ok(())
        } else {
            Err(Error::EOF)
        }
    }

    /// 返回当前字符
    #[inline]
    fn current(&self) -> Result<u8> {
        if self.chunk.len() > self.index {
            Ok(self.chunk[self.index])
        } else {
            Err(Error::EOF)
        }
    }

    /// 跳过注释
    fn skip_comment(&mut self) {
        self.next(2);
        // long comment: --[[ ...... --]]
        match self.current() {
            Ok(b'[') => unimplemented!(),
            _ => {}
        }

        while let Ok(ch) = self.current() {
            self.next(1);
            if is_new_line(ch) {
                break;
            }
        }
    }
}

/// 判断是否新一行
fn is_new_line(c: u8) -> bool {
    c == b'\r' || c == b'\n'
}

/// 判断字符是否符合16进制
fn is_hexadecimal(c: u8) -> bool {
    (b'0' <= c && c <= b'9') || (b'a' <= c && c <= b'f') || (b'A' <= c && c <= b'F')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let s = r#"
            +
            -
            break
        "#
        .to_string();
        let mut lexer = Lexer::new(s, "test".to_string());
        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::OpAdd);
        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::OpMinus);
    }
}
