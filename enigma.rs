#![no_std]

extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::iter;

#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Noop  = 0x00,
    ///////////////////////////////////////////////////////////////////////////
    // R types
    ///////////////////////////////////////////////////////////////////////////
    Add   = 0x01,
    Sub   = 0x02,
    Shl   = 0x03,
    Shr   = 0x04,
    Or    = 0x05,
    And   = 0x06,
    Xor   = 0x07,
    Slt   = 0x08,
    Sltu  = 0x09,
    ///////////////////////////////////////////////////////////////////////////
    // I types
    ///////////////////////////////////////////////////////////////////////////
    Addi  = 0x21,
    Subi  = 0x22,
    Shli  = 0x23,
    Shri  = 0x24,
    Ori   = 0x25,
    Orui  = 0x26,
    Andi  = 0x27,
    Andui = 0x28,
    Xori  = 0x29,
    Xorui = 0x2a,
    Slti  = 0x2b,
    Sltui = 0x2c,
    Ldw   = 0x31,
    Ldhw  = 0x32,
    Ldhwu = 0x33,
    Ldb   = 0x34,
    Ldbu  = 0x35,
    Stw   = 0x36,
    Sthw  = 0x37,
    Stb   = 0x38,
    Jmp   = 0x39,
    Jmpr  = 0x3A,
    Beq   = 0x3B,
    Bne   = 0x3C,
}

#[derive(PartialEq)]
pub enum Encoding {
    Noop,
    R,
    I,
}

impl Op {
    #[rustfmt::skip]
    pub fn name(self) -> &'static str {
        match self {
            Op::Noop  => "noop",
            ////////////////////////////////////////////////////////////////////
            // R types
            ////////////////////////////////////////////////////////////////////
            Op::Add   => "add",
            Op::Sub   => "sub",
            Op::Shl   => "shl",
            Op::Shr   => "shr",
            Op::Or    => "or",
            Op::And   => "and",
            Op::Xor   => "xor",
            Op::Slt   => "slt",
            Op::Sltu  => "sltu",
            ////////////////////////////////////////////////////////////////////
            // I types
            ////////////////////////////////////////////////////////////////////
            Op::Addi  => "add_i",
            Op::Subi  => "sub_i",
            Op::Shli  => "shl_i",
            Op::Shri  => "shr_i",
            Op::Ori   => "or_i",
            Op::Orui  => "oru_i",
            Op::Andi  => "and_i",
            Op::Andui => "andu_i",
            Op::Xori  => "xor_i",
            Op::Xorui => "xoru_i",
            Op::Slti  => "slt_i",
            Op::Sltui => "sltu_i",
            Op::Ldw   => "ldw_i",
            Op::Ldhw  => "ldhw_i",
            Op::Ldhwu => "ldhwu_i",
            Op::Ldb   => "ldb_i",
            Op::Ldbu  => "ldbu_i",
            Op::Stw   => "stw_i",
            Op::Sthw  => "sthw_i",
            Op::Stb   => "stb_i",
            Op::Jmp   => "jmp_i",
            Op::Jmpr  => "jmpr_i",
            Op::Beq   => "beq_i",
            Op::Bne   => "bne_i",
        }
    }

    pub fn opcode(self) -> u8 {
        self as u8
    }

    pub fn encoding(self) -> Encoding {
        match self.opcode() {
            0 => Encoding::Noop,
            0x01..=0x1F => Encoding::R,
            0x20..=0x3F => Encoding::I,
            _ => panic!("unreachable opcode"),
        }
    }
}

impl TryFrom<u8> for Op {
    type Error = ();

