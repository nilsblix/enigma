use std::collections::BTreeMap;

pub mod is {
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
        Debu  = 0x1F,
        ///////////////////////////////////////////////////////////////////////////
        // I types
        ///////////////////////////////////////////////////////////////////////////
        Sys   = 0x20,
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
                Op::Debu   => "deb",
                ////////////////////////////////////////////////////////////////////
                // I types
                ////////////////////////////////////////////////////////////////////
                Op::Sys   => "sys",
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
                0x1F => Ok(Op::Debu),
                // Is
                0x20 => Ok(Op::Sys),
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
            let op =
                Op::try_from(opcode).map_err(|_| InstructionError::InvalidOperation { opcode })?;
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

        pub fn debu(rr: usize) -> Instruction {
            Instruction::r_type(Op::Debu, rr, 0, 0)
        }

        pub fn addi(rr: usize, ra: usize, imm: u16) -> Instruction {
            Instruction::i_type(Op::Addi, rr, ra, imm)
        }

        pub fn sys() -> Instruction {
            Instruction::i_type(Op::Sys, 0, 0, 0)
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
}

use is::{Instruction, InstructionError, Op};

pub const BLOCK_SIZE: usize = 1 << 16;
pub const BLOCK_COUNT: usize = 1 << 16;
pub const REGISTER_COUNT: usize = 32;
pub const WORD_SIZE_BYTES: u32 = 4;

pub struct Registers {
    words: [u32; REGISTER_COUNT],
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            words: [0u32; REGISTER_COUNT],
        }
    }

    pub fn read(&self, index: usize) -> u32 {
        self.words[index % REGISTER_COUNT]
    }

    pub fn write(&mut self, index: usize, word: u32) {
        let index = index % REGISTER_COUNT;
        if index == 0 {
            return;
        }
        self.words[index] = word;
    }
}

pub trait IoController {
    /// Some io relies on writing on read, which justifies the mutable read.
    /// Read supports all 3 main reads, i.e: byte, halfword and word. We always
    /// return a u32 even if we read byte or halfword. Makes the interface
    /// simpler.
    fn read(&mut self, mem: &mut Memory, addr: ByteAddress, width: Width) -> u32;
    /// We always supply u32 data, even on byte and halfword write due to
    /// simpler interface.
    fn write(&mut self, mem: &mut Memory, addr: ByteAddress, width: Width, data: u32);
}

pub trait SystemCall {
    fn invoke(&mut self, mem: &mut Memory, regs: &mut Registers);
}

/// A block of byte-addressed contiguous virtual machine memory.
///
/// Memory for a block is not allocated unless a non-zero
/// value is written within the block.
pub enum Block {
    Empty,
    Memory(Box<[u8; BLOCK_SIZE]>),
    /// To get access to a certain block, you will always be able
    /// to calculate the address.
    Io,
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
    pub fn empty() -> Block {
        Block::Empty
    }

    pub fn with_data(data: [u8; BLOCK_SIZE]) -> Block {
        Block::Memory(Box::new(data))
    }

    pub fn with_empty_data() -> Block {
        Block::with_data([0u8; BLOCK_SIZE])
    }

    pub fn with_io() -> Block {
        Block::Io
    }
}

pub struct Memory {
    /// We don't allocate the entire 4GB upfront, but instead allocate blocks of
    /// 2^16 bytes (~4kB) at a time.
    blocks: Box<[Block; BLOCK_COUNT]>,
}

#[rustfmt::skip]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Width {
    Byte     = 1,
    Halfword = 2,
    Word     = 4,
}

impl Memory {
    fn new() -> Memory {
        Memory {
            blocks: Box::new([const { Block::Empty }; BLOCK_COUNT]),
        }
    }

