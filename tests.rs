#[cfg(test)]
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::is::{self, Payload};
use super::*;

#[test]
fn encode_and_decode_instruction() {
    //          |  op | rr | ra | rb |  packing  |
    //          |     V    V    V    V           |
    let word = 0b00000010000010100010011111111111 as u32;
    let inst = Instruction {
        op: Op::Noop,
        payload: Payload::Noop,
    };
    let res = Instruction::decode(word).unwrap();
    assert_eq!(res, inst);
    assert_eq!(res.op.name(), "noop");
    assert_eq!(res.encode(), 0);

    //          |  op | rr | ra | rb |  packing  |
    //          |     V    V    V    V           |
    let word = 0b00000110000010100010111111111111 as u32;
    let inst = Instruction {
        op: Op::Add,
        payload: Payload::R {
            rr: 16,
            ra: 10,
            rb: 5,
        },
    };
    let res = Instruction::decode(word).unwrap();
    assert_eq!(res, inst);
    assert_eq!(res.op.name(), "add");
    assert_eq!(res.encode(), 0b00000110000010100010100000000000);

    //          |  op | rr | ra |    immediate   |
    //          |     V    V    V                |
    let word = 0b10001010000010100000000010010101 as u32;
    let inst = Instruction {
        op: Op::Subi,
        payload: Payload::I {
            rr: 16,
            ra: 10,
            immediate: 149,
        },
    };
    let res = Instruction::decode(word).unwrap();
    assert_eq!(res, inst);
    assert_eq!(res.op.name(), "sub_i");
    assert_eq!(res.encode(), word);

    //          |  op | rr | ra |    immediate   |
    //          |     V    V    V                |
    let word = 0b11100110000010100000000010010101 as u32;
    let inst = Instruction {
        op: Op::Jmp,
        payload: Payload::I {
            rr: 16,
            ra: 10,
            immediate: 149,
        },
    };
    let res = Instruction::decode(word).unwrap();
    assert_eq!(res, inst);
    assert_eq!(res.op.name(), "jmp_i");
    assert_eq!(res.encode(), word);
}

#[test]
fn painfully_written_execute_and_advance() {
    let mut m = Machine::new();
    m.write_register(2, 34);
    m.write_register(3, 35);

    // add r0, r1, r2
    //          |  op | rr | ra | rb |  packing  |
    //          |     V    V    V    V           |
    let word = 0b00000100001000100001100000000000 as u32;
    let bytes = word.to_be_bytes();
    let mut mem = Box::new([0u8; BLOCK_SIZE]);
    mem.as_mut_slice()[0] = bytes[0];
    mem.as_mut_slice()[1] = bytes[1];
    mem.as_mut_slice()[2] = bytes[2];
    mem.as_mut_slice()[3] = bytes[3];
    m.mem.blocks[0] = Block::Memory(mem);

    let outcome = m.exec_and_advance().unwrap();
    assert!(!outcome.1.jumped);
    assert_eq!(m.read_register(1), 69);
}

#[test]
fn execute_and_advance() {
    // expected behaviour:
    // 1) set r1 to 33 + 34 = 67
    // 2) see that r1 is eq to r4, therefore jump to:
    // 3) set r6 to 67 (0b1000011) to 268 (0b100001100)
    //
    // i.e the noop should not be executed.
    let instructions = [
        is::add(1, 2, 3),
        is::beq(1, 4, 2),
        is::noop(),
        is::shl(6, 1, 5),
    ];

    let mut m = Machine::from_instructions(instructions.as_slice());
    m.write_register(2, 33);
    m.write_register(3, 34);
    m.write_register(4, 67);
    m.write_register(5, 2);

    let outcome = m.exec_and_advance().unwrap();
    assert!(!outcome.1.jumped);
    assert_eq!(m.read_register(1), 67);

    let outcome = m.exec_and_advance().unwrap();
    assert!(outcome.1.jumped);

    let outcome = m.exec_and_advance().unwrap();
    assert!(!outcome.1.jumped);
    assert_eq!(m.read_register(6), 268);
}