    fn try_from(opcode: u8) -> Result<Self, Self::Error> {
        match opcode {
            0x00 => Ok(Op::Noop),
            // Rs
            0x01 => Ok(Op::Add),
            0x02 => Ok(Op::Sub),
            0x03 => Ok(Op::Shl),
            0x04 => Ok(Op::Shr),
            0x05 => Ok(Op::Or),
            0x06 => Ok(Op::And),
            0x07 => Ok(Op::Xor),
            0x08 => Ok(Op::Slt),
            0x09 => Ok(Op::Sltu),
            // Is
            0x21 => Ok(Op::Addi),
            0x22 => Ok(Op::Subi),
            0x23 => Ok(Op::Shli),
            0x24 => Ok(Op::Shri),
            0x25 => Ok(Op::Ori),
            0x26 => Ok(Op::Orui),
            0x27 => Ok(Op::Andi),
            0x28 => Ok(Op::Andui),
            0x29 => Ok(Op::Xori),
            0x2a => Ok(Op::Xorui),
            0x2b => Ok(Op::Slti),
            0x2c => Ok(Op::Sltui),
            0x31 => Ok(Op::Ldw),
            0x32 => Ok(Op::Ldhw),
            0x33 => Ok(Op::Ldhwu),
            0x34 => Ok(Op::Ldb),
            0x35 => Ok(Op::Ldbu),
            0x36 => Ok(Op::Stw),
            0x37 => Ok(Op::Sthw),
            0x38 => Ok(Op::Stb),
            0x39 => Ok(Op::Jmp),
            0x3A => Ok(Op::Jmpr),
            0x3B => Ok(Op::Beq),
            0x3C => Ok(Op::Bne),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for Op {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        for i in 0u8..u8::MAX {
            let op = match Op::try_from(i) {
                Ok(o) => o,
                Err(_) => continue,
            };

            if s == op.name() {
                return Ok(op);
            }
        }

        Err(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Noop,
    R {
        rr: usize,
        ra: usize,
        rb: usize,
    },
    I {
        rr: usize,
        ra: usize,
        immediate: u16,
    },
}

impl Payload {
    pub fn encode(&self) -> u32 {
        match self {
            Payload::Noop => 0x00000000,
            Payload::R { rr, ra, rb } => {
                ((*rr as u32) << 21) | ((*ra as u32) << 16) | ((*rb as u32) << 11)
            }
            Payload::I { rr, ra, immediate } => {
                ((*rr as u32) << 21) | ((*ra as u32) << 16) | (*immediate as u32)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub op: Op,
    pub payload: Payload,
}

pub const REGISTER_MASK: u32 = 0b11111;
pub const OPCODE_MASK: u32 = 0b111111;
pub const IMMEDIATE_MASK: u32 = 0xFFFF;

#[derive(Debug)]
pub enum InstructionError {
    InvalidOperation { opcode: u8 },
}

impl Instruction {
    pub const HALT: Instruction = Instruction {
        op: Op::Jmp,
        payload: Payload::I {
            rr: 0,
            ra: 0,
            immediate: 0,
        },
    };

    pub const NOOP: Instruction = Instruction {
        op: Op::Noop,
        payload: Payload::Noop,
    };

    pub const OPCODE_OFFSET: usize = 26;

    /// Each instruction is stored as either an R-type (register mode) or
    /// I-type, (immediate mode).
    ///
    /// Below is a bit-representation of the different instruction types.
    ///
    /// R-mode:
    /// ```text
    /// +------------------------------------+
    /// |   6  |  5  |  5  |  5  |    11     | == 32
    /// +------------------------------------+
    ///  opcode  rr    ra    rb     packing (unused)
    ///
    /// ```
    ///
    /// I-mode:
    /// ```text
    /// +------------------------------------+
    /// |   6  |  5  |  5  |       16        | == 32
    /// +------------------------------------+
    ///  opcode  rr    ra       immediate
    /// ```
    pub fn decode(word: u32) -> Result<Instruction, InstructionError> {
        let opcode = ((word >> Instruction::OPCODE_OFFSET) & OPCODE_MASK) as u8;
        let op = Op::try_from(opcode).map_err(|_| InstructionError::InvalidOperation { opcode })?;
        let payload = match op.encoding() {
            Encoding::Noop => Payload::Noop,
            Encoding::R => {
                let rr = ((word >> 21) & REGISTER_MASK) as usize;
                let ra = ((word >> 16) & REGISTER_MASK) as usize;
                let rb = ((word >> 11) & REGISTER_MASK) as usize;
                Payload::R { rr, ra, rb }
            }
            Encoding::I => {
                let rr = ((word >> 21) & REGISTER_MASK) as usize;
                let ra = ((word >> 16) & REGISTER_MASK) as usize;
                let immediate = (word & IMMEDIATE_MASK) as u16;
                Payload::I { rr, ra, immediate }
            }
        };
        Ok(Instruction { op, payload })
    }

    pub fn encode(&self) -> u32 {
        ((self.op.opcode() as u32) << Instruction::OPCODE_OFFSET) | self.payload.encode()
    }

    pub fn r_type(op: Op, rr: usize, ra: usize, rb: usize) -> Instruction {
        let payload = Payload::R { rr, ra, rb };
        Instruction { op, payload }
    }

    pub fn i_type(op: Op, rr: usize, ra: usize, imm: u16) -> Instruction {
        let payload = Payload::I {
            rr,
            ra,
            immediate: imm,
        };
        Instruction { op, payload }
    }

    pub fn noop() -> Instruction {
        Instruction::NOOP
    }

    pub fn add(rr: usize, ra: usize, rb: usize) -> Instruction {
        Instruction::r_type(Op::Add, rr, ra, rb)
    }

    pub fn sub(rr: usize, ra: usize, rb: usize) -> Instruction {
        Instruction::r_type(Op::Sub, rr, ra, rb)
    }

    pub fn shl(rr: usize, ra: usize, rb: usize) -> Instruction {
        Instruction::r_type(Op::Shl, rr, ra, rb)
    }

    pub fn shr(rr: usize, ra: usize, rb: usize) -> Instruction {
        Instruction::r_type(Op::Shr, rr, ra, rb)
    }

    pub fn or(rr: usize, ra: usize, rb: usize) -> Instruction {
        Instruction::r_type(Op::Or, rr, ra, rb)
    }

    pub fn and(rr: usize, ra: usize, rb: usize) -> Instruction {
        Instruction::r_type(Op::And, rr, ra, rb)
    }

    pub fn xor(rr: usize, ra: usize, rb: usize) -> Instruction {
        Instruction::r_type(Op::Xor, rr, ra, rb)
    }

    pub fn slt(rr: usize, ra: usize, rb: usize) -> Instruction {
        Instruction::r_type(Op::Slt, rr, ra, rb)
    }

    pub fn sltu(rr: usize, ra: usize, rb: usize) -> Instruction {
        Instruction::r_type(Op::Sltu, rr, ra, rb)
    }

    pub fn addi(rr: usize, ra: usize, imm: u16) -> Instruction {
        Instruction::i_type(Op::Addi, rr, ra, imm)
    }

    pub fn subi(rr: usize, ra: usize, imm: u16) -> Instruction {
        Instruction::i_type(Op::Subi, rr, ra, imm)
    }

    pub fn shli(rr: usize, ra: usize, imm: u16) -> Instruction {
        Instruction::i_type(Op::Shli, rr, ra, imm)
    }

    pub fn shri(rr: usize, ra: usize, imm: u16) -> Instruction {
        Instruction::i_type(Op::Shri, rr, ra, imm)
    }

    pub fn ori(rr: usize, ra: usize, imm: u16) -> Instruction {
        Instruction::i_type(Op::Ori, rr, ra, imm)
    }

    pub fn orui(rr: usize, ra: usize, imm: u16) -> Instruction {
        Instruction::i_type(Op::Orui, rr, ra, imm)
    }

    pub fn andi(rr: usize, ra: usize, imm: u16) -> Instruction {
        Instruction::i_type(Op::Andi, rr, ra, imm)
    }

    pub fn andui(rr: usize, ra: usize, imm: u16) -> Instruction {
        Instruction::i_type(Op::Andui, rr, ra, imm)
    }

    pub fn xori(rr: usize, ra: usize, imm: u16) -> Instruction {
        Instruction::i_type(Op::Xori, rr, ra, imm)
    }

    pub fn xorui(rr: usize, ra: usize, imm: u16) -> Instruction {
        Instruction::i_type(Op::Xorui, rr, ra, imm)
    }

    pub fn slti(rr: usize, ra: usize, imm: u16) -> Instruction {
        Instruction::i_type(Op::Slti, rr, ra, imm)
    }

    pub fn sltui(rr: usize, ra: usize, imm: u16) -> Instruction {
        Instruction::i_type(Op::Sltui, rr, ra, imm)
    }

    pub fn ldw(rr: usize, ra: usize, imm: i16) -> Instruction {
        Instruction::i_type(Op::Ldw, rr, ra, imm as u16)
    }

    pub fn ldhw(rr: usize, ra: usize, imm: i16) -> Instruction {
        Instruction::i_type(Op::Ldhw, rr, ra, imm as u16)
    }

    pub fn ldhwu(rr: usize, ra: usize, imm: i16) -> Instruction {
        Instruction::i_type(Op::Ldhwu, rr, ra, imm as u16)
    }

    pub fn ldb(rr: usize, ra: usize, imm: i16) -> Instruction {
        Instruction::i_type(Op::Ldb, rr, ra, imm as u16)
    }

    pub fn ldbu(rr: usize, ra: usize, imm: i16) -> Instruction {
        Instruction::i_type(Op::Ldbu, rr, ra, imm as u16)
    }

    pub fn stw(rr: usize, ra: usize, imm: i16) -> Instruction {
        Instruction::i_type(Op::Stw, rr, ra, imm as u16)
    }

    pub fn sthw(rr: usize, ra: usize, imm: i16) -> Instruction {
        Instruction::i_type(Op::Sthw, rr, ra, imm as u16)
    }

    pub fn stb(rr: usize, ra: usize, imm: i16) -> Instruction {
        Instruction::i_type(Op::Stb, rr, ra, imm as u16)
    }

    pub fn jmp(rr: usize, imm: i16) -> Instruction {
        Instruction::i_type(Op::Jmp, rr, 0, imm as u16)
    }

    pub fn jmpr(rr: usize, ra: usize, imm: i16) -> Instruction {
        Instruction::i_type(Op::Jmpr, rr, ra, imm as u16)
    }

    pub fn beq(rr: usize, ra: usize, imm: i16) -> Instruction {
        Instruction::i_type(Op::Beq, rr, ra, imm as u16)
    }

    pub fn bne(rr: usize, ra: usize, imm: i16) -> Instruction {
        Instruction::i_type(Op::Bne, rr, ra, imm as u16)
    }
}

pub trait IoController {
    fn read(&self, offset: BlockOffset) -> u8;
    fn tick(&mut self);
    fn write(&mut self, offset: BlockOffset, data: u8);
}

pub const BLOCK_SIZE: usize = 1 << 16;
pub const BLOCK_COUNT: usize = 1 << 16;
pub const REGISTER_COUNT: usize = 32;
pub const WORD_SIZE_BYTES: u32 = 4;

/// A block of byte-addressed contiguous virtual machine memory.
///
/// Memory for a block is not allocated unless a non-zero
/// value is written within the block.
pub enum Block {
    Empty,
    Memory(Box<[u8; BLOCK_SIZE]>),
    Io(Box<dyn IoController>),
}

/// A byte-granular address in the machine's flat memory space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteAddress(pub u32);

/// A signed displacement measured in 32-bit bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteOffset(pub i32);

/// A signed displacement measured in 32-bit words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WordOffset(pub i32);

/// Selects one 64KiB memory block within the sparse address space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockIndex(pub u16);

/// A byte offset within a single 64KiB memory block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockOffset(pub u16);

impl ByteAddress {
    pub const ZERO: ByteAddress = ByteAddress(0);

    pub fn into_block_parts(&self) -> (BlockIndex, BlockOffset) {
        let index = (self.0 >> 16) as u16;
        let offset = (self.0 & 0xFFFF) as u16;
        (BlockIndex(index), BlockOffset(offset))
    }

    fn wrap_on_overflow(x: i64) -> (ByteAddress, bool) {
        let overflow = !(0..=((u32::MAX) as i64)).contains(&x);
        let wrapped = x.rem_euclid((u32::MAX as i64) + 1) as u32;
        (ByteAddress(wrapped), overflow)
    }

    /// Add some byte offset to a byte address, returning a new byte address
    /// and a [`bool`] indicating overflow.
    pub fn overflowing_add_bytes(self, byte_offset: ByteOffset) -> (ByteAddress, bool) {
        let total = (self.0 as i64) + byte_offset.0 as i64;
        ByteAddress::wrap_on_overflow(total)
    }

    /// Add some word offset to a byte address, returning a new byte address
    /// and a [`bool`] indicating overflow.
    pub fn overflowing_add_words(self, word_offset: WordOffset) -> (ByteAddress, bool) {
        let byte_offset = (word_offset.0 as i64) * (WORD_SIZE_BYTES as i64);
        let total = (self.0 as i64) + byte_offset;
        ByteAddress::wrap_on_overflow(total)
    }

    pub fn next_word(self) -> (ByteAddress, bool) {
        self.overflowing_add_bytes(ByteOffset(WORD_SIZE_BYTES as i32))
    }

    pub fn next_block(self) -> Option<ByteAddress> {
        let aligned = self.0 & 0xFFFF0000;
        aligned.checked_add(BLOCK_SIZE as u32).map(ByteAddress)
    }

    pub fn from_block_index(idx: BlockIndex) -> ByteAddress {
        ByteAddress((u32::from(idx.0)) << 16)
    }
}

impl ByteOffset {
    pub fn from_immediate(immediate: u16) -> ByteOffset {
        ByteOffset(immediate as i16 as i32)
    }
}

impl WordOffset {
    pub fn from_immediate(immediate: u16) -> WordOffset {
        WordOffset(immediate as i16 as i32)
    }
}

impl BlockIndex {
    pub fn to_byte_addr(self) -> ByteAddress {
        ByteAddress::from_block_index(self)
    }
}

impl BlockOffset {
    pub fn next(&self) -> BlockOffset {
        BlockOffset(self.0 + 1)
    }
}

impl From<BlockIndex> for usize {
    fn from(idx: BlockIndex) -> usize {
        idx.0 as usize
    }
}

impl From<BlockOffset> for usize {
    fn from(offset: BlockOffset) -> usize {
        offset.0 as usize
    }
}

impl Block {
    pub fn with_data(bytes: Box<[u8; BLOCK_SIZE]>) -> Block {
        Block::Memory(bytes)
    }

    pub fn with_controller(controller: impl IoController + 'static) -> Block {
        Block::Io(Box::new(controller))
    }

    pub fn new_memory() -> Block {
        Block::with_data(Box::new([0; BLOCK_SIZE]))
    }

    pub fn read_byte(&self, offset: BlockOffset) -> u8 {
        match self {
            Block::Empty => 0,
            Block::Memory(mem) => mem[usize::from(offset)],
            Block::Io(con) => con.read(offset),
        }
    }

    pub fn read_half_word(&self, offset: BlockOffset) -> u16 {
        let bytes = match self {
            Block::Empty => return 0,
            Block::Memory(mem) => {
                let u = usize::from(offset);
                [mem[u], mem[u + 1]]
            }
            Block::Io(con) => [con.read(offset), con.read(offset.next())],
        };
        u16::from_be_bytes(bytes)
    }

    pub fn read_word(&self, offset: BlockOffset) -> u32 {
        let bytes = match self {
            Block::Empty => {
                return 0;
            }
            Block::Memory(mem) => {
                let u = usize::from(offset);
                [mem[u], mem[u + 1], mem[u + 2], mem[u + 3]]
            }
            Block::Io(con) => {
                let u0 = offset;
                let u1 = u0.next();
                let u2 = u1.next();
                let u3 = u2.next();
                [con.read(u0), con.read(u1), con.read(u2), con.read(u3)]
            }
        };
        u32::from_be_bytes(bytes)
    }

    pub fn write_byte(&mut self, offset: BlockOffset, byte: u8) {
        match self {
            Block::Empty => {
                *self = Block::new_memory();
                self.write_byte(offset, byte);
            }
            Block::Memory(mem) => mem[usize::from(offset)] = byte,
            Block::Io(con) => con.write(offset, byte),
        }
    }
}

/// We intend to map the addresses above the stack to IO.
pub const STACK_BEGINNING: u32 = 0xEFFFFFFC;
pub const IO_BEGINNING: u32 = 0xF0000000;
pub const SP_INDEX: usize = REGISTER_COUNT - 1;

pub struct Machine {
    program_counter: ByteAddress,
    regs: [u32; REGISTER_COUNT],
    /// We don't allocate the entire 4GB upfront, but instead allocate blocks of
    /// 2^16 bytes (~4kB) at a time.
    blocks: Box<[Block; BLOCK_COUNT]>,
}

/// The side effects of the VM successfully executing a single instruction.
#[derive(Debug, Clone)]
pub struct InstructionOutcome {
    /// Whether the program counter was overwritten (jumped to a location
    /// instead of advancing by 1).
    pub jumped: bool,
}

pub type ExecuteResult = Result<(Instruction, InstructionOutcome), InstructionError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerAttachError {
    NoEmptyIoBlock,
}

impl Machine {
    pub fn new() -> Machine {
        let mut m = Machine {
            program_counter: ByteAddress::ZERO,
            regs: [0u32; REGISTER_COUNT],
            blocks: iter::repeat_with(|| Block::Empty)
                .take(BLOCK_COUNT)
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| ())
                .expect("failed to initialize memory-blocks"),
        };
        m.regs[SP_INDEX] = STACK_BEGINNING;
        m
    }

    pub fn from_instructions(instructions: &[Instruction]) -> Machine {
        let mut m = Machine::new();
        m.override_with_instructions(instructions);
        m
    }

    pub fn override_with_instructions(&mut self, instructions: &[Instruction]) {
        let mut addr = ByteAddress::ZERO;
        for i in instructions {
            self.write_word(addr, i.encode());
            addr = addr.next_word().0;
        }
    }

    pub fn attach_controller(&mut self, con: impl IoController + 'static) -> Option<BlockIndex> {
        let mut addr = ByteAddress(IO_BEGINNING);
        loop {
            if matches!(self.block_from_addr(addr).0, Block::Empty) {
                *self.block_from_addr_mut(addr).0 = Block::with_controller(con);
                let (block_index, _) = addr.into_block_parts();
                return Some(block_index);
            }
            addr = addr.next_block()?;
        }
    }

    pub fn detach_controller(&mut self, block_idx: BlockIndex) -> Option<()> {
        let b = self.block_mut(block_idx);
        *b = Block::Empty;
        Some(())
    }

    pub fn restart(&mut self) {
        self.program_counter = ByteAddress(0);
        self.regs.fill(0);
        self.regs[SP_INDEX] = STACK_BEGINNING;
    }

    pub fn reset(&mut self) {
        self.restart();
        for block in self.blocks.iter_mut() {
            match block {
                Block::Io(_) => {}
                Block::Empty | Block::Memory(_) => *block = Block::Empty,
            }
        }
    }

    pub fn read_reg(&self, index: usize) -> u32 {
        self.regs[index % REGISTER_COUNT]
    }

    pub fn set_reg(&mut self, index: usize, word: u32) {
        let index = index % REGISTER_COUNT;
        if index == 0 {
            return;
        }
        self.regs[index] = word;
    }

    pub fn set_program_counter(&mut self, addr: ByteAddress) {
        self.program_counter = addr;
        if self.program_counter.0 as usize >= BLOCK_COUNT * BLOCK_SIZE {
            panic!("tried to set the program counter to outside of the machine's memory");
        }
    }

    pub fn add_program_counter(&mut self, word_offset: WordOffset) {
        let (addr, _) = self.program_counter.overflowing_add_words(word_offset);
        self.set_program_counter(addr);
    }

    pub fn advance(&mut self) {
        let (addr, _) = self.program_counter.next_word();
        self.set_program_counter(addr);
    }

    pub fn block(&self, block_index: BlockIndex) -> &Block {
        &self.blocks[usize::from(block_index)]
    }

    pub fn block_mut(&mut self, block_index: BlockIndex) -> &mut Block {
        &mut self.blocks[usize::from(block_index)]
    }

    pub fn block_from_addr(&self, addr: ByteAddress) -> (&Block, BlockOffset) {
        let (block_index, block_offset) = addr.into_block_parts();
        (self.block(block_index), block_offset)
    }

    pub fn block_from_addr_mut(&mut self, addr: ByteAddress) -> (&mut Block, BlockOffset) {
        let (block_index, block_offset) = addr.into_block_parts();
        (self.block_mut(block_index), block_offset)
    }

    pub fn read_byte(&self, addr: ByteAddress) -> u8 {
        let (block, offset) = self.block_from_addr(addr);
        block.read_byte(offset)
    }

    pub fn read_half_word(&self, addr: ByteAddress) -> u16 {
        let (addr_1, _) = addr.overflowing_add_bytes(ByteOffset(1));
        let bytes = [self.read_byte(addr), self.read_byte(addr_1)];
        u16::from_be_bytes(bytes)
    }

    pub fn read_word(&self, addr: ByteAddress) -> u32 {
        let (addr_1, _) = addr.overflowing_add_bytes(ByteOffset(1));
        let (addr_2, _) = addr.overflowing_add_bytes(ByteOffset(2));
        let (addr_3, _) = addr.overflowing_add_bytes(ByteOffset(3));
        let bytes = [
            self.read_byte(addr),
            self.read_byte(addr_1),
            self.read_byte(addr_2),
            self.read_byte(addr_3),
        ];
        u32::from_be_bytes(bytes)
    }

    pub fn write_byte(&mut self, addr: ByteAddress, data: u8) {
        let (block, offset) = self.block_from_addr_mut(addr);
        block.write_byte(offset, data);
    }

    pub fn write_half_word(&mut self, addr: ByteAddress, data: u16) {
        let bytes = data.to_be_bytes();
        let (addr_1, _) = addr.overflowing_add_bytes(ByteOffset(1));
        self.write_byte(addr, bytes[0]);
        self.write_byte(addr_1, bytes[1]);
    }

    pub fn write_word(&mut self, addr: ByteAddress, data: u32) {
        let bytes = data.to_be_bytes();
        let (addr_1, _) = addr.overflowing_add_bytes(ByteOffset(1));
        let (addr_2, _) = addr.overflowing_add_bytes(ByteOffset(2));
        let (addr_3, _) = addr.overflowing_add_bytes(ByteOffset(3));
        self.write_byte(addr, bytes[0]);
        self.write_byte(addr_1, bytes[1]);
        self.write_byte(addr_2, bytes[2]);
        self.write_byte(addr_3, bytes[3]);
    }

    pub fn is_io_at_addr(&self, addr: ByteAddress) -> bool {
        let (block_index, _) = addr.into_block_parts();
        match self.blocks[usize::from(block_index)] {
            Block::Empty | Block::Memory(_) => false,
            Block::Io(_) => true,
        }
    }

    fn tick_some_io(&mut self, addr: ByteAddress, len_bytes: usize) {
        let mut prev_block_index = None;
        for i in 0..len_bytes {
            let byte_addr = addr.overflowing_add_bytes(ByteOffset(i as i32)).0;
            let block_index = usize::from(byte_addr.into_block_parts().0);
            if prev_block_index.is_some_and(|p| p == block_index) {
                continue;
            }

            prev_block_index = Some(block_index);
            if let Block::Io(con) = self.block_from_addr_mut(byte_addr).0 {
                con.tick();
            }
        }
    }

    pub fn instruction_at(&self, addr: ByteAddress) -> Result<Instruction, InstructionError> {
        let inst = self.read_word(addr);
        Instruction::decode(inst)
    }

    pub fn current_instruction(&self) -> Result<Instruction, InstructionError> {
        self.instruction_at(self.program_counter)
    }

    pub fn exec(&mut self, inst: &Instruction) -> InstructionOutcome {
        let op = inst.op;

        use Payload::*;
        match inst.payload {
            Noop => InstructionOutcome { jumped: false },
            R { rr, ra, rb } => self.exec_r_type(op, rr, ra, rb),
            I { rr, ra, immediate } => self.exec_i_type(op, rr, ra, immediate),
        }
    }

    const SHIFT_MASK: u32 = 0x1F;

    fn exec_r_type(&mut self, op: Op, rr: usize, ra: usize, rb: usize) -> InstructionOutcome {
        let r_a = self.read_reg(ra);
        let r_b = self.read_reg(rb);

        let result = match op {
            Op::Add => r_a.wrapping_add(r_b),
            Op::Sub => r_a.wrapping_sub(r_b),
            Op::Shl => r_a << (r_b & Self::SHIFT_MASK),
            Op::Shr => r_a >> (r_b & Self::SHIFT_MASK),
            Op::Or => r_a | r_b,
            Op::And => r_a & r_b,
            Op::Xor => r_a ^ r_b,
            Op::Slt => {
                if (r_a as i32) < (r_b as i32) {
                    1
                } else {
                    0
                }
            }
            Op::Sltu => {
                if r_a < r_b {
                    1
                } else {
                    0
                }
            }
            _ => panic!("invalid R-type opcode: {}", op.name()),
        };

        self.set_reg(rr, result);
        InstructionOutcome { jumped: false }
    }

    fn exec_i_type(&mut self, op: Op, rr: usize, ra: usize, imm: u16) -> InstructionOutcome {
        let r_r = self.read_reg(rr);
        let r_a = self.read_reg(ra);
        let mut jumped = false;
        let mut to_tick = None;

        let result = match op {
            Op::Addi => Some(r_a.wrapping_add(imm as u32)),
            Op::Subi => Some(r_a.wrapping_sub(imm as u32)),
            Op::Shli => Some(r_a << (imm & Self::SHIFT_MASK as u16)),
            Op::Shri => Some(r_a >> (imm & Self::SHIFT_MASK as u16)),
            Op::Andi => Some(r_a & (imm as u32 | 0xFFFF0000)),
            Op::Andui => Some(r_a & ((imm as u32) << 16) | 0x0000FFFF),
            Op::Ori => Some(r_a | (imm as u32)),
            Op::Orui => Some(r_a | ((imm as u32) << 16)),
            Op::Xori => Some(r_a ^ (imm as u32)),
            Op::Xorui => Some(r_a ^ ((imm as u32) << 16)),
            Op::Slti => Some(if (r_a as i32) < (imm as i16 as i32) {
                1
            } else {
                0
            }),
            Op::Sltui => Some(if r_a < imm as u32 { 1 } else { 0 }),
            Op::Ldw => {
                let byte_offset = ByteOffset::from_immediate(imm);
                let (addr, _) = ByteAddress(r_a).overflowing_add_bytes(byte_offset);
                to_tick = Some((addr, WORD_SIZE_BYTES));
                Some(self.read_word(addr))
            }
            Op::Ldhw => {
                let byte_offset = ByteOffset::from_immediate(imm);
                let (addr, _) = ByteAddress(r_a).overflowing_add_bytes(byte_offset);
                to_tick = Some((addr, WORD_SIZE_BYTES / 2));
                // Some maneuvering for preserving signedness.
                Some(self.read_half_word(addr) as i16 as i32 as u32)
            }
            Op::Ldhwu => {
                let byte_offset = ByteOffset::from_immediate(imm);
                let (addr, _) = ByteAddress(r_a).overflowing_add_bytes(byte_offset);
                to_tick = Some((addr, WORD_SIZE_BYTES / 2));
                Some(self.read_half_word(addr) as u32)
            }
            Op::Ldb => {
                let byte_offset = ByteOffset::from_immediate(imm);
                let (addr, _) = ByteAddress(r_a).overflowing_add_bytes(byte_offset);
                to_tick = Some((addr, 1));
                // Some maneuvering for preserving signedness.
                Some(self.read_byte(addr) as i8 as i32 as u32)
            }
            Op::Ldbu => {
                let byte_offset = ByteOffset::from_immediate(imm);
                let (addr, _) = ByteAddress(r_a).overflowing_add_bytes(byte_offset);
                to_tick = Some((addr, 1));
                Some(self.read_byte(addr) as u32)
            }
            Op::Stw => {
                let byte_offset = ByteOffset::from_immediate(imm);
                let (addr, _) = ByteAddress(r_a).overflowing_add_bytes(byte_offset);
                self.write_word(addr, r_r);
                to_tick = Some((addr, WORD_SIZE_BYTES));
                None
            }
            Op::Sthw => {
                let byte_offset = ByteOffset::from_immediate(imm);
                let (addr, _) = ByteAddress(r_a).overflowing_add_bytes(byte_offset);
                self.write_half_word(addr, r_r as u16);
                to_tick = Some((addr, WORD_SIZE_BYTES / 2));
                None
            }
            Op::Stb => {
                let byte_offset = ByteOffset::from_immediate(imm);
                let (addr, _) = ByteAddress(r_a).overflowing_add_bytes(byte_offset);
                self.write_byte(addr, r_r as u8);
                to_tick = Some((addr, 1));
                None
            }
            Op::Jmp => {
                let word_offset = WordOffset::from_immediate(imm);
                let (ret, _) = self.program_counter.next_word();
                self.add_program_counter(word_offset);
                jumped = true;
                Some(ret.0)
            }
            Op::Jmpr => {
                let word_offset = WordOffset::from_immediate(imm);
                let (ret, _) = self.program_counter.next_word();
                let (addr, _) = ByteAddress(r_a).overflowing_add_words(word_offset);
                self.set_program_counter(addr);
                jumped = true;
                Some(ret.0)
            }
            Op::Beq => {
                let word_offset = WordOffset::from_immediate(imm);
                if r_r == r_a {
                    self.add_program_counter(word_offset);
                    jumped = true;
                }
                None
            }
            Op::Bne => {
                let word_offset = WordOffset::from_immediate(imm);
                if r_r != r_a {
                    self.add_program_counter(word_offset);
                    jumped = true;
                }
                None
            }
            _ => panic!("invalid I-type opcode: {}", op.name()),
        };

        if let Some(result) = result {
            self.set_reg(rr, result);
        }

        if let Some((addr, len_bytes)) = to_tick {
            self.tick_some_io(addr, len_bytes as usize);
        }

        InstructionOutcome { jumped }
    }

    pub fn exec_and_advance(&mut self) -> ExecuteResult {
        let instruction = match self.current_instruction() {
            Ok(inst) => inst,
            Err(err) => {
                self.advance();
                return Err(err);
            }
        };
        let result = self.exec(&instruction);
        if !result.jumped {
            self.advance();
        }
        Ok((instruction, result))
    }

    pub fn exec_while_not_halt(&mut self) -> Result<(), InstructionError> {
        loop {
            let (inst, _) = self.exec_and_advance()?;
            if inst == Instruction::HALT {
                return Ok(());
            }
        }
    }
}

#[cfg(test)]
mod tests;
