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

enum Block {
    Empty,
    Memory(Box<[u8; BLOCK_SIZE]>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ByteAddress(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockIndex(u16);

impl From<BlockIndex> for usize {
    fn from(val: BlockIndex) -> Self {
        val.0 as usize
    }
}

impl ByteAddress {
    pub fn into_block_parts(&self) -> (BlockIndex, BlockOffset) {
        let index = (self.0 >> 16) as u16;
        let offset = (self.0 & 0xFFFF) as u16;
        (BlockIndex(index), BlockOffset(offset))
    }

    /// Add some word offset to the address, returning a new address
    /// and a [`bool`] indicating overflow.
    pub fn overflowing_add(self, word_offset: u32) -> (ByteAddress, bool) {
        let byte_offset = word_offset << 2;
        let byte_addr = self.0 << 2;
        let (new_byte_addr, overflow) = byte_addr.overflowing_add(byte_offset);
        (ByteAddress(new_byte_addr >> 2), overflow)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct BlockOffset(u16);

impl From<BlockOffset> for usize {
    fn from(val: BlockOffset) -> Self {
        val.0 as usize
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
}

/// We intend to map the addresses above the stack to IO.
pub const STACK_BEGINNING: u32 = 0xEFFFFFFF;
pub const SP_INDEX: usize = REGISTER_COUNT - 1;

struct Machine {
    program_counter: u32,
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
    fn new() -> Machine {
        let mut m = Machine {
            program_counter: 0,
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
        self.program_counter = addr.0;
        if self.program_counter as usize >= BLOCK_COUNT * BLOCK_SIZE {
            panic!("tried to set the program counter to outside of the machine's memory");
        }
    }

    pub fn add_program_counter_signed(&mut self, word_offset: i32) {
        self.program_counter = self.program_counter.overflowing_add_signed(word_offset).0
    }

    pub fn advance(&mut self) {
        self.set_program_counter(ByteAddress(self.program_counter + 1));
    }

    // TODO should this init the block if the index doesn't exist?
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

    pub fn instruction_at(&self, word_addr: ByteAddress) -> Result<Instruction, InstructionError> {
        let inst = self.read_word(word_addr.into());
        Instruction::decode(inst)
    }

    pub fn current_instruction(&self) -> Result<Instruction, InstructionError> {
        self.instruction_at(ByteAddress(self.program_counter))
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

        let result = match op.opcode() {
            Op::ADDI_CODE => Some(r_a.wrapping_add(imm as u32)),
            Op::SUBI_CODE => Some(r_a.wrapping_sub(imm as u32)),
            Op::JMP_CODE => {
                let (ret, _) = ByteAddress(self.program_counter).overflowing_add(1);
                self.add_program_counter_signed(imm as i16 as i32);
                jumped = true;
                Some(ret.0)
            }
            Op::JMPR_CODE => {
                let (ret, _) = ByteAddress(self.program_counter).overflowing_add(1);
                let addr = r_a.wrapping_add_signed(imm as i16 as i32);
                self.set_program_counter(ByteAddress(addr as u32));
                jumped = true;
                Some(ret.0)
            }
            Op::BEQ_CODE => {
                if r_r == r_a {
                    self.add_program_counter_signed(imm as i16 as i32);
                    jumped = true;
                }
                None
            }
            Op::BNE_CODE => {
                if r_r != r_a {
                    self.add_program_counter_signed(imm as i16 as i32);
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
    fn execute_and_advance() {
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
}
