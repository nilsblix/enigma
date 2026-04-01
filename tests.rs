#[cfg(test)]
extern crate std;

use super::*;
use alloc::rc::Rc;
use core::cell::{Cell, RefCell};

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
    m.set_reg(2, 34);
    m.set_reg(3, 35);

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
    m.blocks[0] = Block::Memory(mem);

    let outcome = m.execute_and_advance().unwrap();
    assert!(!outcome.1.jumped);
    assert_eq!(m.read_reg(1), 69);
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
        Instruction::add(1, 2, 3),
        Instruction::beq(1, 4, 2),
        Instruction::noop(),
        Instruction::shl(6, 1, 5),
    ];

    let mut m = Machine::from_instructions(instructions.as_slice());
    m.set_reg(2, 33);
    m.set_reg(3, 34);
    m.set_reg(4, 67);
    m.set_reg(5, 2);

    let outcome = m.execute_and_advance().unwrap();
    assert!(!outcome.1.jumped);
    assert_eq!(m.read_reg(1), 67);

    let outcome = m.execute_and_advance().unwrap();
    assert!(outcome.1.jumped);

    let outcome = m.execute_and_advance().unwrap();
    assert!(!outcome.1.jumped);
    assert_eq!(m.read_reg(6), 268);
}

#[test]
fn jump_and_link_uses_word_offsets() {
    let instructions = [
        Instruction::jmp(1, 2),
        Instruction::noop(),
        Instruction::addi(2, 0, 9),
    ];

    let mut m = Machine::from_instructions(instructions.as_slice());

    let outcome = m.execute_and_advance().unwrap();
    assert!(outcome.1.jumped);
    assert_eq!(m.read_reg(1), WORD_SIZE_BYTES);

    let outcome = m.execute_and_advance().unwrap();
    assert!(!outcome.1.jumped);
    assert_eq!(m.read_reg(2), 9);
}

#[test]
fn register_relative_jump_uses_word_offsets() {
    let instructions = [
        Instruction::jmpr(1, 3, 2),
        Instruction::noop(),
        Instruction::noop(),
        Instruction::addi(2, 0, 11),
    ];

    let mut m = Machine::from_instructions(instructions.as_slice());
    m.set_reg(3, WORD_SIZE_BYTES);

    let outcome = m.execute_and_advance().unwrap();
    assert!(outcome.1.jumped);
    assert_eq!(m.read_reg(1), WORD_SIZE_BYTES);

    let outcome = m.execute_and_advance().unwrap();
    assert!(!outcome.1.jumped);
    assert_eq!(m.read_reg(2), 11);
}

#[test]
fn simple_store_and_load() {
    let is = [
        Instruction::stw(1, 2, 0x100),
        Instruction::sthw(3, 2, 0x104),
        Instruction::stb(4, 2, 0x106),
        Instruction::addi(1, 0, 0),
        Instruction::addi(3, 0, 0),
        Instruction::addi(4, 0, 0),
        Instruction::ldw(5, 2, 0x100),
        Instruction::ldhwu(6, 2, 0x104),
        Instruction::ldbu(7, 2, 0x106),
        Instruction::HALT,
    ];

    let mut m = Machine::from_instructions(is.as_slice());
    m.set_reg(1, 0x1234_5678);
    m.set_reg(2, 0x20000);
    m.set_reg(3, 0xBEEF);
    m.set_reg(4, 0xAB);

    m.execute_while_not_halt().unwrap();

    assert_eq!(m.read_reg(1), 0);
    assert_eq!(m.read_reg(3), 0);
    assert_eq!(m.read_reg(4), 0);
    assert_eq!(m.read_reg(5), 0x1234_5678);
    assert_eq!(m.read_reg(6), 0x0000_BEEF);
    assert_eq!(m.read_reg(7), 0x0000_00AB);

    let (block, word_offset) = m.block_from_addr(ByteAddress(0x20_100));
    assert_eq!(block.read_word(word_offset), 0x1234_5678);

    let (block, half_word_offset) = m.block_from_addr(ByteAddress(0x20_104));
    assert_eq!(block.read_half_word(half_word_offset), 0xBEEF);

    let (block, byte_offset) = m.block_from_addr(ByteAddress(0x20_106));
    assert_eq!(block.read_byte(byte_offset), 0xAB);
}

