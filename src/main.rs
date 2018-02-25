#[macro_use] extern crate sam;
extern crate machina;

mod recompiler;
mod token;

use recompiler::Recompiler;
use std::io::prelude::*;
use std::fs::File;
use std::env;
use machina::memory::Memory;

fn print(d: u8) {
    print!("{}", d as char);
}

fn main() {
    if env::args().nth(1).unwrap() == "--recompile" {
        let mut recompiler = Recompiler::new();
        let mut file = File::open(env::args().nth(2).unwrap()).unwrap();
        let mut buf = String::from("");
        file.read_to_string(&mut buf).unwrap();
        recompiler.translate(buf);
    
        let mut buffer = File::create(env::args().nth(3).unwrap()).unwrap();
        let _ = buffer.write(&recompiler.bytes);
    } else if env::args().nth(1).unwrap() == "--execute" {
        let mut file = File::open(env::args().nth(2).unwrap()).unwrap();
        let mut buf = Vec::<u8>::new();
        file.read_to_end(&mut buf).unwrap();

        let mem = [0u8; 10000];

        let mut memory = Memory::new(1);
        memory.emit_bytes(vec![0x48, 0xb8]); // mov rax, mem
        memory.emit64((&mem as *const _) as u64);
        memory.emit_bytes(vec![0x49, 0xbc]); // mov r12, print
        memory.emit64(print as u64);
        memory.emit_bytes(buf);
        let _ = memory.execute();
    } else {
        println!("Invalid option");
    }
}
