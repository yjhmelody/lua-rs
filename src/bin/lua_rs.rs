#![allow(unused_must_use)]

extern crate lua_rs;

use lua_rs::binary::*;
use lua_rs::compiler::codegen::gen_prototype;
use lua_rs::compiler::lexer::*;
use lua_rs::compiler::parser::parse_block;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args = env::args().collect::<Vec<_>>();

    if args.len() < 3 {
        println!(r##"
        usage:
        sub command
            lexer       file
            parser      file
            codegen     file
            bytecode    file
        "##);
        std::process::exit(0);
    }

    let path = Path::new(&args[2]);
    let file = fs::read(path).expect("couldn't find file");
    let file_name = path.file_name().unwrap();

    if &args[1] == "lexer" {
        let mut lexer = Lexer::from_iter(file, file_name.to_str().unwrap().to_string());
        println!("{:?}\n", file_name);
        while let Ok(tok) = lexer.next_token() {
            println!("{:?}", tok);
        }
    } else if &args[1] == "parser" {
        let mut lexer = Lexer::from_iter(file, file_name.to_str().unwrap().to_string());
        let block = parse_block(&mut lexer).expect("parse error");
        println!("{:?}\n", file_name);
        println!("{:#?}", block);
    } else if &args[1] == "codegen" {
        let mut lexer = Lexer::from_iter(file, file_name.to_str().unwrap().to_string());
        let block = parse_block(&mut lexer).expect("parse error");
        let proto = gen_prototype(Box::new(block)).unwrap();
        println!("{:?}\n", file_name);
        println!("{:#?}", proto);
    } else if &args[1] == "bytecode" {
        let mut lexer = Lexer::from_iter(file, file_name.to_str().unwrap().to_string());
        let block = parse_block(&mut lexer).expect("parse error");
        let proto = gen_prototype(Box::new(block)).unwrap();
        let file_name = file_name.to_str().unwrap().to_string();
        let bytecode = encode(proto, Some("@".to_string() + &file_name));
        let s = unsafe { String::from_utf8_unchecked(bytecode) };
        println!("{:?}\n", file_name);
        fs::write(path.file_stem().unwrap().to_str().unwrap().to_string() + ".out", s);
    } else {
        println!("not a legal command")
    }
}