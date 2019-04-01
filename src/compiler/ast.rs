#![allow(dead_code)]

use crate::compiler::lexer::Line;

/// A Lua chunk also is a Lua block
#[derive(Debug)]
pub struct Block {
    stats: Vec<Stat>,
    ret_exps: Vec<Exp>,
    last_line: Line,
}

impl Block {
    #[inline]
    pub fn new(stats: Vec<Stat>, ret_exps: Vec<Exp>, last_line: Line) -> Self {
        Self {
            stats,
            ret_exps,
            last_line,
        }
    }
}

/// Lua stat, including Lua expression
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
    Unop {
        line: Line,
        op: usize,
        exp: Box<Exp>,
    },
    Binop {
        line: Line,
        op: usize,
        exp1: Box<Exp>,
        exp2: Box<Exp>,
    },
    Concat {
        line: Line,
        exps: Vec<Exp>,
    },

    TableConstructor {
        line: Line,
        last_line: Line,
        key_exps: Vec<Exp>,
        val_exps: Vec<Exp>,
    },
    FnDef {
        line: Line,
        last_line: Line,
        par_list: Vec<String>,
        is_vararg: bool,
        block: Box<Block>,
    },
    Parens(Box<Exp>),
    TableAccess {
        last_line: Line, // line of ']'
        prefix_exp: Box<Exp>,
        key_exp: Box<Exp>,
    },
    FnCall {
        line: Line,      // line of '('
        last_line: Line, // line of ')'
        prefix_exp: Box<Exp>,
        name_exp: Box<Exp>,
        args: Vec<Exp>,
    },
}
