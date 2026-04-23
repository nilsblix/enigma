use std::fmt;

use crate::{
    ByteAddress, Op, image,
    is::{self, Instruction},
};

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pos: usize,
    msg: String,
}

#[derive(Debug, Clone)]
pub struct Diagnostics<'src> {
    src: &'src str,
    pub diags: Vec<Diagnostic>,
}

pub struct DiagnosticsDisplay<'a, 'src> {
    path: &'a str,
    diagnostics: &'a Diagnostics<'src>,
}

impl<'src> Diagnostics<'src> {
    pub fn with_path<'a>(&'a self, path: &'a str) -> DiagnosticsDisplay<'a, 'src> {
        DiagnosticsDisplay {
            path,
            diagnostics: self,
        }
    }
}

impl fmt::Display for Diagnostics<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_diagnostics(f, "<input>", self)
    }
}

impl fmt::Display for DiagnosticsDisplay<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_diagnostics(f, self.path, self.diagnostics)
    }
}

impl std::error::Error for Diagnostics<'_> {}

pub fn assemble_str<'src>(src: &'src str) -> Result<image::Image, Diagnostics<'src>> {
    let assembler = Assembler::new(src);
    assembler.assemble_until_end()
}

type Span = (usize, usize);

#[rustfmt::skip]
#[derive(Debug, PartialEq, Clone, Copy)]
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
    fn logical_start(&self) -> usize {
        match self {
            Token::String  { span, raw: _ }
            | Token::Ident { span, raw: _ } => span.0,
        }
    }

    #[rustfmt::skip]
    fn logical_end(&self) -> usize {
        match self {
            Token::String  { span, raw: _ }
            | Token::Ident { span, raw: _ } => span.1,
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

fn logical_to_row_col(src: &str, idx: usize) -> (usize, usize) {
    let idx = idx.min(src.len());
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

fn source_line_at(src: &str, idx: usize) -> &str {
    let clamped = idx.min(src.len());
    let start = src[..clamped].rfind('\n').map_or(0, |i| i + 1);
    let end = src[clamped..].find('\n').map_or(src.len(), |i| clamped + i);
    &src[start..end]
}

fn write_diagnostics(
    f: &mut fmt::Formatter<'_>,
    path: &str,
    diagnostics: &Diagnostics<'_>,
) -> fmt::Result {
    for (idx, diag) in diagnostics.diags.iter().enumerate() {
        let (row, col) = logical_to_row_col(diagnostics.src, diag.pos);
        let line = source_line_at(diagnostics.src, diag.pos);
        let gutter = " ".repeat(row.to_string().len());
        let caret_pad = " ".repeat(col.saturating_sub(1));

        writeln!(f, "--> {path}:{row}:{col}: error: {}", diag.msg)?;
        writeln!(f, "{gutter} |")?;
        writeln!(f, "{row} | {line}")?;

        if idx == diagnostics.diags.len() - 1 {
            write!(f, "{gutter} | {caret_pad}^")?;
        } else {
            writeln!(f, "{gutter} | {caret_pad}^\n")?;
        }
    }

    Ok(())
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
    fn new(src: &'s str) -> Assembler<'s> {
        Assembler {
            src,
            lexer: Lexer::new(src),
            cursor: 0,
            ptrs: Vec::new(),
            builder: image::ImageBuilder::new(),
            diags: vec![],
        }
    }

    fn assemble_until_end(mut self) -> Result<image::Image, Diagnostics<'s>> {
        while self.cursor < self.lexer.toks.len() {
            let tok = &self.lexer.toks[self.cursor];

            self.cursor += 1;
            match tok {
                Token::String {
                    span: (st, _),
                    raw: _,
                } => {
                    let msg = format!("unexpected token: {}", tok.tag());
                    self.push_diag(*st, msg);
                }
                Token::Ident { span, raw } => self.assemble_ident(*span, raw),
            }
        }

        if self.diags.len() != 0 {
            Err(Diagnostics {
                src: self.src,
                diags: self.diags,
            })
        } else {
            Ok(self.builder.emit_image())
        }
    }

    fn assemble_ident(&mut self, span: Span, raw: &str) {
        let mn = match find_mneumonic(raw) {
            Ok(m) => m,
            Err(msg) => {
                self.push_diag(span.0, msg);
                return;
            }
        };

        match mn {
            Mneumonic::Op(op) => match self.assemble_instruction(span, op) {
                Ok(()) => {}
                Err((pos, msg)) => {
                    self.push_diag(pos, msg);
                    return;
                }
            },
            Mneumonic::Directive(di) => match self.assemble_directive(span, di) {
                Ok(()) => {}
                Err((pos, msg)) => {
                    self.push_diag(pos, msg);
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
                    rest[2].logical_start(),
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
                let name_tok = self.next_token(span.1, "identifier after .ascii directive")?;
                let name = match name_tok {
                    Token::Ident { span: _, raw } => raw,
                    t => return Err((
                            t.logical_start(),
                            format!(
                                "expected identifier after .ascii directive, found: {}",
                                t.tag()
                            ),
                        )),
                };

                let string_tok =
                    self.next_token(name_tok.logical_end(), "string after .ascii directive + name")?;
                let (string_pos, string) = match string_tok {
                    Token::String { span, raw } => (span.0, raw),
                    t => return Err((
                            t.logical_start(),
                            format!(
                                "expected string after .ascii directive + name, found: {}",
                                t.tag()
                            ),
                        )),
                };

                let bytes = decode_string_literal(string)
                    .map_err(|(offset, msg)| (string_pos + 1 + offset, msg))?;
                let addr = self
                    .builder
                    .write_data_bytes(bytes.as_slice())
                    .map_err(|e| (span.0, e.to_string()))?;
                self.ptrs.push((name, addr));
            }
            Directive::SetRegister => {
                let reg_tok = self.next_token(span.1, "register after .setreg directive")?;
                let reg = parse_register_diag(&reg_tok)?;
                let set_tok = self.next_token(
                    reg_tok.logical_end(),
                    "identifier or immediate after .setreg directive",
                )?;
                let set_str = match set_tok {
                    Token::Ident { span: _, raw } => raw,
                    t => return Err((
                            t.logical_start(),
                            format!(
                                "expected an identifier after .setreg directive, found: {}",
                                t.tag()
                            ),
                        )),
                };
                if let Some(name) = set_str.strip_prefix("@") {
                    if let Some(ByteAddress(ptr)) = self.find_ptr_by_name(name) {
                        self.append_is_for_setreg(set_tok.logical_start(), reg, ptr)?;
                        return Ok(());
                    }

                    return Err((
                        set_tok.logical_start(),
                        format!("unknown ptr '{}'", name),
                    ));
                }

                let set = u32::from_str_radix(set_str, 10)
                    .map_err(|e| (set_tok.logical_start(), e.to_string()))?;
                self.append_is_for_setreg(set_tok.logical_start(), reg, set)?;
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
        self.builder
            .write_text_word(is::xori(reg, 0, set as u16).encode())
            .map_err(|e| (source_pos, e.to_string()))?;
        if set > u16::MAX as u32 {
            self.builder
                .write_text_word(is::orui(reg, reg, (set >> 16) as u16).encode())
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

    fn next_token(
        &mut self,
        pos: usize,
        expected: &'static str,
    ) -> Result<Token<'s>, (usize, String)> {
        let tok = *self
            .lexer
            .toks
            .get(self.cursor)
            .ok_or((pos, format!("expected {expected}, found end of input")))?;
        self.cursor += 1;
        Ok(tok)
    }

    fn push_diag(&mut self, pos: usize, msg: String) {
        self.diags.push(Diagnostic { pos, msg });
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
        reg.logical_start(),
        format!("found unknown register: '{}", reg.tag()),
    ))
}

fn decode_string_literal(raw: &str) -> Result<Vec<u8>, (usize, String)> {
    let mut bytes = Vec::new();
    let mut chars = raw.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        if ch != '\\' {
            let mut buf = [0u8; 4];
            bytes.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
            continue;
        }

        let Some((_, esc)) = chars.next() else {
            return Err((idx, "unterminated escape sequence in string".to_string()));
        };

        match esc {
            '\\' => bytes.push(b'\\'),
            '"' => bytes.push(b'"'),
            'n' => bytes.push(b'\n'),
            'r' => bytes.push(b'\r'),
            't' => bytes.push(b'\t'),
            '0' => bytes.push(b'\0'),
            'x' => {
                let Some((_, hi)) = chars.next() else {
                    return Err((idx, "expected two hex digits after \\x".to_string()));
                };
                let Some((_, lo)) = chars.next() else {
                    return Err((idx, "expected two hex digits after \\x".to_string()));
                };

                let hi = hi
                    .to_digit(16)
                    .ok_or((idx, format!("invalid hex escape digit: '{}'", hi)))?;
                let lo = lo
                    .to_digit(16)
                    .ok_or((idx, format!("invalid hex escape digit: '{}'", lo)))?;
                bytes.push(((hi << 4) | lo) as u8);
            }
            other => return Err((idx, format!("unknown escape sequence: \\{}", other))),
        }
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::assemble_str;
    use crate::ByteAddress;

    #[test]
    fn ascii_and_setreg_use_data_pointer_and_decode_newline() {
        let src = ".ascii hello \"Hello, Sailor!\\n\"\n.setreg r3 @hello\n.setreg r4 15\nnoop";
        let image = assemble_str(src).expect("assembly should succeed");
        let mut machine = image.consume_to_machine();

        let mut buf = [0u8; 15];
        for (i, byte) in buf.iter_mut().enumerate() {
            *byte = machine.read_byte(ByteAddress(0x1000_0000 + i as u32));
        }
        assert_eq!(&buf, b"Hello, Sailor!\n");

        machine.exec_and_advance().unwrap();
        machine.exec_and_advance().unwrap();
        assert_eq!(machine.read_register(3), 0x1000_0000);

        machine.exec_and_advance().unwrap();
        machine.exec_and_advance().unwrap();
        assert_eq!(machine.read_register(4), 15);
    }

    #[test]
    fn setreg_small_immediate_emits_one_instruction() {
        let image = assemble_str(".setreg r4 15\nnoop").expect("assembly should succeed");
        let mut machine = image.consume_to_machine();

        machine.exec_and_advance().unwrap();
        assert_eq!(machine.read_register(4), 15);

        let second = machine.current_instruction().unwrap();
        assert_eq!(second, crate::Instruction::NOOP);
    }

    #[test]
    fn ascii_without_name_reports_diagnostic_instead_of_panicking() {
        let err = match assemble_str(".ascii") {
            Ok(_) => panic!("assembly should fail"),
            Err(err) => err,
        };
        let rendered = err.to_string();

        assert!(rendered.contains("--> <input>:1:7: error:"));
        assert!(
            rendered.contains("expected identifier after .ascii directive, found end of input")
        );
        assert!(rendered.contains("  |"));
        assert!(rendered.contains("1 | .ascii"));
        assert!(rendered.contains("  |       ^"));
        assert!(rendered.contains(".ascii"));
    }

    #[test]
    fn setreg_without_operands_reports_diagnostic_instead_of_panicking() {
        let err = match assemble_str(".setreg") {
            Ok(_) => panic!("assembly should fail"),
            Err(err) => err,
        };
        let rendered = err.to_string();

        assert!(rendered.contains("--> <input>:1:8: error:"));
        assert!(rendered.contains("expected register after .setreg directive, found end of input"));
        assert!(rendered.contains("1 | .setreg"));
        assert!(rendered.contains("  |        ^"));
    }
}
