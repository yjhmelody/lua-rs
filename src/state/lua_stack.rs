use crate::state::lua_value::LuaValue;

/// Lua Stack
pub struct LuaStack {
    vec: Vec<LuaValue>,
}

impl LuaStack {
    #[inline]
    pub fn new(size: usize) -> LuaStack {
        LuaStack {
            vec: Vec::with_capacity(size),
        }
    }

    #[inline]
    pub fn top(&self) -> isize {
        self.vec.len() as isize
    }

    #[inline]
    pub fn check(&mut self, n: usize) {
        self.vec.reserve(n);
    }

    #[inline]
    pub fn push(&mut self, val: LuaValue) {
        self.vec.push(val);
    }

    #[inline]
    pub fn pop(&mut self) -> LuaValue {
        self.vec.pop().unwrap()
    }

    #[inline]
    pub fn abs_index(&self, idx: isize) -> isize {
        if idx >= 0 {
            idx
        } else {
            idx + self.top() + 1
        }
    }

    #[inline]
    pub fn is_valid(&self, idx: isize) -> bool {
        let abs_idx = self.abs_index(idx);
        abs_idx > 0 && abs_idx <= self.top()
    }

    pub fn get(&self, idx: isize) -> LuaValue {
        let abs_idx = self.abs_index(idx);
        if abs_idx > 0 && abs_idx <= self.top() {
            let idx = abs_idx as usize - 1;
            self.vec[idx].clone() // TODO
        } else {
            LuaValue::Nil
        }
    }

    pub fn set(&mut self, idx: isize, val: LuaValue) {
        let abs_idx = self.abs_index(idx);
        if abs_idx > 0 && abs_idx <= self.top() {
            let idx = abs_idx as usize - 1;
            self.vec[idx] = val;
        } else {
            // todo: result
            panic!("invalid index!");
        }
    }

    pub fn reverse(&mut self, mut from: usize, mut to: usize) {
        while from < to {
            self.vec.swap(from, to);
            from += 1;
            to -= 1;
        }
    }
}
