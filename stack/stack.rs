use enigma::is::Instruction;
use enigma::{Machine, Memory, Width, ByteAddress, IoController};

#[derive(Debug)]
enum Token {
    Number { value: i32 },
    Add,
    Print,
}

struct Lexer {
    tokens: Vec<Token>,
}

impl Lexer {
    fn new(content: &str) -> Lexer {
        let tokens: Vec<Token> = content
            .split_whitespace()
            .map(|ident| match ident {
                "+" => Some(Token::Add),
                "." => Some(Token::Print),
                _ => if let Ok(value) = i32::from_str_radix(ident, 10) {
                    Some(Token::Number { value })
                } else {
                    None
                }
            })
            .filter(|t| t.is_some())
            .map(|t| t.unwrap())
            .collect();
        Lexer { tokens }
    }
}

const USAGE: &'static str =
"\
Usage:
    $ compiler [Commands]

Commands:
    run      [path] Parse a file, then run the program.
    compile  [path] Emit bytecode at filename + `.ebc`.
    bytecode [path] Run pure bytecode of the specified file.
";

fn usage() -> ! {
    print!("{USAGE}");
    std::process::exit(1);
}

struct EmitBytes {
    ptr: Option<ByteAddress>,
    len: Option<u32>,
}

impl EmitBytes {
    fn new() -> EmitBytes {
        EmitBytes { ptr: None, len: None }
    }
}

impl IoController for EmitBytes {
    /// 1 means ready, 0 means not ready.
    fn read(&mut self, _: &mut Memory, _: ByteAddress, width: Width) -> u32 {
        match width {
            Width::Byte => self.ptr = None,
            Width::Halfword => self.ptr = None,
            Width::Word => if self.ptr.is_some() && self.len.is_some() {
                return 1;
            },
        }

        0
    }

    fn write(&mut self, mem: &mut Memory, _: ByteAddress, width: Width, data: u32) {
        if width != Width::Word {
            return;
        }

        if self.ptr.is_none() {
            self.ptr = Some(ByteAddress(data));
        } else if self.len.is_none() {
            self.len = Some(data);
        } else {
            let mut buf = Vec::new();
            unsafe { buf.set_len(self.len.unwrap() as usize) };
            mem.read_raw_bytes(self.ptr.unwrap(), buf.as_mut_slice());
            let s = String::from_utf8(buf);
            if let Ok(s) = s {
                print!("{s}");
            }
        }
    }
}

fn compile_program(lexer: Lexer, emit_addr: ByteAddress) -> Vec<Instruction> {
    let mut out = Vec::new();
    for t in lexer.tokens {
        match t {
            Token::Number { value } => {
                out.push(Instruction::subi(31, 31, 4));
                let lower = (value & 0x0000_FFFF) as u16;
                out.push(Instruction::xori(1, 0, lower));
                let upper = (value as u32 & 0xFFFF_0000) as u16;
                out.push(Instruction::xorui(1, 0, upper));
                out.push(Instruction::stw(1, 31, 0));
            },
            Token::Add => {
                out.push(Instruction::ldw(1, 31, 0));
                out.push(Instruction::addi(31, 31, 4));
                out.push(Instruction::ldw(2, 31, 0));
                out.push(Instruction::debu(1));
                out.push(Instruction::debu(2));
                out.push(Instruction::add(1, 1, 2));
                out.push(Instruction::stw(1, 31, 0));
            },
            Token::Print => {
                // 1. put emit_addr into r1.
                let emit = emit_addr.0;
                let lower = (emit & 0x0000_FFFF) as u16;
                out.push(Instruction::xori(1, 0, lower));
                let upper = (emit & 0xFFFF_0000) as u16;
                out.push(Instruction::xorui(1, 0, upper));
                // 2. write addr of head into emit_addr
                out.push(Instruction::stw(31, 1, 0));
                // 4. set r2 to 4 (# bytes we want to print).
                out.push(Instruction::xori(2, 0, 4));
                // 4. write r2 into emit_addr
                out.push(Instruction::stw(2, 1, 0));
                // 5. call emit.
                out.push(Instruction::stw(0, 1, 0));
                // 6. clear emit
                out.push(Instruction::stb(0, 1, 0));
                out.push(Instruction::sthw(0, 1, 0));
            },
        }
    }
    out.push(Instruction::HALT);
    out
}

fn parse_and_run_program(program: &str) {
    let lexer = Lexer::new(program);
    let mut m = Machine::new();
    let emit_addr = m.attach_io_controller(EmitBytes::new())
        .expect("could not attach EmitBytes to machine");

    let ins = compile_program(lexer, emit_addr);
    for i in ins.iter() {
        println!("INSTR: {:?}", i);
    }
    m.override_with_instructions(ins.as_slice());

    // FIXME: don't unwrap...
    m.exec_while_not_halt().unwrap();
}

fn parse_and_run_file(path: &str) {
    let p = std::path::Path::new(path);
    let program = match std::fs::read_to_string(p) {
        Ok(s) => s,
        Err(_) => {
            println!("error: could not find file '{}'", path);
            std::process::exit(1);
        },
    };
    parse_and_run_program(program.as_str());
}

fn main() {
    let mut args = std::env::args();
    _ = args.next();

    let cmd = match args.next() {
        Some(c) => c,
        None => usage(),
    };

    match cmd.as_str() {
        "run" => {
            let path = match args.next() {
                Some(p) => p,
                None => {
                    println!("error: no path was supplied");
                    usage();
                },
            };
            parse_and_run_file(path.as_str());
        },
        "compile" | "bytecode" => unimplemented!(),
        unknown => {
            println!("error: unknown command '{unknown}'");
            usage();
        },
    }
}
