use std::result;

// todo: 传递错误位置信息
/// 编译期间产生的错误
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    /// Need more bytes
    EOF,
    /// Illegal Token
    IllegalToken,
    /// Cannot be Escaped
    IllegalEscape,
    /// Need more Tokens
    NoMoreTokens,
    /// Illegal Expression
    IllegalExpression,
    /// Illegal Expression
    IllegalStat,
    /// Not a Identifier
    NotIdentifier,
    /// Not a var expression
    NotVarExpression,
    /// Not a operator
    NotOperator,
    /// Illegal function params
    IllegalFunction,
}

/// 包装编译错误信息
pub type Result<T> = result::Result<T, Error>;
