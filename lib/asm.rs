use std::fmt;

use crate::{
    ByteAddress, Op, image,
    is::{self, Instruction},
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
    pub diags: Vec<Diagnostic>,
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diag in self.diags.iter() {
            writeln!(f, "{diag}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostics {}

pub fn assemble_str(src: &str) -> Result<image::Image, Diagnostics> {
    let assembler = Assembler::new(src);
    assembler.assemble_until_end()
}

type Span = (usize, usize);

#[rustfmt::skip]
#[derive(Debug, PartialEq)]
enum Token<'a> {
    String { span: Span, raw: &'a str },
    Ident  { span: Span, raw: &'a str },
}

impl Token<'_> {
    #[rustfmt::skip]
    fn tag(&self) -> String {
        match self {
            Token::String { span: _, raw } => format!("string \"{}\"", raw),
            Token::Ident  { span: _, raw } => format!("identifier `{}`", raw),
        }
    }

    #[rustfmt::skip]
    fn source_pos(&self) -> usize {
        match self {
            Token::String { span, raw: _ }
            | Token::Ident { span, raw: _ } => span.0,
        }
    }
}

struct Lexer<'src> {
    toks: Vec<Token<'src>>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Lexer<'a> {
        let mut toks = vec![];
        let mut chars = src.char_indices().peekable();

        while let Some((start, ch)) = chars.next() {
            if ch.is_whitespace() {
                continue;
            }

            if ch == ';' {
                for (_, next) in chars.by_ref() {
                    if next == '\n' {
                        break;
                    }
                }
                continue;
            }

            if ch == '"' {
                let mut end = src.len();

                for (i, next) in chars.by_ref() {
                    if next == '"' {
                        end = i + next.len_utf8();
                        break;
                    }
                }

                toks.push(Token::String {
                    span: (start, end),
                    raw: &src[start + 1..end - 1],
                });
                continue;
            }

            let mut end = src.len();

            while let Some(&(i, next)) = chars.peek() {
                if next.is_whitespace() || next == ';' {
                    end = i;
                    break;
                }

                if next == '"' {
                    break;
                }

                chars.next();
            }

            toks.push(Token::Ident {
                span: (start, end),
                raw: &src[start..end],
            });
        }

        Lexer { toks }
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

struct Assembler<'s> {
    src: &'s str,
    lexer: Lexer<'s>,
    cursor: usize,

    ptrs: Vec<(&'s str, ByteAddress)>,
    builder: image::ImageBuilder,

    diags: Vec<Diagnostic>,
}

impl<'s> Assembler<'s> {
    fn new(src: &str) -> Assembler {
        Assembler {
            src,
            lexer: Lexer::new(src),
            cursor: 0,
            ptrs: Vec::new(),
            builder: image::ImageBuilder::new(),
            diags: vec![],
        }
    }

    fn assemble_until_end(mut self) -> Result<image::Image, Diagnostics> {
        while self.cursor < self.lexer.toks.len() {
            let tok = &self.lexer.toks[self.cursor];

            self.cursor += 1;
            match tok {
                Token::String {
                    span: (st, _),
                    raw: _,
                } => {
                    let msg = format!("unexpected token: {}", tok.tag());
                    let (row, col) = idx_to_row_col(self.src, *st);
                    self.diags.push(Diagnostic { msg, row, col });
                }
                Token::Ident { span, raw } => self.assemble_ident(*span, raw),
            }
        }

        if self.diags.len() != 0 {
            Err(Diagnostics {
                diags: self.diags.clone(),
            })
        } else {
            Ok(self.builder.emit_image())
        }
    }

    fn assemble_ident(&mut self, span: Span, raw: &str) {
        let mn = match find_mneumonic(raw) {
            Ok(m) => m,
            Err(msg) => {
                let (row, col) = self.pos(span.0);
                self.diags.push(Diagnostic { msg, row, col });
                return;
            }
        };

        match mn {
            Mneumonic::Op(op) => match self.assemble_instruction(span, op) {
                Ok(()) => {}
                Err((pos, msg)) => {
                    let (row, col) = self.pos(pos);
                    self.diags.push(Diagnostic { msg, row, col });
                    return;
                }
            },
            Mneumonic::Directive(di) => match self.assemble_directive(span, di) {
                Ok(()) => {}
                Err((pos, msg)) => {
                    let (row, col) = self.pos(pos);
                    self.diags.push(Diagnostic { msg, row, col });
                    return;
                }
            },
        }
    }

    fn assemble_instruction(&mut self, span: Span, op: Op) -> Result<(), (usize, String)> {
        let rest = &self.lexer.toks[self.cursor..];

        let inst = match op {
            Op::Noop => Instruction::NOOP,
            ////////////////////////////////////////////////////////////////////
            // R types
            ////////////////////////////////////////////////////////////////////
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
                expect_token_count(op, rest, 3).map_err(|e| (span.0, e))?;
                let rr = parse_register_diag(&rest[0])?;
                let ra = parse_register_diag(&rest[1])?;
                let rb = parse_register_diag(&rest[2])?;
                self.cursor += 3;
                Instruction::r_type(op, rr, ra, rb)
            }
            Op::Debu => {
                expect_token_count(op, rest, 1).map_err(|e| (span.0, e))?;
                let rr = parse_register_diag(&rest[0])?;
                self.cursor += 1;
                is::debu(rr)
            }
            ////////////////////////////////////////////////////////////////////
            // I types
            ////////////////////////////////////////////////////////////////////
            Op::Sys => {
                expect_token_count(op, rest, 0).map_err(|e| (span.0, e))?;
                is::sys()
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
                expect_token_count(op, rest, 3).map_err(|e| (span.0, e))?;
                let rr = parse_register_diag(&rest[0])?;
                let ra = parse_register_diag(&rest[1])?;
                let imm = parse_u16(&rest[2]).ok_or((
                    rest[2].source_pos(),
                    format!("could not parse u16 immediate: {}", rest[2].tag()),
                ))?;
                self.cursor += 3;

                Instruction::i_type(op, rr, ra, imm)
            }
        };

        self.builder
            .write_text_word(inst.encode())
            .map_err(|e| (span.0, e.to_string()))?;
        Ok(())
    }

    #[rustfmt::skip]
    fn assemble_directive(
        &mut self,
        span: Span,
        di: Directive
    ) -> Result<(), (usize, String)> {
        match di {
            Directive::Ascii => {
                let name = match &self.lexer.toks[self.cursor] {
                    Token::Ident { span: _, raw } => *raw,
                    t => return Err((
                            t.source_pos(),
                            format!(
                                "expected identifier after .ascii directive, found: {}",
                                t.tag()
                            ),
                        )),
                };
                self.cursor += 1;

                let string = match &self.lexer.toks[self.cursor] {
                    Token::String { span: _, raw } => *raw,
                    t => return Err((
                            t.source_pos(),
                            format!(
                                "expected string after .ascii directive + name, found: {}",
                                t.tag()
                            ),
                        )),
                };
                self.cursor += 1;

                let bytes = string.as_bytes();
                let addr = self
                    .builder
                    .write_data_bytes(bytes)
                    .map_err(|e| (span.0, e.to_string()))?;
                self.ptrs.push((name, addr));

                println!("self.ptrs: {:?}", self.ptrs);
            }
            Directive::SetRegister => {
                let reg = parse_register_diag(&self.lexer.toks[self.cursor])?;
                self.cursor += 1;
                let set_tok = &self.lexer.toks[self.cursor];
                let set_str = match set_tok {
                    Token::Ident { span: _, raw } => *raw,
                    t => return Err((
                            t.source_pos(),
                            format!(
                                "expected an identifier after .setreg directive, found: {}",
                                t.tag()
                            ),
                        )),
                };
                self.cursor += 1;
                if let Some(name) = set_str.strip_prefix("@") {
                    if let Some(ByteAddress(ptr)) = self.find_ptr_by_name(name) {
                        self.append_is_for_setreg(set_tok.source_pos(), reg, ptr)?;
                        return Ok(());
                    }

                    return Err((
                        set_tok.source_pos(),
                        format!("unknown ptr '{}'", name),
                    ));
                }

                let set = u32::from_str_radix(set_str, 10)
                    .map_err(|e| (set_tok.source_pos(), e.to_string()))?;
                self.append_is_for_setreg(set_tok.source_pos(), reg, set)?;
            },
        }

        Ok(())
    }

    fn append_is_for_setreg(
        &mut self,
        source_pos: usize,
        reg: usize,
        set: u32,
    ) -> Result<(), (usize, String)> {
        let iss = [
            is::xori(reg, 0, set as u16),
            is::orui(reg, reg, (set >> 16) as u16),
        ];
        for i in iss {
            self.builder
                .write_text_word(i.encode())
                .map_err(|e| (source_pos, e.to_string()))?;
        }
        Ok(())
    }

    fn find_ptr_by_name(&self, name: &str) -> Option<ByteAddress> {
        for (n, ptr) in self.ptrs.iter() {
            if *n == name {
                return Some(*ptr);
            }
        }

        None
    }

    fn pos(&self, buf_idx: usize) -> (usize, usize) {
        idx_to_row_col(self.src, buf_idx)
    }
}

#[derive(Debug)]
enum Directive {
    Ascii,
    SetRegister,
}

impl TryFrom<&str> for Directive {
    type Error = String;

    fn try_from(value: &str) -> Result<Directive, Self::Error> {
        match value {
            ".ascii" => Ok(Directive::Ascii),
            ".setreg" => Ok(Directive::SetRegister),
            s => Err(format!("unknown directive: '{}'", s)),
        }
    }
}

enum Mneumonic {
    Op(Op),
    Directive(Directive),
}

fn find_mneumonic(ident: &str) -> Result<Mneumonic, String> {
    if let Ok(op) = Op::try_from(ident) {
        return Ok(Mneumonic::Op(op));
    }

    if let Ok(dir) = Directive::try_from(ident) {
        return Ok(Mneumonic::Directive(dir));
    }

    let f = format!(
        "unknown mneumonic, expected either an instruction or a directive, found: {}",
        ident
    );
    Err(f)
}

fn expect_token_count(op: Op, rest: &[Token], count: usize) -> Result<(), String> {
    if rest.len() < count {
        let f = format!(
            "truncated token count. instruction '{}' expects {} registers, found: {}",
            op.name(),
            count,
            rest.len()
        );
        return Err(f);
    }

    Ok(())
}

fn parse_u16(tok: &Token) -> Option<u16> {
    match tok {
        Token::Ident { span: _, raw } => u16::from_str_radix(raw, 10).ok(),
        _ => None,
    }
}

fn parse_register(reg: &Token) -> Option<usize> {
    match reg {
        Token::Ident { span: _, raw } => raw.strip_prefix('r')?.parse().ok(),
        _ => None,
    }
}

fn parse_register_diag(reg: &Token) -> Result<usize, (usize, String)> {
    parse_register(reg).ok_or((
        reg.source_pos(),
        format!("found unknown register: '{}", reg.tag()),
    ))
}
