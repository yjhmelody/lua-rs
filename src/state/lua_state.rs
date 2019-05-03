use crate::api::consts::*;
use crate::api::LuaAPI;
use crate::state::lua_stack::LuaStack;
use crate::state::lua_value::LuaValue;

/// Lua State containing Lua Stack
pub struct LuaState {
    stack: LuaStack,
}

impl LuaState {
    pub fn new() -> LuaState {
        LuaState {
            stack: LuaStack::new(20),
        }
    }
}

impl LuaAPI for LuaState {
    /* basic stack manipulation */

    fn get_top(&self) -> isize {
        self.stack.top()
    }

    fn abs_index(&self, idx: isize) -> isize {
        self.stack.abs_index(idx)
    }

    fn check_stack(&mut self, n: usize) -> bool {
        self.stack.check(n);
        true
    }

    fn pop(&mut self, n: usize) {
        for _ in 0..n {
            self.stack.pop();
        }
    }

    fn copy(&mut self, from_idx: isize, to_idx: isize) {
        let val = self.stack.get(from_idx);
        self.stack.set(to_idx, val);
    }

    fn push_value(&mut self, idx: isize) {
        let val = self.stack.get(idx);
        self.stack.push(val);
    }

    fn replace(&mut self, idx: isize) {
        let val = self.stack.pop();
        self.stack.set(idx, val);
    }

    fn insert(&mut self, idx: isize) {
        self.rotate(idx, 1);
    }

    fn remove(&mut self, idx: isize) {
        self.rotate(idx, -1);
        self.pop(1);
    }

    fn rotate(&mut self, idx: isize, n: isize) {
        let abs_idx = self.stack.abs_index(idx);
        if abs_idx < 0 || !self.stack.is_valid(abs_idx) {
            panic!("invalid index!");
        }

        let t = self.stack.top() - 1; /* end of stack segment being rotated */
        let p = abs_idx - 1; /* start of segment */
        let m = if n >= 0 { t - n } else { p - n - 1 }; /* end of prefix */
        self.stack.reverse(p as usize, m as usize); /* reverse the prefix with length 'n' */
        self.stack.reverse(m as usize + 1, t as usize); /* reverse the suffix */
        self.stack.reverse(p as usize, t as usize); /* reverse the entire segment */
    }

    fn set_top(&mut self, idx: isize) {
        let new_top = self.stack.abs_index(idx);
        if new_top < 0 {
            panic!("stack underflow!");
        }

        let n = self.stack.top() - new_top;
        if n > 0 {
            for _ in 0..n {
                self.stack.pop();
            }
        } else if n < 0 {
            for _ in n..0 {
                self.stack.push(LuaValue::Nil);
            }
        }
    }

    /* access functions (stack -> rust) */

    fn type_name(&self, tp: i8) -> &str {
        match tp {
            LUA_TNONE => "no value",
            LUA_TNIL => "nil",
            LUA_TBOOLEAN => "boolean",
            LUA_TNUMBER => "number",
            LUA_TSTRING => "string",
            LUA_TTABLE => "table",
            LUA_TFUNCTION => "function",
            LUA_TTHREAD => "thread",
            LUA_TLIGHTUSERDATA => "userdata",
            LUA_TUSERDATA => "userdata",
            _ => "?", // TODO
        }
    }

    #[inline]
    fn type_id(&self, idx: isize) -> i8 {
        if self.stack.is_valid(idx) {
            self.stack.get(idx).type_id()
        } else {
            LUA_TNONE
        }
    }

    #[inline]
    fn is_none(&self, idx: isize) -> bool {
        self.type_id(idx) == LUA_TNONE
    }

    #[inline]
    fn is_nil(&self, idx: isize) -> bool {
        self.type_id(idx) == LUA_TNIL
    }

    #[inline]
    fn is_none_or_nil(&self, idx: isize) -> bool {
        self.type_id(idx) <= LUA_TNIL
    }

    #[inline]
    fn is_boolean(&self, idx: isize) -> bool {
        self.type_id(idx) == LUA_TBOOLEAN
    }

    #[inline]
    fn is_table(&self, idx: isize) -> bool {
        self.type_id(idx) == LUA_TTABLE
    }

    #[inline]
    fn is_function(&self, idx: isize) -> bool {
        self.type_id(idx) == LUA_TFUNCTION
    }

    #[inline]
    fn is_thread(&self, idx: isize) -> bool {
        self.type_id(idx) == LUA_TTHREAD
    }

    #[inline]
    fn is_string(&self, idx: isize) -> bool {
        let t = self.type_id(idx);
        t == LUA_TSTRING || t == LUA_TNUMBER
    }

    #[inline]
    fn is_number(&self, idx: isize) -> bool {
        self.to_numberx(idx).is_some()
    }

    #[inline]
    fn is_integer(&self, idx: isize) -> bool {
        match self.stack.get(idx) {
            LuaValue::Integer(_) => true,
            _ => false,
        }
    }

    #[inline]
    fn to_boolean(&self, idx: isize) -> bool {
        self.stack.get(idx).to_boolean()
    }

    #[inline]
    fn to_integer(&self, idx: isize) -> i64 {
        self.to_integerx(idx).unwrap_or_default()
    }

    #[inline]
    fn to_integerx(&self, idx: isize) -> Option<i64> {
        match self.stack.get(idx) {
            LuaValue::Integer(i) => Some(i),
            _ => None,
        }
    }

    #[inline]
    fn to_number(&self, idx: isize) -> f64 {
        self.to_numberx(idx).unwrap_or_default()
    }

    #[inline]
    fn to_numberx(&self, idx: isize) -> Option<f64> {
        match self.stack.get(idx) {
            LuaValue::Number(n) => Some(n),
            LuaValue::Integer(i) => Some(i as f64),
            _ => None,
        }
    }

    #[inline]
    fn to_string(&self, idx: isize) -> String {
        self.to_stringx(idx).unwrap_or_default()
    }

    #[inline]
    fn to_stringx(&self, idx: isize) -> Option<String> {
        match self.stack.get(idx) {
            LuaValue::String(s) => Some(s),
            LuaValue::Number(n) => Some(n.to_string()),
            LuaValue::Integer(i) => Some(i.to_string()),
            _ => None,
        }
    }

    /* push functions (rust -> stack) */

    #[inline]
    fn push_nil(&mut self) {
        self.stack.push(LuaValue::Nil);
    }

    #[inline]
    fn push_boolean(&mut self, b: bool) {
        self.stack.push(LuaValue::Boolean(b));
    }

    #[inline]
    fn push_integer(&mut self, n: i64) {
        self.stack.push(LuaValue::Integer(n));
    }

    #[inline]
    fn push_number(&mut self, n: f64) {
        self.stack.push(LuaValue::Number(n));
    }

    #[inline]
    fn push_string(&mut self, s: String) {
        self.stack.push(LuaValue::String(s));
    }
}
