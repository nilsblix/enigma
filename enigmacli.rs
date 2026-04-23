#![allow(unused)]

use enigma::{ByteAddress, Machine, Memory, Registers, SystemCall, image::Image, is};
use std::{
    fs::File,
    io::{self, Read},
    mem::ManuallyDrop,
    os::unix::io::FromRawFd,
};

/// Arguments and their libc correspondants:
/// * `r2`: `int fd`          (the file descriptor to read from),
/// * `r3`: `void buf[count]` (ptr to the buffer to read bytes in to),
/// * `r4`: `size_t count`    (number of bytes to read),
struct ReadFromFd {}

impl SystemCall for ReadFromFd {
    fn invoke(&mut self, mem: &mut Memory, regs: &mut Registers) {
        use io::Read;

        let fd = regs.read(2) as i32;
        let buf_addr = ByteAddress(regs.read(3));
        let count = regs.read(4) as usize;

        let mut f = ManuallyDrop::new(unsafe { File::from_raw_fd(fd) });
        let mut buf = vec![0u8; count];
        let bytes_read = (&mut *f).read(buf.as_mut_slice()).unwrap_or(0);

        mem.write_raw_bytes(buf_addr, &buf[..bytes_read]);
        regs.write(1, bytes_read as u32);
    }
}

/// Arguments and their libc correspondants:
/// * `r2`: `int fd`                (the file descriptor to write to),
/// * `r3`: `const void buf[count]` (ptr to the buffer of bytes to write),
/// * `r4`: `size_t count`          (number of bytes to write),
struct WriteToFd {}

impl SystemCall for WriteToFd {
    fn invoke(&mut self, mem: &mut Memory, regs: &mut Registers) {
        use io::Write;

        let fd = regs.read(2) as i32;
        let buf_addr = ByteAddress(regs.read(3));
        let count = regs.read(4) as usize;

        let mut f = ManuallyDrop::new(unsafe { File::from_raw_fd(fd) });
        let mut buf = vec![0u8; count];
        mem.read_raw_bytes(buf_addr, buf.as_mut_slice());
        _ = (&mut *f).write_all(buf.as_slice());
        _ = (&mut *f).flush();
        regs.write(1, count as u32);
    }
}

const SYSCALL_READ_FROM_FD: u16 = 0;
const SYSCALL_WRITE_TO_FD: u16 = 1;

const STDIN_FD: u16 = 0;
const STDOUT_FD: u16 = 1;
const STDERR_FD: u16 = 2;

fn attach_os_to_machine(m: &mut Machine) {
    _ = m.attach_system_call(SYSCALL_WRITE_TO_FD as u32, WriteToFd {});
    _ = m.attach_system_call(SYSCALL_READ_FROM_FD as u32, ReadFromFd {});
}

fn build_from_bytecode(bytecode: &[u8]) -> Machine {
    let image = Image::from_chunk_bytes(bytecode).expect("invalid EVM image");
    let mut m = image.branch_to_machine();
    attach_os_to_machine(&mut m);
    m
}

fn usage() -> ! {
    println!("Usage: evm [run-image | run-asm | emit-image] file_path");
    std::process::exit(1);
}

fn open_next_file(args: &mut std::env::Args) -> (String, std::fs::File) {
    let file_path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("error: no path to bytecode was given.");
            usage();
        }
    };

    let path = std::path::Path::new(file_path.as_str());
    match std::fs::File::open(path) {
        Ok(f) => (file_path, f),
        Err(_) => {
            eprintln!("error: file {file_path} doesn't exist.");
            usage();
        }
    }
}

fn read_all_from_file(f: &mut std::fs::File) -> Vec<u8> {
    let mut s = Vec::new();
    _ = match f.read_to_end(&mut s) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("error: io error: {}", e);
            std::process::exit(1);
        }
    };
    s
}

fn read_all_from_next_file(args: &mut std::env::Args) -> (String, Vec<u8>) {
    let (path, mut f) = open_next_file(args);
    (path, read_all_from_file(&mut f))
}

fn print_asm_diagnostics(path: &str, diags: &enigma::asm::Diagnostics<'_>) {
    eprintln!("{}", diags.with_path(path));
}

fn run_image(args: &mut std::env::Args) {
    let (_, bytecode) = read_all_from_next_file(args);
    let mut m = build_from_bytecode(bytecode.as_slice());
    match m.exec_while_not_halt() {
        Ok(_) => {}
        Err(is::InstructionError::InvalidOperation { opcode }) => {
            eprintln!("error: found invalid opcode: {opcode}.");
            std::process::exit(1);
        }
    };
}

fn run_asm(args: &mut std::env::Args) {
    let (file_path, bytes) = read_all_from_next_file(args);
    let src = match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("utf8 error: {}", e);
            std::process::exit(1);
        }
    };
    let img = match enigma::asm::assemble_str(src.as_str()) {
        Ok(i) => i,
        Err(ds) => {
            print_asm_diagnostics(file_path.as_str(), &ds);
            std::process::exit(1);
        }
    };
    let mut m = img.consume_to_machine();
    attach_os_to_machine(&mut m);
    match m.exec_while_not_halt() {
        Ok(_) => {}
        Err(is::InstructionError::InvalidOperation { opcode }) => {
            eprintln!("error: found invalid opcode: {opcode}.");
            std::process::exit(1);
        }
    };
}

fn emit_image(args: &mut std::env::Args) {
    let (file_path, bytes) = read_all_from_next_file(args);
    let src = match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("utf8 error: {}", e);
            std::process::exit(1);
        }
    };
    let img = match enigma::asm::assemble_str(src.as_str()) {
        Ok(i) => i,
        Err(ds) => {
            print_asm_diagnostics(file_path.as_str(), &ds);
            std::process::exit(1);
        }
    };

    let out_path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("error: no out path was given.");
            usage();
        }
    };

    let path = std::path::Path::new(out_path.as_str());
    let mut out_file = match std::fs::File::create_new(path) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("error: file {out_path} already exists:");
            usage();
        }
    };

    img.dump_chunks(&mut out_file);
}

fn main() {
    let mut args = std::env::args();
    _ = args.next();

    let program = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("error: no program was specified.");
            usage();
        }
    };

    match program.as_str() {
        "run-image" => run_image(&mut args),
        "run-asm" => run_asm(&mut args),
        "emit-image" => emit_image(&mut args),
        f => {
            eprintln!("unknown program: '{}'", f);
            usage();
        }
    }
}
