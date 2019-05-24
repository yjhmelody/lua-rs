use std::rc::Rc;

pub mod chunk;
mod reader;

/// decode Lua binary chunk to prototype structure
pub fn decode(data: Vec<u8>) -> Rc<chunk::Prototype> {
    let mut r = reader::Reader::new(data);
    r.check_header();
    r.read_byte();
    r.read_proto()
}
