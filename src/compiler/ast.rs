#![allow(dead_code)]

use crate::compiler::lexer::Line;
use crate::compiler::token::Token;

/// A Lua chunk also is a Lua block
#[derive(Debug)]
pub struct Block {
    pub stats: Vec<Stat>,
    pub ret_exps: Vec<Exp>,
    pub last_line: Line,
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
    Break(Line),
    Label(String),
    Goto(String),
    Do(Box<Block>),
    While(Exp, Box<Block>),
    Repeat(Exp, Box<Block>),
    /// exps stores conditions. compile `else` to `elseif true`
    Condition(Vec<Exp>, Vec<Block>),
    /* line of for, line of do */
    ForNum(Box<ForNum>, Line, Line),
    /* line of do */
    ForIn(Box<ForIn>, Line),
    /* last_line */
    LocalVarDecl(Vec<String>, Vec<Exp>, Line),
    LocalFnDef(String, Exp),
    /* last line */
    Assign(Vec<Exp>, Vec<Exp>, Line),
    /// function call is either expression or statement
    FnCall(FnCall, Line, Line),
}

/// Lua expression
#[derive(Debug)]
pub enum Exp {
    Nil(Line),
    True(Line),
    False(Line),
    Vararg(Line),
    Integer(i64, Line),
    String(String, Line),
    Name(String, Line),
    Parens(Box<Exp>),
    Unop(Token, Box<Exp>, Line),
    Binop(Box<Exp>, Token, Box<Exp>, Line),
    /// right-association, parse it to multi-node tree
    Concat(Vec<Exp>, Line),
    // last line
    TableConstructor(Vec<Field>, Line),
    /// (Object, Key)
    TableAccess(Box<Exp>, Box<Exp>, Line),
    FnDef(ParList, Box<Block>, Line, Line),
    FnCall(FnCall, Line, Line),
}


#[derive(Debug)]
pub struct ForNum {
    pub name: String,
    pub init: Exp,
    pub limit: Exp,
    pub step: Exp,
    pub block: Box<Block>,
}

impl ForNum {
    pub fn new(name: String, init: Exp, limit: Exp, step: Exp, block: Box<Block>) -> Box<Self> {
        Box::new(Self {
            name,
            init,
            limit,
            step,
            block,
        })
    }
}

#[derive(Debug)]
pub struct ForIn {
    pub name_list: Vec<String>,
    pub exp_list: Vec<Exp>,
    pub block: Box<Block>,
}

impl ForIn {
    pub fn new(name_list: Vec<String>, exp_list: Vec<Exp>, block: Box<Block>) -> Box<Self> {
        Box::new(Self {
            name_list,
            exp_list,
            block,
        })
    }
}

#[derive(Debug)]
pub struct Field {
    pub key: Option<Exp>,
    pub val: Exp,
}

impl Field {
    pub fn new(key: Option<Exp>, val: Exp) -> Self { Self { key, val } }
}

#[derive(Debug)]
pub struct FnCall {
    pub prefix: Box<Exp>,
    pub name: Option<Box<Exp>>,
    pub args: Vec<Exp>,
}

impl FnCall {
    pub fn new(prefix: Box<Exp>, name: Option<Box<Exp>>, args: Vec<Exp>) -> Self {
        Self {
            prefix,
            name,
            args,
        }
    }
}


#[derive(Debug)]
pub struct ParList {
    pub params: Vec<String>,
    pub is_vararg: bool,
}

impl ParList {
    pub fn new(params: Vec<String>, is_vararg: bool) -> Self {
        Self {
            is_vararg,
            params,
        }
    }

    pub fn set_vararg(&mut self, vararg: bool) {
        self.is_vararg = vararg;
    }

    pub fn set_params(&mut self, params: Vec<String>) {
        self.params = params;
    }

    pub fn push_param(&mut self, param: String) {
        self.params.push(param)
    }
}