#[test]
fn jump_and_link_uses_word_offsets() {
    let instructions = [is::jmp(1, 2), is::noop(), is::addi(2, 0, 9)];

    let mut m = Machine::from_instructions(instructions.as_slice());

    let outcome = m.exec_and_advance().unwrap();
    assert!(outcome.1.jumped);
    assert_eq!(m.read_register(1), WORD_SIZE_BYTES);

    let outcome = m.exec_and_advance().unwrap();
    assert!(!outcome.1.jumped);
    assert_eq!(m.read_register(2), 9);
}

#[test]
fn register_relative_jump_uses_word_offsets() {
    let instructions = [
        is::jmpr(1, 3, 2),
        is::noop(),
        is::noop(),
        is::addi(2, 0, 11),
    ];

    let mut m = Machine::from_instructions(instructions.as_slice());
    m.write_register(3, WORD_SIZE_BYTES);

    let outcome = m.exec_and_advance().unwrap();
    assert!(outcome.1.jumped);
    assert_eq!(m.read_register(1), WORD_SIZE_BYTES);

    let outcome = m.exec_and_advance().unwrap();
    assert!(!outcome.1.jumped);
    assert_eq!(m.read_register(2), 11);
}

#[test]
fn simple_store_and_load() {
    let is = [
        is::stw(1, 2, 0x100),
        is::sthw(3, 2, 0x104),
        is::stb(4, 2, 0x106),
        is::addi(1, 0, 0),
        is::addi(3, 0, 0),
        is::addi(4, 0, 0),
        is::ldw(5, 2, 0x100),
        is::ldhwu(6, 2, 0x104),
        is::ldbu(7, 2, 0x106),
        Instruction::HALT,
    ];

    let mut m = Machine::from_instructions(is.as_slice());
    m.write_register(1, 0x1234_5678);
    m.write_register(2, 0x20000);
    m.write_register(3, 0xBEEF);
    m.write_register(4, 0xAB);

    m.exec_while_not_halt().unwrap();

    assert_eq!(m.read_register(1), 0);
    assert_eq!(m.read_register(3), 0);
    assert_eq!(m.read_register(4), 0);
    assert_eq!(m.read_register(5), 0x1234_5678);
    assert_eq!(m.read_register(6), 0x0000_BEEF);
    assert_eq!(m.read_register(7), 0x0000_00AB);

    assert_eq!(m.mem.read_raw_word(ByteAddress(0x20_100)), 0x1234_5678);
    assert_eq!(m.mem.read_raw_half_word(ByteAddress(0x20_104)), 0xBEEF);
    assert_eq!(m.mem.read_raw_byte(ByteAddress(0x20_106)), 0xAB);
}

#[test]
fn signed_and_unsigned_loads_extend_correctly() {
    let is = [
        is::ldhw(1, 10, 0x10),
        is::ldhwu(2, 10, 0x10),
        is::ldb(3, 10, 0x20),
        is::ldbu(4, 10, 0x20),
        Instruction::HALT,
    ];

    let mut m = Machine::from_instructions(is.as_slice());
    m.write_register(10, 0x20_000);

    m.write_half_word(ByteAddress(0x20_010), 0x8001);
    m.write_byte(ByteAddress(0x20_020), 0x80);

    m.exec_while_not_halt().unwrap();

    assert_eq!(m.read_register(1), 0xFFFF_8001);
    assert_eq!(m.read_register(2), 0x0000_8001);
    assert_eq!(m.read_register(3), 0xFFFF_FF80);
    assert_eq!(m.read_register(4), 0x0000_0080);
}