#[test]
fn signed_and_unsigned_loads_extend_correctly() {
    let is = [
        Instruction::ldhw(1, 10, 0x10),
        Instruction::ldhwu(2, 10, 0x10),
        Instruction::ldb(3, 10, 0x20),
        Instruction::ldbu(4, 10, 0x20),
        Instruction::HALT,
    ];

    let mut m = Machine::from_instructions(is.as_slice());
    m.set_reg(10, 0x20_000);

    m.write_half_word(ByteAddress(0x20_010), 0x8001);
    m.write_byte(ByteAddress(0x20_020), 0x80);

    m.execute_while_not_halt().unwrap();

    assert_eq!(m.read_reg(1), 0xFFFF_8001);
    assert_eq!(m.read_reg(2), 0x0000_8001);
    assert_eq!(m.read_reg(3), 0xFFFF_FF80);
    assert_eq!(m.read_reg(4), 0x0000_0080);
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
fn program_fibonacci() {
    let instructions = &[
        /* 0 */ Instruction::addi(1, 0, 0), // r1 = 0
        /* 1 */ Instruction::addi(2, 0, 1), // r2 = 1
        /* 2 */ Instruction::addi(3, 0, 0), // r3 = 0 (counter)
        /* 3 */ Instruction::addi(4, 0, 7), // r4 = 7 (iterations for fib(8))
        /* 4 */ Instruction::add(5, 1, 2),  // r5 = r1 + r2
        /* 5 */ Instruction::addi(1, 2, 0), // r1 = r2 (addi r1, r2, 0)
        /* 6 */ Instruction::addi(2, 5, 0), // r2 = r5
        /* 7 */ Instruction::addi(3, 3, 1), // r3++
        /* 8 */ Instruction::bne(3, 4, -5), // if r3 != 7, loop back to addr 4
        /* 9 */ Instruction::jmp(0, 0),
    ];

    let mut m = Machine::from_instructions(instructions);
    m.execute_while_not_halt().unwrap();
    assert_eq!(m.read_reg(2), 21);
}

struct TestControllerState {
    bytes: RefCell<Box<[u8; BLOCK_SIZE]>>,
    ticks: Cell<u32>,
}

struct TestController {
    state: Rc<TestControllerState>,
}

struct NullController;

impl IoController for TestController {
    fn read(&self, offset: BlockOffset) -> u8 {
        self.state.bytes.borrow()[usize::from(offset)]
    }

    fn tick(&mut self) {
        self.state.ticks.set(self.state.ticks.get() + 1);
    }

    fn write(&mut self, offset: BlockOffset, data: u8) {
        self.state.bytes.borrow_mut()[usize::from(offset)] = data;
    }
}

impl IoController for NullController {
    fn read(&self, _offset: BlockOffset) -> u8 {
        0
    }
    fn tick(&mut self) {}
    fn write(&mut self, _offset: BlockOffset, _data: u8) {}
}

fn new_test_controller() -> (TestController, Rc<TestControllerState>) {
    let state = Rc::new(TestControllerState {
        bytes: RefCell::new(Box::new([0; BLOCK_SIZE])),
        ticks: Cell::new(0),
    });
    (
        TestController {
            state: Rc::clone(&state),
        },
        state,
    )
}

#[test]
fn reset_preserves_io_controllers() {
    let (controller, state) = new_test_controller();
    state.bytes.borrow_mut()[0] = 0xAB;

    let mut m = Machine::new().with_controller(controller).unwrap();
    m.write_byte(ByteAddress(0x100), 0xCD);

    m.reset();

    assert!(m.is_io_at_addr(ByteAddress(IO_BEGINNING)));
    assert_eq!(m.read_byte(ByteAddress(IO_BEGINNING)), 0xAB);
    assert_eq!(m.read_byte(ByteAddress(0x100)), 0);
}

#[test]
fn with_controller_skips_non_empty_io_window_blocks() {
    let (controller, _) = new_test_controller();
    let mut m = Machine::new();
    m.write_byte(ByteAddress(IO_BEGINNING), 0xAA);

    let m = m.with_controller(controller).unwrap();

    assert!(!m.is_io_at_addr(ByteAddress(IO_BEGINNING)));
    assert_eq!(m.read_byte(ByteAddress(IO_BEGINNING)), 0xAA);
    assert!(m.is_io_at_addr(ByteAddress(IO_BEGINNING + BLOCK_SIZE as u32)));
}

#[test]
fn with_controller_returns_error_when_no_io_slots_remain() {
    let mut m = Machine::new();
    let (start_block, _) = ByteAddress(IO_BEGINNING).into_block_parts();
    for index in usize::from(start_block)..BLOCK_COUNT {
        m.blocks[index] = Block::with_controller(NullController);
    }

    let (controller, _) = new_test_controller();
    match m.with_controller(controller) {
        Ok(_) => panic!("expected controller attachment to fail"),
        Err(err) => assert_eq!(err, ControllerAttachError::NoEmptyIoBlock),
    }
}

#[test]
fn word_load_spanning_ram_and_io_ticks_the_io_controller() {
    let instructions = [Instruction::ldw(1, 2, 0), Instruction::HALT];
    let (controller, state) = new_test_controller();
    state.bytes.borrow_mut()[0] = 0xBB;
    state.bytes.borrow_mut()[1] = 0xCC;
    state.bytes.borrow_mut()[2] = 0xDD;

    let mut m = Machine::from_instructions(instructions.as_slice())
        .with_controller(controller)
        .unwrap();
    m.write_byte(ByteAddress(IO_BEGINNING - 1), 0xAA);
    m.set_reg(2, IO_BEGINNING - 1);

    m.execute_while_not_halt().unwrap();

    assert_eq!(m.read_reg(1), 0xAABB_CCDD);
    assert_eq!(state.ticks.get(), 1);
}

#[test]
fn word_load_spanning_two_io_blocks_ticks_both_controllers() {
    let instructions = [Instruction::ldw(1, 2, 0), Instruction::HALT];
    let (controller_a, state_a) = new_test_controller();
    let (controller_b, state_b) = new_test_controller();
    state_a.bytes.borrow_mut()[BLOCK_SIZE - 1] = 0x11;
    state_b.bytes.borrow_mut()[0] = 0x22;
    state_b.bytes.borrow_mut()[1] = 0x33;
    state_b.bytes.borrow_mut()[2] = 0x44;

    let mut m = Machine::from_instructions(instructions.as_slice())
        .with_controller(controller_a)
        .unwrap()
        .with_controller(controller_b)
        .unwrap();
    m.set_reg(2, IO_BEGINNING + BLOCK_SIZE as u32 - 1);

    m.execute_while_not_halt().unwrap();

    assert_eq!(m.read_reg(1), 0x1122_3344);
    assert_eq!(state_a.ticks.get(), 1);
    assert_eq!(state_b.ticks.get(), 1);
}
