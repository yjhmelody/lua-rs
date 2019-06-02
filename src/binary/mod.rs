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

pub fn encode(proto: Rc<chunk::Prototype>) -> Vec<u8> {
    let mut writer = writer::Writer::new();
    writer.write_header();
    writer.write_byte(4);
    writer.write_proto(proto);
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
        assert_eq!(1, 0);
    }


    #[test]
    fn test_decode() {
        let s = fs::read("D:/code/Rust/lua-rs/tests/luac.out").expect("error");
        let proto = decode(s);

        println!("{:#?}", proto);
        assert_eq!(1, 0);
    }

    #[test]
    fn test_encode() {
        let s = fs::read("D:/code/Rust/lua-rs/tests/luac.out").expect("error");
        let proto = encode(decode(s));
//        println!("{:#?}", proto);

        let s = unsafe { String::from_utf8_unchecked(proto) };
        fs::write("D:/code/Rust/lua-rs/tests/test.out", s);
        assert_eq!(2, 0);
    }
}
