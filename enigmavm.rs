#![allow(unused)]

use enigma::{Builder, ByteAddress, Machine, Memory, Registers, SystemCall, is};
use std::{
    fs::File,
    io::{self, Read},
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

        let f = unsafe { File::from_raw_fd(fd) };
        let mut x = Vec::with_capacity(count);
        _ = f.take(count as u64).read_to_end(&mut x);

        mem.write_raw_bytes(buf_addr, x.as_slice());
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

        let mut f = unsafe { File::from_raw_fd(fd) };
        let mut buf = Vec::with_capacity(count);
        unsafe { buf.set_len(count) }
        mem.read_raw_bytes(buf_addr, buf.as_mut_slice());
        let s = String::from_utf8(buf).expect("non utf8 bytes");
        _ = write!(&mut f, "{}", s);

        _ = f.flush();
    }
}

const SYSCALL_READ_FROM_FD: u16 = 0;
const SYSCALL_WRITE_TO_FD: u16 = 1;

const STDIN_FD: u16 = 0;
const STDOUT_FD: u16 = 1;
const STDERR_FD: u16 = 2;

fn attach_os_to_machine(m: &mut Machine) {
    _ = m.attach_system_call(SYSCALL_READ_FROM_FD as u32, ReadFromFd {});
    _ = m.attach_system_call(SYSCALL_WRITE_TO_FD as u32, WriteToFd {});
}

fn build_from_bytecode(bytecode: &[u8]) -> Machine {
    let builder = Builder::from_chunk_bytes(bytecode).expect("invalid EVM image");
    let mut m = builder.branch_to_machine();
    attach_os_to_machine(&mut m);
    m
}

fn usage() -> ! {
    println!("Usage: evm file_path");
    std::process::exit(1);
}

fn main() {
    let mut args = std::env::args();
    _ = args.next();

    let bytecode_path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("error: no path to bytecode was given.");
            usage();
        }
    };

    let path = std::path::Path::new(bytecode_path.as_str());
    let mut f = match std::fs::File::open(path) {
        Ok(b) => b,
        Err(_) => {
            eprintln!("error: file {bytecode_path} doesn't exist.");
            usage();
        }
    };

    let mut bytecode = Vec::new();
    _ = match f.read_to_end(&mut bytecode) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("error: io error: {}", e);
            std::process::exit(1);
        }
    };

    let mut m = build_from_bytecode(bytecode.as_slice());
    match m.exec_while_not_halt() {
        Ok(_) => {}
        Err(is::InstructionError::InvalidOperation { opcode }) => {
            eprintln!("error: found invalid opcode: {opcode}.");
            std::process::exit(1);
        }
    };
}
