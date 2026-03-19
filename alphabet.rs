// FIX Remove
#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Op(u8);

enum Encoding {
    Noop,
    R,
    I,
}

impl Op {
    pub const COUNT: usize = 1 << 6;

    pub const NOOP_CODE: u8 = 0x00;
    pub const NOOP: Self = Self(Self::NOOP_CODE);

    pub const ADD_CODE: u8 = 0x01;
    pub const ADD: Self = Self(Self::ADD_CODE);

    pub const SUB_CODE: u8 = 0x02;
    pub const SUB: Self = Self(Self::SUB_CODE);

    pub const SHL_CODE: u8 = 0x03;
    pub const SHL: Self = Self(Self::SHL_CODE);

    pub const SHR_CODE: u8 = 0x04;
    pub const SHR: Self = Self(Self::SHR_CODE);

    pub const ADDI_CODE: u8 = 0x21;
    pub const ADDI: Self = Self(Self::ADDI_CODE);

    pub const SUBI_CODE: u8 = 0x22;
    pub const SUBI: Self = Self(Self::SUBI_CODE);

    /// Jump and link by offset.
    pub const JMP_CODE: u8 = 0x39;
    pub const JMP: Self = Self(Self::JMP_CODE);

    /// Jump and link relative to register.
    pub const JMPR_CODE: u8 = 0x3A;
    pub const JMPR: Self = Self(Self::JMPR_CODE);

    /// Branch by offset if equal.
    pub const BEQ_CODE: u8 = 0x3B;
    pub const BEQ: Self = Self(Self::BEQ_CODE);

    /// Branch by offset if not equal.
    pub const BNE_CODE: u8 = 0x3C;
    pub const BNE: Self = Self(Self::BNE_CODE);

    pub const NAMES: [Option<&'static str>; Op::COUNT] = [
        Some("noop"), // 0x00
        Some("add"),  // 0x01
        Some("sub"),  // 0x02
        Some("shl"),  // 0x03
        Some("shr"),  // 0x04
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("addi"), // 0x21
        Some("subi"), // 0x22
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("jmp"),   // 0x39
        Some("jmpr"),  // 0x3A
        Some("beq"),   // 0x3B
        Some("bne"),   // 0x3C
        None,
        None,
        None,
    ];

    const fn new(opcode: u8) -> Option<Self> {
        if Op::is_valid_opcode(opcode) {
            Some(Op(opcode))
        } else {
            None
        }
    }