#[test]
fn word_access_crosses_ram_block_boundaries() {
    let mut m = Machine::new();
    let addr = ByteAddress(0x0000_FFFE);

    m.write_word(addr, 0x1234_5678);

    assert_eq!(m.read_word(addr), 0x1234_5678);
    assert_eq!(m.read_byte(ByteAddress(0x0000_FFFE)), 0x12);
    assert_eq!(m.read_byte(ByteAddress(0x0000_FFFF)), 0x34);
    assert_eq!(m.read_byte(ByteAddress(0x0001_0000)), 0x56);
    assert_eq!(m.read_byte(ByteAddress(0x0001_0001)), 0x78);
}

#[test]
fn image_chunk_round_trip_preserves_sparse_memory() {
    let hello_addr = ByteAddress(0x0000_F000);
    let hello = b"Hello, Sailor!\n";
    let instructions = [
        is::xori(1, 0, 1),
        is::xori(2, 0, 1),
        is::xori(3, 0, hello_addr.0 as u16),
        is::xori(4, 0, hello.len() as u16),
        is::sys(),
        Instruction::HALT,
    ];

    let mut image = Image::new();
    image.override_with_instructions(instructions.as_slice());
    image.write_bytes(hello_addr, hello);

    let mut chunk_bytes = Vec::new();
    image.dump_chunks(&mut chunk_bytes).unwrap();
    assert!(chunk_bytes.len() < hello_addr.0 as usize);

    let rebuilt = Image::from_chunk_bytes(chunk_bytes.as_slice()).unwrap();
    let mut machine = rebuilt.branch_to_machine();

    assert_eq!(
        machine.read_word(ByteAddress::ZERO),
        instructions[0].encode()
    );
    assert_eq!(machine.read_word(ByteAddress(4)), instructions[1].encode());
    let mut loaded_hello = [0u8; 15];
    machine.mem.read_raw_bytes(hello_addr, &mut loaded_hello);
    assert_eq!(loaded_hello, *hello);
}

#[test]
fn image_copy_into_machine_keeps_original_memory() {
    let addr = ByteAddress(0x0000_2000);
    let mut image = Image::new();
    image.write_word(addr, 0x1234_5678);

    let mut machine_a = image.branch_to_machine();
    machine_a.write_word(addr, 0xAABB_CCDD);

    let mut machine_b = image.branch_to_machine();

    assert_eq!(machine_a.read_word(addr), 0xAABB_CCDD);
    assert_eq!(machine_b.read_word(addr), 0x1234_5678);
}

#[test]
fn program_fibonacci() {
    let instructions = &[
        /* 0 */ is::addi(1, 0, 0), // r1 = 0
        /* 1 */ is::addi(2, 0, 1), // r2 = 1
        /* 2 */ is::addi(3, 0, 0), // r3 = 0 (counter)
        /* 3 */ is::addi(4, 0, 7), // r4 = 7 (iterations for fib(8))
        /* 4 */ is::add(5, 1, 2), // r5 = r1 + r2
        /* 5 */ is::addi(1, 2, 0), // r1 = r2 (addi r1, r2, 0)
        /* 6 */ is::addi(2, 5, 0), // r2 = r5
        /* 7 */ is::addi(3, 3, 1), // r3++
        /* 8 */ is::bne(3, 4, -5), // if r3 != 7, loop back to addr 4
        /* 9 */ is::jmp(0, 0),
    ];

    let mut m = Machine::from_instructions(instructions);
    m.exec_while_not_halt().unwrap();
    assert_eq!(m.read_register(2), 21);
}

struct TestControllerState {
    bytes: RefCell<Box<[u8; BLOCK_SIZE]>>,
    reads: Cell<u32>,
    writes: Cell<u32>,
}

struct TestController {
    state: Rc<TestControllerState>,
}

