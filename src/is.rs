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