    const fn name(&self) -> &'static str {
        Op::NAMES[self.opcode() as usize].expect("invalid op")
    }

    const fn is_valid_opcode(opcode: u8) -> bool {
        let opcode = opcode as usize;
        if opcode >= Op::COUNT {
            return false;
        }
        Op::NAMES[opcode].is_some()
    }

    const fn opcode(&self) -> u8 {
        self.0
    }

    const fn encoding(&self) -> Encoding {
        const COUNT: u8 = Op::COUNT as u8;
        match self.opcode() {
            Op::NOOP_CODE => Encoding::Noop,
            0x01..=0x1F => Encoding::R,
            0x20..=0x3F => Encoding::I,
            COUNT.. => panic!("invalid op"),
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
            Payload::R{rr, ra, rb} =>
                ((*rr as u32) << 21)
                | ((*ra as u32) << 16)
                | ((*rb as u32) << 11),
            Payload::I{rr, ra, immediate} =>
                ((*rr as u32) << 21)
                | ((*ra as u32) << 16)
                | (*immediate as u32)
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
    InvalidOperation {
        opcode: u8,
    }
}

impl Instruction {
    pub const HALT: Instruction = Instruction{
        op: Op::JMP,
        payload: Payload::I{ rr: 0, ra: 0, immediate: 0 },
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
        let op = Op::new(opcode)
            .ok_or(InstructionError::InvalidOperation { opcode })?;
        let payload = match op.encoding() {
            Encoding::Noop => Payload::Noop,
            Encoding::R => {
                let rr = ((word >> 21) & REGISTER_MASK) as usize;
                let ra = ((word >> 16) & REGISTER_MASK) as usize;
                let rb = ((word >> 11) & REGISTER_MASK) as usize;
                Payload::R{ rr, ra, rb }
            },
            Encoding::I => {
                let rr = ((word >> 21) & REGISTER_MASK) as usize;
                let ra = ((word >> 16) & REGISTER_MASK) as usize;
                let immediate = (word & IMMEDIATE_MASK) as u16;
                Payload::I{ rr, ra, immediate }
            },
        };
        Ok(Instruction{ op, payload })
    }

    pub const fn encode(&self) -> u32 {
        ((self.op.opcode() as u32) << Instruction::OPCODE_OFFSET) | self.payload.encode()
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

/// An instruction index measured in 32-bit words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WordAddress(u32);

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

    /// Add some word offset to a byte address, returning a new byte address
    /// and a [`bool`] indicating overflow.
    pub fn overflowing_add_words(self, word_offset: WordOffset) -> (ByteAddress, bool) {
        let byte_offset = (word_offset.0 as i64) * (WORD_SIZE_BYTES as i64);
        let total = (self.0 as i64) + byte_offset;
        let overflow = !(0..=((u32::MAX) as i64)).contains(&total);
        let wrapped = total.rem_euclid((u32::MAX as i64) + 1) as u32;
        (ByteAddress(wrapped), overflow)
    }

    pub fn next_word(self) -> (ByteAddress, bool) {
        self.overflowing_add_words(WordOffset::words(1))
    }
}

impl WordAddress {
    pub const fn words(index: u32) -> Self {
        Self(index)
    }

    pub fn as_byte_address(self) -> ByteAddress {
        let (addr, overflow) = self.0.overflowing_mul(WORD_SIZE_BYTES);
        if overflow {
            panic!("word address overflowed");
        }
        ByteAddress(addr)
    }

    pub fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

impl WordOffset {
    pub const fn words(offset: i32) -> Self {
        Self(offset)
    }

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
    pub fn read_word(&self, offset: BlockOffset) -> u32 {
        let bytes = match self {
            Block::Empty => {
                return 0;
            },
            Block::Memory(mem) => {
                let u = usize::from(offset);
                [ mem[u], mem[u + 1], mem[u + 2], mem[u + 3] ]
            },
        };
        u32::from_be_bytes(bytes)
    }

    pub fn set_byte(&mut self, byte: u8, offset: BlockOffset) {
        match self {
            Block::Empty => {
                *self = Block::Memory(Box::new([0u8; BLOCK_SIZE]));
                self.set_byte(byte, offset);
            },
            Block::Memory(mem) => mem[usize::from(offset)] = byte,
        }
    }

    pub fn set_instruction(&mut self, inst: Instruction, offset: BlockOffset) {
        let bytes = inst.encode().to_be_bytes();
        self.set_byte(bytes[0], offset);
        self.set_byte(bytes[1], BlockOffset(offset.0 + 1));
        self.set_byte(bytes[2], BlockOffset(offset.0 + 2));
        self.set_byte(bytes[3], BlockOffset(offset.0 + 3));
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
                .expect("failed to initialize memory-blocks")
        };
        m.regs[SP_INDEX] = STACK_BEGINNING;
        m
    }

    pub fn from_instructions(instructions: &[Instruction]) -> Machine {
        let mut m = Machine::new();
        let blocks = m.blocks.as_mut_slice();

        let mut addr = WordAddress::words(0);
        for i in instructions {
            let (idx, offset) = addr.as_byte_address().into_block_parts();
            blocks[usize::from(idx)].set_instruction(i.clone(), offset);
            addr = addr.next();
        }

        m
    }

    pub fn register(&self, index: usize) -> u32 {
        self.regs[index % REGISTER_COUNT]
    }

    pub fn set_register(&mut self, index: usize, word: u32) {
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

    pub fn block_from_addr(&self, addr: ByteAddress) -> (&Block, BlockOffset) {
        let (block_index, block_offset) = addr.into_block_parts();
        (self.block(block_index), block_offset)
    }

    pub fn read_word(&self, addr: ByteAddress) -> u32 {
        let (block, offset) = self.block_from_addr(addr);
        block.read_word(offset)
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
            R{rr, ra, rb} => self.exec_r_type(op, rr, ra, rb),
            I{rr, ra, immediate} => self.exec_i_type(op, rr, ra, immediate),
        }
    }

    const SHIFT_MASK: u32 = 0x1F;

    pub fn exec_r_type(
        &mut self,
        op: Op,
        rr: usize,
        ra: usize,
        rb: usize,
    ) -> InstructionOutcome {
        let r_a = self.register(ra);
        let r_b = self.register(rb);

        let result = match op.opcode() {
            Op::ADD_CODE => r_a.wrapping_add(r_b),
            Op::SUB_CODE => r_a.wrapping_sub(r_b),
            Op::SHL_CODE => r_a << (r_b & Self::SHIFT_MASK),
            Op::SHR_CODE => r_a >> (r_b & Self::SHIFT_MASK),
            _ => panic!("invalid R-type opcode"),
        };

        self.set_register(rr, result);
        InstructionOutcome { jumped: false }
    }

    pub fn exec_i_type(
        &mut self,
        op: Op,
        rr: usize,
        ra: usize,
        imm: u16
    ) -> InstructionOutcome {
        let r_r = self.register(rr);
        let r_a = self.register(ra);
        let mut jumped = false;
        let word_offset = WordOffset::from_immediate(imm);

        let result = match op.opcode() {
            Op::ADDI_CODE => Some(r_a.wrapping_add(imm as u32)),
            Op::SUBI_CODE => Some(r_a.wrapping_sub(imm as u32)),
            Op::JMP_CODE => {
                let (ret, _) = self.program_counter.next_word();
                self.add_program_counter(word_offset);
                jumped = true;
                Some(ret.as_u32())
            }
            Op::JMPR_CODE => {
                let (ret, _) = self.program_counter.next_word();
                let (addr, _) = ByteAddress::from_u32(r_a).overflowing_add_words(word_offset);
                self.set_program_counter(addr);
                jumped = true;
                Some(ret.as_u32())
            }
            Op::BEQ_CODE => {
                if r_r == r_a {
                    self.add_program_counter(word_offset);
                    jumped = true;
                }
                None
            }
            Op::BNE_CODE => {
                if r_r != r_a {
                    self.add_program_counter(word_offset);
                    jumped = true;
                }
                None
            }
            _ => panic!("invalid I-type opcode"),
        };

        if let Some(result) = result {
            self.set_register(rr, result);
        }

        InstructionOutcome { jumped }
    }

    pub fn execute_and_advance(&mut self) -> ExecuteResult {
        let instruction = match self.current_instruction() {
            Ok(inst) => inst,
            Err(err) => {
                self.advance();
                return Err(err);
            },
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
            op: Op::NOOP,
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
            op: Op::ADD,
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
            op: Op::SUBI,
            payload: Payload::I {
                rr: 16,
                ra: 10,
                immediate: 149,
            },
        };
        let res = Instruction::decode(word).unwrap();
        assert_eq!(res, inst);
        assert_eq!(res.op.name(), "subi");
        assert_eq!(res.encode(), word);

        //          |  op | rr | ra |    immediate   |
        //          |     V    V    V                |
        let word = 0b11100110000010100000000010010101 as u32;
        let inst = Instruction {
            op: Op::JMP,
            payload: Payload::I {
                rr: 16,
                ra: 10,
                immediate: 149,
            },
        };
        let res = Instruction::decode(word).unwrap();
        assert_eq!(res, inst);
        assert_eq!(res.op.name(), "jmp");
        assert_eq!(res.encode(), word);
    }

    #[test]
    fn painfully_written_execute_and_advance() {
        let mut m = Machine::new();
        m.set_register(2, 34);
        m.set_register(3, 35);

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
        assert_eq!(m.register(1), 69);
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
            Instruction{ op: Op::ADD, payload: Payload::R{ rr: 1, ra: 2, rb: 3 }},
            Instruction{ op: Op::BEQ, payload: Payload::I{ rr: 1, ra: 4, immediate: 2 }},
            Instruction{ op: Op::NOOP, payload: Payload::Noop},
            Instruction{ op: Op::SHL, payload: Payload::R{ rr: 6, ra: 1, rb: 5 }},
        ];

        let mut m = Machine::from_instructions(instructions.as_slice());
        m.set_register(2, 33);
        m.set_register(3, 34);
        m.set_register(4, 67);
        m.set_register(5, 2);

        let outcome = m.execute_and_advance().unwrap();
        assert!(!outcome.1.jumped);
        assert_eq!(m.register(1), 67);

        let outcome = m.execute_and_advance().unwrap();
        assert!(outcome.1.jumped);

        let outcome = m.execute_and_advance().unwrap();
        assert!(!outcome.1.jumped);
        assert_eq!(m.register(6), 268);
    }

    #[test]
    fn jump_and_link_uses_word_offsets() {
        let instructions = [
            Instruction{ op: Op::JMP, payload: Payload::I{ rr: 1, ra: 0, immediate: 2 }},
            Instruction{ op: Op::NOOP, payload: Payload::Noop },
            Instruction{ op: Op::ADDI, payload: Payload::I{ rr: 2, ra: 0, immediate: 9 }},
        ];

        let mut m = Machine::from_instructions(instructions.as_slice());

        let outcome = m.execute_and_advance().unwrap();
        assert!(outcome.1.jumped);
        assert_eq!(m.register(1), WordAddress::words(1).as_byte_address().as_u32());

        let outcome = m.execute_and_advance().unwrap();
        assert!(!outcome.1.jumped);
        assert_eq!(m.register(2), 9);
    }

    #[test]
    fn register_relative_jump_uses_word_offsets() {
        let instructions = [
            Instruction{ op: Op::JMPR, payload: Payload::I{ rr: 1, ra: 3, immediate: 2 }},
            Instruction{ op: Op::NOOP, payload: Payload::Noop },
            Instruction{ op: Op::NOOP, payload: Payload::Noop },
            Instruction{ op: Op::ADDI, payload: Payload::I{ rr: 2, ra: 0, immediate: 11 }},
        ];

        let mut m = Machine::from_instructions(instructions.as_slice());
        m.set_register(3, WordAddress::words(1).as_byte_address().as_u32());

        let outcome = m.execute_and_advance().unwrap();
        assert!(outcome.1.jumped);
        assert_eq!(m.register(1), WordAddress::words(1).as_byte_address().as_u32());

        let outcome = m.execute_and_advance().unwrap();
        assert!(!outcome.1.jumped);
        assert_eq!(m.register(2), 11);
    }
}
