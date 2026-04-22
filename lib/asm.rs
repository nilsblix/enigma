use std::fmt;

use crate::{
    ByteAddress,
    Op,
    WordOffset,
    image::Image,
    is::{self, Instruction}
};

#[derive(Debug, Clone)]
pub struct Diagnostic {
    msg: String,
    row: usize,
    col: usize,
}


impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: error: {}", self.row, self.col, self.msg)
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostics {
    diags: Vec<Diagnostic>,
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diag in self.diags.iter() {
            write!(f, "{diag}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostics {}

struct Lexer<'a> {
    src: &'a str,
    idents: Vec<(usize, &'a str)>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Lexer<'a> {
        let mut idents = vec![];
        let mut start = None;

        for (i, c) in src.char_indices() {
            if start.is_none() && !c.is_whitespace() {
                start = Some(i);
                continue;
            }

            if let Some(st) = start {
                if c.is_whitespace() {
                    idents.push((st, &src[st..i]));
                    start = None;
                }
            }
        }

        if let Some(st) = start {
            idents.push((st, &src[st..]));
        }

        Lexer { src, idents }
    }

    fn lines(&self) -> Vec<Vec<(usize, &str)>> {
        let mut lines = vec![];
        let mut i = 0;

        while i < self.idents.len() {
            let start = self.idents[i];
            let (row, _) = idx_to_row_col(self.src, start.0);

            let mut line = vec![start];
            let mut next_i = i + 1;

            while next_i < self.idents.len() {
                let other = self.idents[next_i];
                let (other_row, _) = idx_to_row_col(self.src, other.0);

                if other_row != row {
                    break;
                }

                line.push(other);
                next_i += 1;
            }

            lines.push(line);
            i = next_i;
        }

        lines
    }
}

fn idx_to_row_col(src: &str, idx: usize) -> (usize, usize) {
    let mut r = 1;
    let mut c = 1;

    for ch in src[..idx].chars() {
        if ch == '\n' {
            r += 1;
            c = 0;
            continue;
        }

        c += 1;
    }

    (r, c)
}

pub fn assemble_str(src: &str) -> Result<Image, Diagnostics> {
    let mut img = Image::new();
    let mut ptr = ByteAddress::ZERO;

    let mut diags = vec![];
    let lexer = Lexer::new(src);
    let lines = lexer.lines();

    for line in lines {
        // We are guaranteed that each line has at least one identifier.
        let first = line.first().unwrap();
        let mn = match match_mneumonic(first.1) {
            Ok(m) => m,
            Err(msg) => {
                let (row, col) = idx_to_row_col(src, first.0);
                diags.push(Diagnostic { msg, row, col });
                continue;
            },
        };

        match mn {
            Mneumonic::Op(op) => match parse_from_op(op, first.0, &line[1..]) {
                Ok(i) => {
                    img.write_word(ptr, i.encode());
                    ptr = ptr.overflowing_add_words(WordOffset(1)).0;
                }
                Err((pos, msg)) => {
                    let (row, col) = idx_to_row_col(src, pos);
                    diags.push(Diagnostic { msg, row, col });
                }
            }
            Mneumonic::Directive(Directive::Ascii) => {
                todo!();
            },
        }
    }

    if diags.len() != 0 {
        Err(Diagnostics { diags })
    } else {
        Ok(img)
    }
}

enum Directive {
    Ascii,
}

impl TryFrom<&str> for Directive {
    type Error = String;

    fn try_from(value: &str) -> Result<Directive, Self::Error> {
        match value {
            ".ascii" => Ok(Directive::Ascii),
            s => Err(format!("unknown directive: '{}'", s))
        }
    }
}

enum Mneumonic {
    Op(Op),
    Directive(Directive),
}

fn match_mneumonic(ident: &str) -> Result<Mneumonic, String> {
    if let Ok(op) = Op::try_from(ident) {
        return Ok(Mneumonic::Op(op));
    }

    if let Ok(dir) = Directive::try_from(ident) {
        return Ok(Mneumonic::Directive(dir));
    }

    let f = format!(
        "unknown mneumonic, expected either an instruction or a directive, found: {}",
        ident);
    Err(f)
}

fn expect_token_count(
    op: Op,
    rest: &[(usize, &str)],
    count: usize,
) -> Result<(), String> {
    if rest.len() != count {
        let f = format!(
            "instruction '{}' expects {} registers, found: {}",
            op.name(), count, rest.len());
        return Err(f);
    }

    Ok(())
}

fn parse_from_op(
    op: Op,
    pos: usize,
    rest: &[(usize, &str)],
) -> Result<Instruction, (usize, String)> {
    match op {
        Op::Noop => Ok(Instruction::NOOP),
        ////////////////////////////////////////////////////////////////////////
        // R types
        ////////////////////////////////////////////////////////////////////////
        Op::Add
        | Op::Sub
        | Op::Shl
        | Op::Shr
        | Op::Or
        | Op::And
        | Op::Xor
        | Op::Slt
        | Op::Sltu
        | Op::Eql => {
            expect_token_count(op, rest, 3).map_err(|e| (pos, e))?;
            let rr = parse_register_diag(rest[0])?;
            let ra = parse_register_diag(rest[1])?;
            let rb = parse_register_diag(rest[2])?;
            Ok(Instruction::r_type(op, rr, ra, rb))
        }
        Op::Debu => {
            expect_token_count(op, rest, 1).map_err(|e| (pos, e))?;
            let rr = parse_register_diag(rest[0])?;
            Ok(is::debu(rr))
        }
        // /////////////////////////////////////////////////////////////////////
        // // I types
        // /////////////////////////////////////////////////////////////////////
        Op::Sys => {
            expect_token_count(op, rest, 0).map_err(|e| (pos, e))?;
            Ok(is::sys())
        }
        Op::Addi
        | Op::Subi
        | Op::Shli
        | Op::Shri
        | Op::Ori
        | Op::Orui
        | Op::Andi
        | Op::Andui
        | Op::Xori
        | Op::Xorui
        | Op::Slti
        | Op::Sltui
        | Op::Ldw
        | Op::Ldhw
        | Op::Ldhwu
        | Op::Ldb
        | Op::Ldbu
        | Op::Stw
        | Op::Sthw
        | Op::Stb
        | Op::Jmp
        | Op::Jmpr
        | Op::Beq
        | Op::Bne => {
            expect_token_count(op, rest, 3).map_err(|e| (pos, e))?;
            let rr = parse_register_diag(rest[0])?;
            let ra = parse_register_diag(rest[1])?;
            let imm = u16::from_str_radix(rest[2].1, 10).map_err(|_| {
                let f = format!(
                    "could not parse immediate as u16: '{}'", rest[2].1);
                (rest[2].0, f)
            })?;

            Ok(Instruction::i_type(op, rr, ra, imm))
        }
    }
}

fn parse_register(reg: &str) -> Option<usize> {
    reg.strip_prefix('r')?.parse().ok()
}

fn parse_register_diag(reg: (usize, &str)) -> Result<usize, (usize, String)> {
    parse_register(reg.1).ok_or(
        (reg.0, format!("found unknown register: '{}", reg.1)))
}
