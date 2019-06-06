use std::rc::Rc;

pub mod chunk;
pub mod reader;
pub mod writer;


/// decode Lua binary chunk to prototype structure
pub fn decode(data: Vec<u8>) -> Rc<chunk::Prototype> {
    let mut r = reader::Reader::new(data);
    r.check_header();
    r.read_byte();
    r.read_proto()
}

pub fn encode(proto: Rc<chunk::Prototype>, src: Option<String>) -> Vec<u8> {
    let mut writer = writer::Writer::new();
    writer.write_header();
    writer.write_byte(1);
    writer.write_proto(proto, src);
    writer.as_bytes()
}

mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_header() {
        let mut writer = writer::Writer::new();
        writer.write_header();
        let mut reader = reader::Reader::new(writer.as_bytes());
        reader.check_header();
    }


    #[test]
    fn test_decode() {
        let s = fs::read("./tests/luac.out").expect("error");
        let proto = decode(s);
    }

    fn test_encode() {
        let chunk = fs::read("./tests/luac.out").expect("error");
        let proto = encode(decode(chunk.clone()), Some("@hello.lua".to_string()));
        let s = unsafe { String::from_utf8_unchecked(proto.clone()) };
        fs::write("./tests/test.out", s);
    }
}