    pub fn reset(&mut self) {
        for block in self.blocks.iter_mut() {
            match block {
                Block::Io => {}
                Block::Empty | Block::Memory(_) => *block = Block::Empty,
            }
        }
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

    pub fn read_raw_byte(&self, addr: ByteAddress) -> u8 {
        let (block, offset) = self.block_from_addr(addr);
        match block {
            Block::Empty => 0,
            Block::Memory(mem) => mem[usize::from(offset)],
            Block::Io => panic!("cannot read raw from io"),
        }
    }

    pub fn read_raw_half_word(&self, addr: ByteAddress) -> u16 {
        let (next_addr, _) = addr.overflowing_add_bytes(ByteOffset(1));
        let bytes = [self.read_raw_byte(addr), self.read_raw_byte(next_addr)];
        u16::from_be_bytes(bytes)
    }

    pub fn read_raw_word(&self, addr: ByteAddress) -> u32 {
        let (addr_1, _) = addr.overflowing_add_bytes(ByteOffset(1));
        let (addr_2, _) = addr.overflowing_add_bytes(ByteOffset(2));
        let (addr_3, _) = addr.overflowing_add_bytes(ByteOffset(3));
        let bytes = [
            self.read_raw_byte(addr),
            self.read_raw_byte(addr_1),
            self.read_raw_byte(addr_2),
            self.read_raw_byte(addr_3),
        ];
        u32::from_be_bytes(bytes)
    }

    /// Justification of u32 is described in the definition of the IoController
    /// interface.
    pub fn read_io(
        &mut self,
        io: &mut Box<dyn IoController>,
        addr: ByteAddress,
        width: Width,
    ) -> u32 {
        io.read(self, addr, width)
    }

    pub fn read_raw_bytes(&self, ptr: ByteAddress, buf: &mut [u8]) {
        for i in 0..buf.len() {
            let (addr, _) = ptr.overflowing_add_bytes(ByteOffset(i as i32));
            buf[i] = self.read_raw_byte(addr);
        }
    }

    pub fn write_raw_byte(&mut self, addr: ByteAddress, data: u8) {
        let (block, offset) = self.block_from_addr_mut(addr);
        match block {
            Block::Empty => {
                *block = Block::with_empty_data();
                self.write_raw_byte(addr, data);
            }
            Block::Memory(mem) => mem[usize::from(offset)] = data,
            Block::Io => panic!("cannot write raw byte to io"),
        }
    }

    pub fn write_raw_half_word(&mut self, addr: ByteAddress, data: u16) {
        let (next_addr, _) = addr.overflowing_add_bytes(ByteOffset(1));
        let bytes = data.to_be_bytes();
        self.write_raw_byte(addr, bytes[0]);
        self.write_raw_byte(next_addr, bytes[1]);
    }

    pub fn write_raw_word(&mut self, addr: ByteAddress, data: u32) {
        let (addr_1, _) = addr.overflowing_add_bytes(ByteOffset(1));
        let (addr_2, _) = addr.overflowing_add_bytes(ByteOffset(2));
        let (addr_3, _) = addr.overflowing_add_bytes(ByteOffset(3));
        let bytes = data.to_be_bytes();
        self.write_raw_byte(addr, bytes[0]);
        self.write_raw_byte(addr_1, bytes[1]);
        self.write_raw_byte(addr_2, bytes[2]);
        self.write_raw_byte(addr_3, bytes[3]);
    }

    pub fn write_io(
        &mut self,
        io: &mut Box<dyn IoController>,
        addr: ByteAddress,
        width: Width,
        data: u32,
    ) {
        io.write(self, addr, width, data);
    }
}

/// We intend to map the addresses above the stack to IO.
///
/// Note that the stack beginning should be divisible by the word-size, which
/// makes sense, i guess?
pub const STACK_BEGINNING: u32 = 0xEFFFFFFC;
pub const IO_BEGINNING: u32 = 0xF0000000;
pub const SP_INDEX: usize = REGISTER_COUNT - 1;

pub struct Machine {
    program_counter: ByteAddress,
    regs: Registers,
    mem: Memory,
    /// Each io gets mapped to the range between a base address + 2^16 bytes,
    /// i.e some BlockIndex.
    ios: BTreeMap<BlockIndex, Box<dyn IoController>>,
    sys: BTreeMap<u32, Box<dyn SystemCall>>,
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
            regs: Registers::new(),
            mem: Memory::new(),
            ios: BTreeMap::new(),
            sys: BTreeMap::new(),
        };
        m.regs.write(SP_INDEX, STACK_BEGINNING);
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

    fn attach_io_controller_at(
        &mut self,
        addr: ByteAddress,
        io: Box<dyn IoController>,
    ) -> Option<ByteAddress> {
        let (block_index, _) = addr.into_block_parts();
        if !matches!(self.mem.block(block_index), Block::Empty) {
            return None;
        }

        *self.mem.block_mut(block_index) = Block::with_io();
        self.ios.insert(block_index, io);
        Some(block_index.to_byte_addr())
    }

