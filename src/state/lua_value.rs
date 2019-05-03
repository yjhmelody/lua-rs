use crate::api::consts::*;

/// Lua Basic Type Value
#[derive(Clone)]
pub enum LuaValue {
    Nil,
    Boolean(bool),
    Number(f64),
    Integer(i64),
    String(String),
}

impl LuaValue {
    pub fn type_id(&self) -> i8 {
        match self {
            LuaValue::Nil => LUA_TNIL,
            LuaValue::Boolean(_) => LUA_TBOOLEAN,
            LuaValue::Number(_) => LUA_TNUMBER,
            LuaValue::Integer(_) => LUA_TNUMBER,
            LuaValue::String(_) => LUA_TSTRING,
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            LuaValue::Nil => false,
            LuaValue::Boolean(b) => *b, // TODO
            _ => true,
        }
    }
}
