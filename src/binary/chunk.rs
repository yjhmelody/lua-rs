#![allow(dead_code)]

use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// "\x1bLua"
pub const LUA_SIGNATURE: [u8; 4] = [0x1b, 0x4c, 0x75, 0x61];
pub const LUAC_VERSION: u8 = 0x53;
pub const LUAC_FORMAT: u8 = 0;
/// "\x19\x93\r\n\x1a\n"
pub const LUAC_DATA: [u8; 6] = [0x19, 0x93, 0x0d, 0x0a, 0x1a, 0x0a];
pub const CINT_SIZE: u8 = 4;
pub const CSIZET_SIZE: u8 = 8;
pub const INSTRUCTION_SIZE: u8 = 4;
pub const LUA_INTEGER_SIZE: u8 = 8;
pub const LUA_NUMBER_SIZE: u8 = 8;
pub const LUAC_INT: i64 = 0x5678;
pub const LUAC_NUM: f64 = 370.5;

pub const TAG_NIL: u8 = 0x00;
pub const TAG_BOOLEAN: u8 = 0x01;
pub const TAG_NUMBER: u8 = 0x03;
pub const TAG_INTEGER: u8 = 0x13;
pub const TAG_SHORT_STR: u8 = 0x04;
pub const TAG_LONG_STR: u8 = 0x14;


/// Lua Binary Chunk
#[derive(Debug)]
struct BinaryChunk {
    header: Header,
    size_up_values: u8,
    main_func: Prototype,
}

/// Lua Header
#[derive(Debug)]
struct Header {
    signature: [u8; 4],
    version: u8,
    format: u8,
    luac_data: [u8; 6],
    c_int_size: u8,
    c_size_t_size: u8,
    instruction_size: u8,
    lua_integer_size: u8,
    lua_number_size: u8,
    luac_int: i64,
    luac_num: f64,
}

/// Lua Function Prototype
#[derive(Debug)]
pub struct Prototype {
    pub source: Option<String>,
    /// For debug
    pub line_defined: u32,
    pub last_line_defined: u32,
    pub num_params: u8,
    pub is_vararg: u8,
    pub max_stack_size: u8,
    pub code: Vec<u32>,
    pub constants: Vec<Constant>,
    pub up_values: Vec<UpValue>,
    pub prototypes: Vec<Rc<Prototype>>,
    pub line_info: Vec<u32>,
    /// For debug
    pub local_vars: Vec<LocalVar>,
    /// For debug
    pub up_value_names: Vec<String>, // debug
}

/// Lua Up Value
#[derive(Debug, Copy, Clone)]
pub struct UpValue {
    pub instack: u8,
    pub idx: u8,
}

impl Default for UpValue {
    fn default() -> Self {
        Self {
            instack: 0,
            idx: 0,
        }
    }
}

impl UpValue {
    pub fn new(instack: u8, idx: u8) -> Self {
        Self {
            instack,
            idx,
        }
    }
}

#[derive(Debug)]
pub struct LocalVar {
    pub var_name: String,
    pub start_pc: u32,
    pub end_pc: u32,
}

/// Constant can be stored in constant pool
#[derive(Debug, Clone)]
pub enum Constant {
    Nil,
    Boolean(bool),
    Number(f64),
    Integer(i64),
    String(String),
}

impl Hash for Constant {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Constant::Nil => 0.hash(state),
            Constant::Boolean(b) => b.hash(state),
            Constant::Number(n) => n.to_bits().hash(state),
            Constant::Integer(i) => i.hash(state),
            Constant::String(s) => s.hash(state),
        }
    }
}

impl PartialEq for Constant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Constant::Nil, Constant::Nil) => true,
            (Constant::Boolean(a), Constant::Boolean(b)) if a == b => true,
            (Constant::Number(a), Constant::Number(b)) if a == b => true,
            (Constant::Integer(a), Constant::Integer(b)) if a == b => true,
            (Constant::String(s1), Constant::String(s2)) if s1 == s2 => true,
            // todo: cmp f64 and i64
            _ => false,
        }
    }
}

impl Eq for Constant {}