    pub fn attach_io_controller(
        &mut self,
        desired_address: Option<ByteAddress>,
        io: impl IoController + 'static,
    ) -> Option<ByteAddress> {
        let mut io = Some(Box::new(io) as Box<dyn IoController>);

        if let Some(addr) = desired_address {
            return self.attach_io_controller_at(addr, io.take().unwrap());
        }

        let mut addr = ByteAddress(IO_BEGINNING);
        loop {
            if matches!(self.mem.block_from_addr(addr).0, Block::Empty) {
                return self.attach_io_controller_at(addr, io.take().unwrap());
            }

            // This breaks out when no next block exists.
            addr = addr.next_block()?;
        }
    }

    pub fn attach_system_call(
        &mut self,
        desired_call_number: u32,
        system_call: impl SystemCall + 'static,
    ) -> Option<u32> {
        if self.sys.contains_key(&desired_call_number) {
            return None;
        }

        self.sys.insert(desired_call_number, Box::new(system_call));
        Some(desired_call_number)
    }

    pub fn detach_io_controller(&mut self, block_idx: BlockIndex) -> Option<()> {
        self.ios.remove(&block_idx).map(|_| ())?;
        *self.mem.block_mut(block_idx) = Block::Empty;
        Some(())
    }

    pub fn read_register(&self, index: usize) -> u32 {
        self.regs.read(index)
    }

    pub fn write_register(&mut self, index: usize, word: u32) {
        self.regs.write(index, word);
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

    pub fn io_block_index_in_span(
        &self,
        addr: ByteAddress,
        len_bytes: usize,
    ) -> Option<ByteAddress> {
        let mut io_block_index = None;
        let mut touched_ram = false;
        for i in 0..len_bytes {
            let (byte_addr, _) = addr.overflowing_add_bytes(ByteOffset(i as i32));
            match self.mem.block_from_addr(byte_addr).0 {
                Block::Io => {
                    let (block_index, _) = byte_addr.into_block_parts();

                    assert!(
                        !touched_ram,
                        "MMIO access crossed RAM/IO boundary at {:#010X}",
                        addr.0
                    );
                    if let Some(existing) = io_block_index {
                        assert_eq!(
                            existing, block_index,
                            "MMIO access crossed controller boundaries at {:#010X}",
                            addr.0
                        );
                    } else {
                        io_block_index = Some(block_index);
                    }
                }
                Block::Empty | Block::Memory(_) => {
                    touched_ram = true;
                    assert!(
                        io_block_index.is_none(),
                        "MMIO access crossed RAM/IO boundary at {:#010X}",
                        addr.0
                    );
                }
            }
        }

        io_block_index.map(|block_index| block_index.to_byte_addr())
    }

    pub fn read_io(&mut self, addr: ByteAddress, width: Width) -> u32 {
        let (block_index, _) = addr.into_block_parts();
        let io = self
            .ios
            .get_mut(&block_index)
            .expect("memory contains reference to invalid io");
        self.mem.read_io(io, addr, width)
    }

    pub fn read_byte(&mut self, addr: ByteAddress) -> u8 {
        match self.io_block_index_in_span(addr, 1) {
            Some(_) => self.read_io(addr, Width::Byte) as u8,
            None => self.mem.read_raw_byte(addr),
        }
    }

    pub fn read_half_word(&mut self, addr: ByteAddress) -> u16 {
        match self.io_block_index_in_span(addr, 2) {
            Some(_) => self.read_io(addr, Width::Halfword) as u16,
            None => self.mem.read_raw_half_word(addr),
        }
    }

    pub fn read_word(&mut self, addr: ByteAddress) -> u32 {
        match self.io_block_index_in_span(addr, 4) {
            Some(_) => self.read_io(addr, Width::Word) as u32,
            None => self.mem.read_raw_word(addr),
        }
    }

    pub fn write_io(&mut self, addr: ByteAddress, width: Width, data: u32) {
        let (block_index, _) = addr.into_block_parts();
        let io = self
            .ios
            .get_mut(&block_index)
            .expect("memory contains reference to invalid io");
        self.mem.write_io(io, addr, width, data);
    }

    pub fn write_byte(&mut self, addr: ByteAddress, data: u8) {
        match self.io_block_index_in_span(addr, 1) {
            Some(_) => self.write_io(addr, Width::Byte, data as u32),
            None => self.mem.write_raw_byte(addr, data),
        }
    }

    pub fn write_half_word(&mut self, addr: ByteAddress, data: u16) {
        match self.io_block_index_in_span(addr, 2) {
            Some(_) => self.write_io(addr, Width::Halfword, data as u32),
            None => self.mem.write_raw_half_word(addr, data),
        }
    }

    pub fn write_word(&mut self, addr: ByteAddress, data: u32) {
        match self.io_block_index_in_span(addr, 4) {
            Some(_) => self.write_io(addr, Width::Word, data as u32),
            None => self.mem.write_raw_word(addr, data),
        }
    }

    pub fn instruction_at(&mut self, addr: ByteAddress) -> Result<Instruction, InstructionError> {
        let inst = self.read_word(addr);
        Instruction::decode(inst)
    }

    pub fn current_instruction(&mut self) -> Result<Instruction, InstructionError> {
        self.instruction_at(self.program_counter)
    }

    pub fn exec(&mut self, inst: &Instruction) -> InstructionOutcome {
        let op = inst.op;

        use is::Payload::*;
        match inst.payload {
            Noop => InstructionOutcome { jumped: false },
            R { rr, ra, rb } => self.exec_r_type(op, rr, ra, rb),
            I { rr, ra, immediate } => self.exec_i_type(op, rr, ra, immediate),
        }
    }

    const SHIFT_MASK: u32 = 0x1F;

    fn exec_r_type(&mut self, op: Op, rr: usize, ra: usize, rb: usize) -> InstructionOutcome {
        let r_a = self.read_register(ra);
        let r_b = self.read_register(rb);

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
            Op::Debu => {
                let r_r = self.read_register(rr);
                println!("DEBUG: r{rr} = {r_r}");
                r_r
            }
            _ => panic!("invalid R-type opcode: {}", op.name()),
        };

        self.write_register(rr, result);
        InstructionOutcome { jumped: false }
    }

