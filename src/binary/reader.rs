use std::rc::Rc;

use crate::binary::chunk::*;

#[derive(Debug, Clone)]
pub struct Reader {
    data: Vec<u8>,
    pos: usize,
}

impl Reader {
    #[inline]
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    #[inline]
    pub fn read_byte(&mut self) -> u8 {
        let b = self.data[self.pos];
        self.pos += 1;
        b
    }

    #[inline]
    fn read_u32(&mut self) -> u32 {
        let a0 = self.read_byte() as u32;
        let a1 = self.read_byte() as u32;
        let a2 = self.read_byte() as u32;
        let a3 = self.read_byte() as u32;
        (a3 << 24) | (a2 << 16) | (a1 << 8) | a0
    }

    #[inline]
    fn read_u64(&mut self) -> u64 {
        let a0 = self.read_u32() as u64;
        let a1 = self.read_u32() as u64;
        dbg!(a1 << 32 | a0);
        (a1 << 32) | a0
    }

    #[inline]
    fn read_lua_integer(&mut self) -> i64 {
        self.read_u64() as i64
    }

    #[inline]
    fn read_lua_number(&mut self) -> f64 {
        use std::f64; // TODO
        f64::from_bits(self.read_u64())
    }

    fn read_bytes(&mut self, n: usize) -> Vec<u8> {
        let mut vec = Vec::new();
        for _ in 0..n {
            vec.push(self.read_byte());
        }
        vec
    }

    #[inline]
    fn read_string(&mut self) -> String {
        self.read_string0().unwrap_or_default()
    }

    fn read_string0(&mut self) -> Option<String> {
        let mut size = self.read_byte() as usize;
        if size == 0 {
            return None;
        }
        if size == 0xFF {
            size = self.read_u64() as usize; // size_t
        }
        let bytes = self.read_bytes(size - 1);
        let string = String::from_utf8(bytes);
        string.ok()
    }

    fn read_vec<T, F>(&mut self, f: F) -> Vec<T>
        where
            F: Fn(&mut Reader) -> T,
    {
        let n = self.read_u32() as usize;
        let mut vec = Vec::with_capacity(n);
        for _i in 0..n {
            vec.push(f(self));
        }
        vec
    }

    pub fn check_header(&mut self) {
        assert_eq!(self.read_bytes(4), LUA_SIGNATURE, "not a precompiled chunk!");
        assert_eq!(self.read_byte(), LUAC_VERSION, "version mismatch!");
        assert_eq!(self.read_byte(), LUAC_FORMAT, "format mismatch!");
        assert_eq!(self.read_bytes(6), LUAC_DATA, "corrupted!");
        assert_eq!(self.read_byte(), CINT_SIZE, "int size mismatch!");
        assert_eq!(self.read_byte(), CSIZET_SIZE, "size_t size mismatch!");
        assert_eq!(self.read_byte(), INSTRUCTION_SIZE, "instruction size mismatch!");
        assert_eq!(self.read_byte(), LUA_INTEGER_SIZE, "lua_Integer size mismatch!");
        assert_eq!(self.read_byte(), LUA_NUMBER_SIZE, "lua_Number size mismatch!");
        assert_eq!(self.read_lua_integer(), LUAC_INT, "endianness mismatch!");
        assert_eq!(self.read_lua_number(), LUAC_NUM, "float format mismatch!");
    }

    #[inline]
    pub fn read_proto(&mut self) -> Rc<Prototype> {
        self.read_proto0(None)
    }

    fn read_proto0(&mut self, parent_source: Option<String>) -> Rc<Prototype> {
        let source = self.read_string0().or(parent_source);
        Rc::new(Prototype {
            source: source.clone(), // debug
            line_defined: self.read_u32(),
            last_line_defined: self.read_u32(),
            num_params: self.read_byte(),
            is_vararg: self.read_byte(),
            max_stack_size: self.read_byte(),
            code: self.read_vec(|r| r.read_u32()),
            constants: self.read_vec(|r| r.read_constant()),
            up_values: self.read_vec(|r| r.read_up_value()),
            prototypes: self.read_vec(|r| r.read_proto0(source.clone())),
            line_info: self.read_vec(|r| r.read_u32()),        // debug
            local_vars: self.read_vec(|r| r.read_loc_var()),     // debug
            up_value_names: self.read_vec(|r| r.read_string()), // debug
        })
    }

    fn read_constant(&mut self) -> Constant {
        let tag = self.read_byte();
        match tag {
            TAG_NIL => Constant::Nil,
            TAG_BOOLEAN => Constant::Boolean(self.read_byte() != 0),
            TAG_INTEGER => Constant::Integer(self.read_lua_integer()),
            TAG_NUMBER => Constant::Number(self.read_lua_number()),
            TAG_SHORT_STR => Constant::String(self.read_string()),
            TAG_LONG_STR => Constant::String(self.read_string()),
            _ => panic!("corrupted!"),
        }
    }

    #[inline]
    fn read_up_value(&mut self) -> UpValue {
        UpValue {
            instack: self.read_byte(),
            idx: self.read_byte(),
        }
    }

    #[inline]
    fn read_loc_var(&mut self) -> LocalVar {
        LocalVar {
            var_name: self.read_string(),
            start_pc: self.read_u32(),
            end_pc: self.read_u32(),
        }
    }
}