impl IoController for TestController {
    fn read(&mut self, _mem: &mut Memory, addr: ByteAddress, width: Width) -> u32 {
        self.state.reads.set(self.state.reads.get() + 1);
        let (_, offset) = addr.into_block_parts();
        let offset = usize::from(offset);
        let bytes = self.state.bytes.borrow();
        match width {
            Width::Byte => bytes[offset] as u32,
            Width::Halfword => u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as u32,
            Width::Word => u32::from_be_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]),
        }
    }

    fn write(&mut self, _mem: &mut Memory, addr: ByteAddress, width: Width, data: u32) {
        self.state.writes.set(self.state.writes.get() + 1);
        let (_, offset) = addr.into_block_parts();
        let offset = usize::from(offset);
        let mut bytes = self.state.bytes.borrow_mut();
        match width {
            Width::Byte => bytes[offset] = data as u8,
            Width::Halfword => {
                let data = (data as u16).to_be_bytes();
                bytes[offset] = data[0];
                bytes[offset + 1] = data[1];
            }
            Width::Word => {
                let data = data.to_be_bytes();
                bytes[offset] = data[0];
                bytes[offset + 1] = data[1];
                bytes[offset + 2] = data[2];
                bytes[offset + 3] = data[3];
            }
        }
    }
}

fn new_test_controller() -> (TestController, Rc<TestControllerState>) {
    let state = Rc::new(TestControllerState {
        bytes: RefCell::new(Box::new([0; BLOCK_SIZE])),
        reads: Cell::new(0),
        writes: Cell::new(0),
    });
    (
        TestController {
            state: Rc::clone(&state),
        },
        state,
    )
}

struct TestSystemCallState {
    invocations: Cell<u32>,
    observed_arg: Cell<u32>,
}

struct TestSystemCall {
    state: Rc<TestSystemCallState>,
}

impl SystemCall for TestSystemCall {
    fn invoke(&mut self, mem: &mut Memory, regs: &mut Registers) {
        self.state.invocations.set(self.state.invocations.get() + 1);

        let ptr = ByteAddress(regs.read(2));
        let word = mem.read_raw_word(ptr);
        self.state.observed_arg.set(word);

        regs.write(0, u32::MAX);
        regs.write(1, 0);
        regs.write(4, word);
    }
}

fn new_test_system_call() -> (TestSystemCall, Rc<TestSystemCallState>) {
    let state = Rc::new(TestSystemCallState {
        invocations: Cell::new(0),
        observed_arg: Cell::new(0),
    });
    (
        TestSystemCall {
            state: Rc::clone(&state),
        },
        state,
    )
}

#[test]
fn with_controller_skips_non_empty_io_window_blocks() {
    let (controller, _) = new_test_controller();
    let mut m = Machine::new();
    m.write_byte(ByteAddress(IO_BEGINNING), 0xAA);

    let addr = m.attach_io_controller(None, controller).unwrap();

    assert!(!matches!(
        m.mem.block_from_addr(ByteAddress(IO_BEGINNING)).0,
        Block::Io
    ));
    assert_eq!(m.read_byte(ByteAddress(IO_BEGINNING)), 0xAA);
    assert!(matches!(m.mem.block_from_addr(addr).0, Block::Io));
}

#[test]
fn with_controller_returns_error_when_no_io_slots_remain() {
    let mut m = Machine::new();
    let (start_block, _) = ByteAddress(IO_BEGINNING).into_block_parts();
    for index in usize::from(start_block)..BLOCK_COUNT {
        m.mem.blocks[index] = Block::with_io();
    }

    let (controller, _) = new_test_controller();
    match m.attach_io_controller(None, controller) {
        Some(_) => panic!("expected controller attachment to fail"),
        None => {}
    }
}

#[test]
fn with_controller_uses_desired_address() {
    let (controller, _) = new_test_controller();
    let mut m = Machine::new();
    let desired_addr = ByteAddress(IO_BEGINNING + 2 * BLOCK_SIZE as u32);

    let addr = m
        .attach_io_controller(Some(desired_addr), controller)
        .unwrap();

    assert_eq!(addr, desired_addr);
    assert!(matches!(m.mem.block_from_addr(desired_addr).0, Block::Io));
}

#[test]
fn attach_system_call_rejects_duplicate_number() {
    let (system_call_a, _) = new_test_system_call();
    let (system_call_b, _) = new_test_system_call();
    let mut m = Machine::new();

    assert_eq!(m.attach_system_call(7, system_call_a), Some(7));
    assert_eq!(m.attach_system_call(7, system_call_b), None);
}

