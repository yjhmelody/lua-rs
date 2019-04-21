use std::result;

// todo: better error reports
/// Some Errors produced by parser and lexer which be dealt by parser for reporting syntax errors
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
    /// Brackets do not match
    NotMatchBrackets,
    /// Missing assignment
    MissingAssignment,
    /// Illegal Function call
    IllegalFnCall,
}

/// Wrapped for parsing time errors
pub type Result<T> = result::Result<T, Error>;
