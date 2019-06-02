use std::rc::Rc;

use crate::binary::chunk::*;

#[derive(Debug, Clone)]
pub struct Writer {
    data: Vec<u8>,
}

impl Writer {
    #[inline]
    pub fn new() -> Self {
        Self { data: Vec::with_capacity(1024) }
    }

    #[inline]
    pub fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }

    #[inline]
    pub fn as_bytes(self) -> Vec<u8> {
        self.data
    }

    fn write_u32(&mut self, b4: u32) {
        let a0 = (b4 >> 24) as u8;
        let a1 = (b4 >> 16) as u8;
        let a2 = (b4 >> 8) as u8;
        let a3 = b4 as u8;
        self.write_byte(a3);
        self.write_byte(a2);
        self.write_byte(a1);
        self.write_byte(a0);
    }

    #[inline]
    fn write_u64(&mut self, b8: u64) {
        dbg!(&b8);
        let a0 = ((b8 & 0xFF_FF_FF_FF_00_00_00_00) >> 32) as u32;
        let a1 = (b8 & 0xFF_FF_FF_FF) as u32;

        self.write_u32(a1);
        self.write_u32(a0);
    }

    #[inline]
    fn write_lua_integer(&mut self, i: i64) {
        self.write_u64(i as u64);
    }

    #[inline]
    fn write_lua_number(&mut self, n: f64) {
        self.write_u64(n.to_bits());
    }

    #[inline]
    fn write_bytes(&mut self, bytes: Vec<u8>) {
        for b in bytes {
            self.write_byte(b);
        }
    }

    #[inline]
    fn write_string(&mut self, s: &String) {
        self.write_string0(s).unwrap_or_default()
    }

    fn write_string0(&mut self, s: &String) -> Option<()> {
        if s.len() == 0 {
            None
        } else if s.len() < 0xFF {
            self.write_byte(s.len() as u8 - 1);
            self.write_bytes(s.as_bytes().to_vec());
            Some(())
        } else {
            // todo
            unimplemented!()
        }
    }

    pub fn write_header(&mut self) {
        self.write_bytes(LUA_SIGNATURE.to_vec());
        self.write_byte(LUAC_VERSION);
        self.write_byte(LUAC_FORMAT);
        self.write_bytes(LUAC_DATA.to_vec());
        self.write_byte(CINT_SIZE);
        self.write_byte(CSIZET_SIZE);
        self.write_byte(INSTRUCTION_SIZE);
        self.write_byte(LUA_INTEGER_SIZE);
        self.write_byte(LUA_NUMBER_SIZE);
        self.write_lua_integer(LUAC_INT);
        self.write_lua_number(LUAC_NUM);
    }

    pub fn write_proto(&mut self, proto: Rc<Prototype>) {
        self.write_string0(&proto.source.clone().unwrap());
        self.write_u32(proto.line_defined);
        self.write_u32(proto.last_line_defined);
        self.write_byte(proto.num_params);
        self.write_byte(proto.is_vararg);
        self.write_byte(proto.max_stack_size);

        self.write_u32(proto.code.len() as u32);
        for ins in proto.code.iter() {
            self.write_u32(*ins);
        }

        self.write_u32(proto.constants.len() as u32);
        for cst in proto.constants.iter() {
            self.write_constant(cst);
        }

        self.write_u32(proto.up_values.len() as u32);
        for up_val in proto.up_values.iter() {
            self.write_up_value(up_val);
        }

        self.write_u32(proto.prototypes.len() as u32);
        for prototype in proto.prototypes.iter() {
            self.write_proto(prototype.clone());
        }

        self.write_u32(proto.line_info.len() as u32);
        for line in proto.line_info.iter() {
            self.write_u32(*line);
        }

        self.write_u32(proto.local_vars.len() as u32);
        for local_var in proto.local_vars.iter() {
            self.write_loc_var(local_var);
        }

        self.write_u32(proto.up_value_names.len() as u32);
        for name in proto.up_value_names.iter() {
            self.write_string(name);
        }
    }

    fn write_constant(&mut self, cst: &Constant) {
        match cst {
            Constant::Nil => { self.write_byte(TAG_NIL) }
            Constant::Boolean(b) => {
                self.write_byte(TAG_BOOLEAN);
                self.write_byte(*b as u8);
            }
            Constant::Integer(i) => {
                self.write_byte(TAG_INTEGER);
                self.write_lua_integer(*i);
            }
            Constant::Number(n) => {
                self.write_byte(TAG_NUMBER);
                self.write_lua_number(*n);
            }
            Constant::String(s) => {
                if s.len() >= 255 {
                    self.write_byte(TAG_LONG_STR);
                } else {
                    self.write_byte(TAG_SHORT_STR);
                }
                self.write_string(s);
            }
        };
    }


    #[inline]
    fn write_up_value(&mut self, upval: &UpValue) {
        self.write_byte(upval.instack);
        self.write_byte(upval.idx);
    }

    #[inline]
    fn write_loc_var(&mut self, local_var: &LocalVar) {
        self.write_string(&local_var.var_name);
        self.write_u32(local_var.start_pc);
        self.write_u32(local_var.end_pc);
    }
}
