// FIX Remove
#![allow(dead_code)]

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Noop = 0x00,
    // ========== R-types ==========
    //
    /// Signed and unsigned addition.
    Add = 0x01,
    /// Signed and unsigned subtraction.
    Sub = 0x02,
    /// Logical bitshift left.
    Shl = 0x03,
    /// Logical bitshift right.
    Shr = 0x04,
    /// Logical bitwise or.
    Or = 0x05,
    /// Logical bitwise and.
    And = 0x06,
    /// Logical bitwise exclusive-or.
    Xor = 0x07,
    /// Less-than signed comparison.
    Slt = 0x08,
    /// Less-than unsigned comparison
    Sltu = 0x09,
    // ========== I-types ==========
    //
    /// Immediate unsigned value addition.
    Addi = 0x21,
    /// Immediate unsigned value subtraction.
    Subi = 0x22,
    /// Immediate logical bitshift left.
    Shli = 0x23,
    /// Immediate logical bitshift right.
    Shri = 0x24,
    /// Logical bitwise or for lower 16 bits.
    Ori = 0x25,
    /// Logical bitwise or for upper 16 bits.
    Orui = 0x26,
    /// Logical bitwise and for lower 16 bits.
    Andi = 0x27,
    /// Logical bitwise and for upper 16 bits.
    Andui = 0x28,
    /// Logical bitwise exclusive-or for lower 16 bits.
    Xori = 0x29,
    /// Logical bitwise exclusive-or for upper 16 bits.
    Xorui = 0x2a,
    /// Less-than immediate signed comparison.
    Slti = 0x2b,
    /// Less-than immediate unsigned comparison.
    Sltui = 0x2c,
    /// Load word from memory.
    Ldw = 0x31,
    /// Load half-word from memory.
    Ldhw = 0x32,
    /// Load unsigned half-word from memory.
    Ldhwu = 0x33,
    /// Load byte from memory.
    Ldb = 0x34,
    /// Load unsigned byte from memory.
    Ldbu = 0x35,
    /// Store word to memory.
    Stw = 0x36,
    /// Store half-word to memory.
    Sthw = 0x37,
    /// Store byte to memory.
    Stb = 0x38,
    /// Jump and link by offset.
    Jmp = 0x39,
    /// Jump and link relative to register.
    Jmpr = 0x3A,
    /// Branch by offset if equal.
    Beq = 0x3B,
    /// Branch by offset if not equal.
    Bne = 0x3C,
}

#[derive(PartialEq)]
enum Encoding {
    Noop,
    R,
    I,
}