    fn exec_i_type(&mut self, op: Op, rr: usize, ra: usize, imm: u16) -> InstructionOutcome {
        let r_r = self.read_register(rr);
        let r_a = self.read_register(ra);
        let mut jumped = false;

        let result = match op {
            Op::Sys => {
                // r1 is the syscall number.
                let nr = self.read_register(1);
                if let Some(system_call) = self.sys.get_mut(&nr) {
                    system_call.invoke(&mut self.mem, &mut self.regs);
                } else {
                    // We return non-zero on error. In this case, 1 signifies
                    // unknown system call number.
                    self.write_register(1, 1);
                }

                None
            }
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
                Some(self.read_word(addr))
            }
            Op::Ldhw => {
                let byte_offset = ByteOffset::from_immediate(imm);
                let (addr, _) = ByteAddress(r_a).overflowing_add_bytes(byte_offset);
                // Some maneuvering for preserving signedness.
                Some(self.read_half_word(addr) as i16 as i32 as u32)
            }
            Op::Ldhwu => {
                let byte_offset = ByteOffset::from_immediate(imm);
                let (addr, _) = ByteAddress(r_a).overflowing_add_bytes(byte_offset);
                Some(self.read_half_word(addr) as u32)
            }
            Op::Ldb => {
                let byte_offset = ByteOffset::from_immediate(imm);
                let (addr, _) = ByteAddress(r_a).overflowing_add_bytes(byte_offset);
                // Some maneuvering for preserving signedness.
                Some(self.read_byte(addr) as i8 as i32 as u32)
            }
            Op::Ldbu => {
                let byte_offset = ByteOffset::from_immediate(imm);
                let (addr, _) = ByteAddress(r_a).overflowing_add_bytes(byte_offset);
                Some(self.read_byte(addr) as u32)
            }
            Op::Stw => {
                let byte_offset = ByteOffset::from_immediate(imm);
                let (addr, _) = ByteAddress(r_a).overflowing_add_bytes(byte_offset);
                self.write_word(addr, r_r);
                None
            }
            Op::Sthw => {
                let byte_offset = ByteOffset::from_immediate(imm);
                let (addr, _) = ByteAddress(r_a).overflowing_add_bytes(byte_offset);
                self.write_half_word(addr, r_r as u16);
                None
            }
            Op::Stb => {
                let byte_offset = ByteOffset::from_immediate(imm);
                let (addr, _) = ByteAddress(r_a).overflowing_add_bytes(byte_offset);
                self.write_byte(addr, r_r as u8);
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
            self.write_register(rr, result);
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