#[test]
fn system_call_dispatches_and_preserves_r0() {
    let instructions = [is::sys(), Instruction::HALT];
    let (system_call, state) = new_test_system_call();
    let mut m = Machine::from_instructions(instructions.as_slice());

    assert_eq!(m.attach_system_call(7, system_call), Some(7));
    m.write_word(ByteAddress(0x2_0000), 0x1234_5678);
    m.write_register(1, 7);
    m.write_register(2, 0x2_0000);

    m.exec_while_not_halt().unwrap();

    assert_eq!(state.invocations.get(), 1);
    assert_eq!(state.observed_arg.get(), 0x1234_5678);
    assert_eq!(m.read_register(0), 0);
    assert_eq!(m.read_register(1), 0);
    assert_eq!(m.read_register(4), 0x1234_5678);
}

#[test]
fn unknown_system_call_sets_error_code() {
    let instructions = [is::sys(), Instruction::HALT];
    let mut m = Machine::from_instructions(instructions.as_slice());
    m.write_register(1, 99);

    m.exec_while_not_halt().unwrap();

    assert_eq!(m.read_register(1), 1);
}

#[test]
#[should_panic(expected = "MMIO access crossed RAM/IO boundary")]
fn word_load_spanning_ram_and_io_panics() {
    let instructions = [is::ldw(1, 2, 0), Instruction::HALT];
    let (controller, state) = new_test_controller();
    state.bytes.borrow_mut()[0] = 0xBB;
    state.bytes.borrow_mut()[1] = 0xCC;
    state.bytes.borrow_mut()[2] = 0xDD;

    let mut m = Machine::from_instructions(instructions.as_slice());
    let addr = m.attach_io_controller(None, controller).unwrap();

    let addr_minus_1 = addr.overflowing_add_bytes(ByteOffset(-1)).0;
    m.write_byte(addr_minus_1, 0xAA);
    m.write_register(2, addr_minus_1.0);

    m.exec_while_not_halt().unwrap();
    let _ = state;
}

#[test]
#[should_panic(expected = "MMIO access crossed controller boundaries")]
fn word_load_spanning_two_io_blocks_panics() {
    let instructions = [is::ldw(1, 2, 0), Instruction::HALT];
    let (controller_a, state_a) = new_test_controller();
    let (controller_b, state_b) = new_test_controller();
    state_a.bytes.borrow_mut()[BLOCK_SIZE - 1] = 0x11;
    state_b.bytes.borrow_mut()[0] = 0x22;
    state_b.bytes.borrow_mut()[1] = 0x33;
    state_b.bytes.borrow_mut()[2] = 0x44;

    let mut m = Machine::from_instructions(instructions.as_slice());
    let addr_a = m.attach_io_controller(None, controller_a).unwrap();
    let addr_b = m.attach_io_controller(None, controller_b).unwrap();
    m.write_register(2, addr_a.0 + BLOCK_SIZE as u32 - 1);

    m.exec_while_not_halt().unwrap();
    let _ = (state_a, state_b, addr_b);
}

#[test]
fn word_load_within_single_io_block_uses_single_read() {
    let instructions = [is::ldw(1, 2, 0), Instruction::HALT];
    let (controller, state) = new_test_controller();
    state.bytes.borrow_mut()[0] = 0x11;
    state.bytes.borrow_mut()[1] = 0x22;
    state.bytes.borrow_mut()[2] = 0x33;
    state.bytes.borrow_mut()[3] = 0x44;

    let mut m = Machine::from_instructions(instructions.as_slice());
    let addr = m.attach_io_controller(None, controller).unwrap();
    m.write_register(2, addr.0);

    m.exec_while_not_halt().unwrap();

    assert_eq!(m.read_register(1), 0x1122_3344);
    assert_eq!(state.reads.get(), 1);
}
