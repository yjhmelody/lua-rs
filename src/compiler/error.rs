use std::fmt::{self, Display, Formatter};
use std::result;

use crate::compiler::lexer::Line;

/// Wrapped for parsing time errors
pub type Result<T> = result::Result<T, Error>;

/// Some Errors produced by parser and lexer which be dealt by parser for reporting syntax errors
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    /// Need more bytes
    EOF { line: Line },
    /// Illegal Identifier
    IllegalIdentifier { line: Line },
    /// Illegal Number Literal
    IllegalNumLiteral { line: Line },
    /// Illegal String
    IllegalString { line: Line },
    /// Illegal Token
    IllegalToken { line: Line },
    /// Cannot be Escaped
    IllegalEscape { line: Line },
    /// Need more Tokens
    NoMoreTokens { line: Line },
    /// Illegal Expression
    IllegalExpression { line: Line },
    /// Illegal Expression
    IllegalStat { line: Line },
    /// Not a Identifier
    NotIdentifier { line: Line },
    /// Not a var expression
    NotVarExpression { line: Line },
    /// Not a operator
    NotOperator { line: Line },
    /// Illegal function params
    IllegalFunction { line: Line },
    /// Brackets do not match
    NotMatchBrackets { line: Line },
    /// Missing assignment
    MissingAssignment { line: Line },
    /// Illegal Function call
    IllegalFnCall { line: Line },
    /// Illegal Function definition
    IllegalFnDef { line: Line },
    /// No more Registers
    NoMoreRegisters,
    /// Illegal Register
    IllegalRegister,
    /// No more Scopes
    NoMoreScopes,
    /// Not in a loop
    NoLoop { line: Line },
    /// Not a UpValue
    NotUpValue { line: Line },
    /// Not a vararg function
    NotVararg { line: Line },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Error::*;
        match self {
            EOF { line } => write!(f, "line: {}, error: EOF", *line),
            IllegalIdentifier { line } => write!(f, "line: {}, error: IllegalIdentifier", *line),
            IllegalNumLiteral { line } => write!(f, "line: {}, error: IllegalNumLiteral", *line),
            IllegalString { line } => write!(f, "line: {}, error: IllegalString", *line),
            IllegalToken { line } => write!(f, "line: {}, error: IllegalToken", *line),
            IllegalEscape { line } => write!(f, "line: {}, error: IllegalEscape", *line),
            NoMoreTokens { line } => write!(f, "line: {}, error: NoMoreTokens", *line),
            IllegalExpression { line } => write!(f, "line: {}, error: IllegalExpression", *line),
            IllegalStat { line } => write!(f, "line: {}, error: IllegalStat", *line),
            NotIdentifier { line } => write!(f, "line: {}, error: NotIdentifier", *line),
            NotVarExpression { line } => write!(f, "line: {}, error: NotVarExpression", *line),
            NotOperator { line } => write!(f, "line: {}, error: NotOperator", *line),
            IllegalFunction { line } => write!(f, "line: {}, error: IllegalFunction", *line),
            NotMatchBrackets { line } => write!(f, "line: {}, error: NotMatchBrackets", *line),
            MissingAssignment { line } => write!(f, "line: {}, error: MissingAssignment", *line),
            IllegalFnCall { line } => write!(f, "line: {}, error: IllegalFnCall", *line),
            IllegalFnDef { line } => write!(f, "line: {}, error: IllegalFnDef", *line),
            NoMoreRegisters => write!(f, "codegen error: NoMoreRegisters"),
            IllegalRegister => write!(f, "codegen error: IllegalRegister"),
            NoMoreScopes => write!(f, "codegen error: NoMoreScopes"),
            NoLoop { line } => write!(f, "line: {}, codegen error: NoLoop", *line),
            NotUpValue { line } => write!(f, "line: {}, codegen error: NotUpValue", *line),
            NotVararg { line } => write!(f, "line: {}, codegen error: NotVararg", *line),
        }
    }
}


