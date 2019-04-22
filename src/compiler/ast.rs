#![allow(dead_code)]

use crate::compiler::lexer::Line;
use crate::compiler::token::Token;

/// A Lua chunk also is a Lua block
#[derive(Debug)]
pub struct Block {
    stats: Vec<Stat>,
    ret_exps: Vec<Exp>,
    last_line: Line,
}

impl Block {
    /// Creates a Lua Block, which is also a Lua Chunk
    #[inline]
    pub fn new(stats: Vec<Stat>, ret_exps: Vec<Exp>, last_line: Line) -> Self {
        Self {
            stats,
            ret_exps,
            last_line,
        }
    }
}

/// Lua statement, including some Lua expression
#[derive(Debug)]
pub enum Stat {
    Empty,
    Break {
        line: Line,
    },
    Label {
        name: String,
    },
    Goto {
        name: String,
    },
    Do {
        block: Box<Block>,
    },
    While {
        exp: Exp,
        block: Box<Block>,
    },
    Repeat {
        exp: Exp,
        block: Box<Block>,
    },
    If {
        exps: Vec<Exp>,
        blocks: Vec<Block>,
    },
    ForNum {
        line_of_for: Line,
        line_of_do: Line,
        var_name: String,
        exps: (Exp, Exp, Exp),
        block: Box<Block>,
    },
    ForIn {
        line_of_do: Line,
        name_list: Vec<String>,
        exp_list: Vec<Exp>,
        block: Box<Block>,
    },
    LocalVarDecl {
        last_line: Line,
        name_list: Vec<String>,
        exp_list: Vec<Exp>,
    },
    Assign {
        last_line: Line,
        var_list: Vec<Exp>,
        exp_list: Vec<Exp>,
    },
    LocalFnDef {
        name: String,
        exp: Exp,
    },
    /// function call is either expression or statement
    FnCall {
        /// line of '('
        line: Line,
        // line of ')'
        last_line: Line,
        prefix_exp: Box<Exp>,
        name_exp: Option<Box<Exp>>,
        args: Vec<Exp>,
    }
}

/// Lua Expression
#[derive(Debug)]
pub enum Exp {
    Nil {
        line: Line,
    },
    True {
        line: Line,
    },
    False {
        line: Line,
    },
    Vararg {
        line: Line,
    },
    Integer {
        line: Line,
        val: i64,
    },
    Float {
        line: Line,
        val: f64,
    },
    String {
        line: Line,
        val: String,
    },
    Name {
        line: Line,
        val: String,
    },
    Parens(Box<Exp>),
    Unop {
        line: Line,
        op: Token,
        exp: Box<Exp>,
    },
    Binop {
        line: Line,
        op: Token,
        exp1: Box<Exp>,
        exp2: Box<Exp>,
    },
    /// right-association, parse it to multi-node tree
    Concat {
        line: Line,
        exps: Vec<Exp>,
    },
    TableConstructor {
        line: Line,
        last_line: Line,
        /// key can be omitted
        key_exps: Vec<Option<Exp>>,
        val_exps: Vec<Exp>,
    },
    TableAccess {
        /// line of ']'
        last_line: Line,
        prefix_exp: Box<Exp>,
        key_exp: Box<Exp>,
    },
    FnDef {
        line: Line,
        last_line: Line,
        par_list: Vec<String>,
        is_vararg: bool,
        block: Box<Block>,
    },
    FnCall {
        /// line of '('
        line: Line,
        /// line of ')'
        last_line: Line,
        prefix_exp: Box<Exp>,
        name_exp: Option<Box<Exp>>,
        args: Vec<Exp>,
    },
}