impl Op {
    pub fn name(self) -> &'static str {
        match self {
            Op::Noop => "noop",
            // Rs
            Op::Add => "add",
            Op::Sub => "sub",
            Op::Shl => "shl",
            Op::Shr => "shr",
            Op::Or => "or",
            Op::And => "and",
            Op::Xor => "xor",
            Op::Slt => "slt",
            Op::Sltu => "sltu",
            // Is
            Op::Addi => "add_i",
            Op::Subi => "sub_i",
            Op::Shli => "shl_i",
            Op::Shri => "shr_i",
            Op::Ori => "or_i",
            Op::Orui => "oru_i",
            Op::Andi => "and_i",
            Op::Andui => "andu_i",
            Op::Xori => "xor_i",
            Op::Xorui => "xoru_i",
            Op::Slti => "slt_i",
            Op::Sltui => "sltu_i",
            Op::Ldw => "ldw_i",
            Op::Ldhw => "ldhw_i",
            Op::Ldhwu => "ldhwu_i",
            Op::Ldb => "ldb_i",
            Op::Ldbu => "ldbu_i",
            Op::Stw => "stw_i",
            Op::Sthw => "sthw_i",
            Op::Stb => "stb_i",
            Op::Jmp => "jmp_i",
            Op::Jmpr => "jmpr_i",
            Op::Beq => "beq_i",
            Op::Bne => "bne_i",
        }
    }

    pub const fn opcode(self) -> u8 {
        self as u8
    }

    const fn encoding(self) -> Encoding {
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
    pub const fn encode(&self) -> u32 {
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
    /// ```
    /// +------------------------------------+
    /// |   6  |  5  |  5  |  5  |    11     | == 32
    /// +------------------------------------+
    /// ```
    ///  opcode  rr    ra    rb     packing (not used)
    ///
    /// I-mode:
    /// ```
    /// +------------------------------------+
    /// |   6  |  5  |  5  |       16        | == 32
    /// +------------------------------------+
    /// ```
    ///  opcode  rr    ra       immediate
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

    pub const fn encode(&self) -> u32 {
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

    pub const fn noop() -> Instruction {
        Instruction::NOOP
    }

    // ========== R-types ==========

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

    // ========== I-types ==========

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

    pub fn jmp(rr: usize, ra: usize, imm: i16) -> Instruction {
        Instruction::i_type(Op::Jmp, rr, ra, imm as u16)
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
}

/// A byte-granular address in the machine's flat memory space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteAddress(u32);

/// A signed displacement measured in 32-bit bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteOffset(i32);

/// A signed displacement measured in 32-bit words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WordOffset(i32);

/// Selects one 64KiB memory block within the sparse address space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockIndex(u16);

/// A byte offset within a single 64KiB memory block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockOffset(u16);

impl From<BlockIndex> for usize {
    fn from(val: BlockIndex) -> Self {
        val.0 as usize
    }
}

impl ByteAddress {
    pub const ZERO: Self = Self(0);

    pub fn into_block_parts(&self) -> (BlockIndex, BlockOffset) {
        let index = (self.0 >> 16) as u16;
        let offset = (self.0 & 0xFFFF) as u16;
        (BlockIndex(index), BlockOffset(offset))
    }

    pub const fn from_u32(addr: u32) -> Self {
        Self(addr)
    }

    pub const fn as_u32(self) -> u32 {
        self.0
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
        self.overflowing_add_bytes(ByteOffset::byte_offset(WORD_SIZE_BYTES as i32))
    }
}

impl ByteOffset {
    pub const fn byte_offset(offset: i32) -> Self {
        Self(offset)
    }

    pub const fn from_immediate(immediate: u16) -> Self {
        Self(immediate as i16 as i32)
    }
}

impl WordOffset {
    pub const fn from_immediate(immediate: u16) -> Self {
        Self(immediate as i16 as i32)
    }
}

impl From<BlockOffset> for usize {
    fn from(val: BlockOffset) -> Self {
        val.0 as usize
    }
}

impl From<ByteAddress> for u32 {
    fn from(val: ByteAddress) -> Self {
        val.as_u32()
    }
}

impl Block {
    pub fn new_memory() -> Block {
        Block::Memory(Box::new([0u8; BLOCK_SIZE]))
    }

    pub fn read_byte(&self, offset: BlockOffset) -> u8 {
        match self {
            Block::Empty => 0,
            Block::Memory(mem) => mem[usize::from(offset)],
        }
    }

    pub fn read_half_word(&self, offset: BlockOffset) -> u16 {
        let bytes = match self {
            Block::Empty => {
                return 0;
            }
            Block::Memory(mem) => {
                let u = usize::from(offset);
                [mem[u], mem[u + 1]]
            }
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
        }
    }

    pub fn write_half_word(&mut self, offset: BlockOffset, word: u16) {
        let bytes = word.to_be_bytes();
        self.write_byte(offset, bytes[0]);
        self.write_byte(BlockOffset(offset.0 + 1), bytes[1]);
    }

    pub fn write_word(&mut self, offset: BlockOffset, word: u32) {
        let bytes = word.to_be_bytes();
        self.write_byte(offset, bytes[0]);
        self.write_byte(BlockOffset(offset.0 + 1), bytes[1]);
        self.write_byte(BlockOffset(offset.0 + 2), bytes[2]);
        self.write_byte(BlockOffset(offset.0 + 3), bytes[3]);
    }
}

/// We intend to map the addresses above the stack to IO.
pub const STACK_BEGINNING: u32 = 0xEFFFFFFF;
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

impl Machine {
    pub fn new() -> Machine {
        let mut m = Machine {
            program_counter: ByteAddress::ZERO,
            regs: [0u32; REGISTER_COUNT],
            blocks: std::iter::repeat_with(|| Block::Empty)
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
        let blocks = m.blocks.as_mut_slice();

        let mut addr = ByteAddress::ZERO;
        for i in instructions {
            let (idx, offset) = addr.into_block_parts();
            blocks[usize::from(idx)].write_word(offset, i.encode());
            addr = addr.next_word().0;
        }

        m
    }

    pub fn restart(&mut self) {
        self.program_counter = ByteAddress(0);
        self.regs.fill(0);
        self.regs[SP_INDEX] = STACK_BEGINNING;
    }

    pub fn reset(&mut self) {
        self.restart();
        self.blocks.fill_with(|| Block::Empty);
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
        if self.program_counter.as_u32() as usize >= BLOCK_COUNT * BLOCK_SIZE {
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
        let (block, offset) = self.block_from_addr(addr);
        block.read_half_word(offset)
    }

    pub fn read_word(&self, addr: ByteAddress) -> u32 {
        let (block, offset) = self.block_from_addr(addr);
        block.read_word(offset)
    }

    pub fn write_byte(&mut self, addr: ByteAddress, data: u8) {
        let (block, offset) = self.block_from_addr_mut(addr);
        block.write_byte(offset, data);
    }

    pub fn write_half_word(&mut self, addr: ByteAddress, data: u16) {
        let (block, offset) = self.block_from_addr_mut(addr);
        block.write_half_word(offset, data);
    }

    pub fn write_word(&mut self, addr: ByteAddress, data: u32) {
        let (block, offset) = self.block_from_addr_mut(addr);
        block.write_word(offset, data);
    }

    pub fn instruction_at(&self, addr: ByteAddress) -> Result<Instruction, InstructionError> {
        let inst = self.read_word(addr);
        Instruction::decode(inst)
    }

    pub fn current_instruction(&self) -> Result<Instruction, InstructionError> {
        self.instruction_at(self.program_counter)
    }

    pub fn execute(&mut self, inst: &Instruction) -> InstructionOutcome {
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
                Some(ret.as_u32())
            }
            Op::Jmpr => {
                let word_offset = WordOffset::from_immediate(imm);
                let (ret, _) = self.program_counter.next_word();
                let (addr, _) = ByteAddress::from_u32(r_a).overflowing_add_words(word_offset);
                self.set_program_counter(addr);
                jumped = true;
                Some(ret.as_u32())
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

        InstructionOutcome { jumped }
    }

    pub fn execute_and_advance(&mut self) -> ExecuteResult {
        let instruction = match self.current_instruction() {
            Ok(inst) => inst,
            Err(err) => {
                self.advance();
                return Err(err);
            }
        };
        let result = self.execute(&instruction);
        if !result.jumped {
            self.advance();
        }
        Ok((instruction, result))
    }

    pub fn execute_while_not_halt(&mut self) -> Result<(), InstructionError> {
        loop {
            let (inst, _) = self.execute_and_advance()?;
            if inst == Instruction::HALT {
                return Ok(());
            }
        }
    }
}

fn main() {
    let m = Machine::new();
    _ = m;
}

#[cfg(test)]
mod tests {
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
            Instruction::jmp(1, 0, 2),
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
}
