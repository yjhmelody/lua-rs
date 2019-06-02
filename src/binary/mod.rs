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
        let chunk = fs::read("D:/code/Rust/lua-rs/tests/luac.out").expect("error");
        let proto = encode(decode(chunk.clone()), Some("@example.lua".to_string()));
        let proto = encode(decode(chunk.clone()), Some("@hello.lua".to_string()));

        let s = unsafe { String::from_utf8_unchecked(proto.clone()) };
        fs::write("D:/code/Rust/lua-rs/tests/test.out", s);
        let mut i = 0;
        for (a, b) in chunk.iter().zip(proto.iter()) {
            println!("{}", i);
            assert_eq!(*a, *b);
            i += 1;
        }
        assert_eq!(chunk[33..], proto[33..]);
    }
}
