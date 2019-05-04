use std::result;

use crate::compiler::lexer::Line;

// todo: better error reports
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
    NoLoop,
    /// Not a UpValue
    NotUpValue,
    /// Not a vararg function
    NotVararg,
}

/// Wrapped for parsing time errors
pub type Result<T> = result::Result<T, Error>;
