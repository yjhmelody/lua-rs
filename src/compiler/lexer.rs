use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::result;

use crate::compiler::error::Error;

/// 包装编译信息
pub type Result<T> = result::Result<T, Error>;

// 代码原位置，用于代码生成的信息
pub type Line = usize;

#[derive(Debug)]
pub struct Lexer {
    /// 源码
    chunk: Vec<u8>,
    /// 当前位置
    index: usize,
    /// 源文件名
    chunk_name: String,
    /// 当前位置
    line: Line,
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

    /// 返回当前单个字符的token
    fn simple_token(&mut self, token: Token) -> Result<Token> {
        self.next(1)?;
        Ok(token)
    }
    /// 返回下一个token
    pub fn next_token<'a>(&mut self) -> Result<Token> {
        self.skip_whitespaces();
        let ch = self.current()?;
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
                Ok(Token::Vararg)
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
                    Ok(Token::String(self.scan_long_string()))
                } else {
                    self.simple_token(Token::SepLbrack)
                }
            }
            b'\'' | b'"' => Ok(Token::String(self.scan_short_string())),
            _ => {
                if ch == b'.' || ch.is_ascii_digit() {
                    Ok(Token::Number(self.scan_number()))
                } else if ch == b'_' || ch.is_ascii_alphabetic() {
                    Ok(Token::Identifier(self.scan_identifier()))
                } else {
                    unreachable!()
                }
            }
        }
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

/// 判断是否开始新一行
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
            >>
            ==
        "#
        .to_string();
        let mut lexer = Lexer::new(s, "test".to_string());

        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::OpAdd);
        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::OpMinus);
        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::OpShr);
        let res = lexer.next_token();
        assert_eq!(res.unwrap(), Token::OPEq);

        assert_eq!(lexer.next_token().is_err(), true);
    }
}
