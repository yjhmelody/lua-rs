use std::result;

/// 编译期间产生的错误
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    /// No more bytes
    EOF,
    /// Illegal Token
    IllegalToken,
    /// cannot be Escaped
    IllegalEscape,
}

/// 包装编译错误信息
pub type Result<T> = result::Result<T, Error>;