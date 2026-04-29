use std::fmt;

use crate::{
    ByteAddress, Op,
    builders::{MemoryBuilder, MemoryBuilderError, SegmentId},
    image,
    is::{self, Instruction},
};

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pos: usize,
    kind: AsmError,
}

impl Diagnostic {
    fn new(pos: usize, kind: AsmError) -> Diagnostic {
        Diagnostic { pos, kind }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn kind(&self) -> &AsmError {
        &self.kind
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum AsmError {
    #[error("unexpected token: {token}")]
    UnexpectedToken { token: String },
    #[error("expected {expected}, found: {found}")]
    ExpectedToken {
        expected: &'static str,
        found: String,
    },
    #[error("expected {expected}, found end of input")]
    ExpectedEnd { expected: &'static str },
    #[error("unknown mnemonic, expected either an instruction or a directive, found: {ident}")]
    UnknownMnemonic { ident: String },
    #[error("unknown directive: '{directive}'")]
    UnknownDirective { directive: String },
    #[error(
        "truncated token count. instruction '{instruction}' expects {expected} registers, found: {found}"
    )]
    WrongTokenCount {
        instruction: &'static str,
        expected: usize,
        found: usize,
    },
    #[error("found unknown register: {token}")]
    InvalidRegister { token: String },
    #[error("unknown symbol '{symbol}'")]
    UnknownSymbol { symbol: String },
    #[error("duplicate symbol '{symbol}'")]
    DuplicateSymbol { symbol: String },
    #[error("empty label name")]
    EmptyLabel,
    #[error("{item} appears outside any segment")]
    OutsideSegment { item: String },
    #[error("value does not fit in u16 immediate: {value:#x}")]
    ImmediateOutOfRange { value: u32 },
    #[error("unterminated escape sequence in string")]
    UnterminatedEscape,
    #[error("expected two hex digits after \\x")]
    ExpectedHexDigits,
    #[error("invalid hex escape digit: '{digit}'")]
    InvalidHexEscapeDigit { digit: char },
    #[error("unknown escape sequence: \\{escape}")]
    UnknownEscapeSequence { escape: char },
    #[error(transparent)]
    MemoryBuilder(#[from] MemoryBuilderError),
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
type AsmResult<T> = Result<T, Diagnostic>;

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
            Token::String { span: _, raw } => format!("string \"{raw}\""),
            Token::Ident  { span: _, raw } => format!("identifier `{raw}`"),
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

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.tag())
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

        writeln!(f, "--> {path}:{row}:{col}: error: {}", diag.kind)?;
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pass {
    Layout,
    Emit,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SymbolKind {
    Constant,
    Label,
}

struct Symbol<'src> {
    name: &'src str,
    value: u32,
    kind: SymbolKind,
}

struct Assembler<'s> {
    src: &'s str,
    lexer: Lexer<'s>,
    cursor: usize,

    symbols: Vec<Symbol<'s>>,
    builder: MemoryBuilder,
    active_segment: Option<SegmentId>,
    last_instruction: Option<(SegmentId, usize)>,
    setreg_widths: Vec<(usize, usize)>,

    diags: Vec<Diagnostic>,
}

impl<'s> Assembler<'s> {
    fn new(src: &'s str) -> Assembler<'s> {
        Assembler {
            src,
            lexer: Lexer::new(src),
            cursor: 0,
            symbols: Vec::new(),
            builder: MemoryBuilder::new(),
            active_segment: None,
            last_instruction: None,
            setreg_widths: Vec::new(),
            diags: vec![],
        }
    }

    fn assemble_until_end(mut self) -> Result<image::Image, Diagnostics<'s>> {
        self.assemble_pass(Pass::Layout);
        if !self.diags.is_empty() {
            return Err(Diagnostics {
                src: self.src,
                diags: self.diags,
            });
        }

        self.cursor = 0;
        self.builder = MemoryBuilder::new();
        self.active_segment = None;
        self.last_instruction = None;

        self.assemble_pass(Pass::Emit);
        if let Some((segment, pos)) = self.last_instruction {
            if let Err(err) = self.builder.write_word(segment, is::halt().encode()) {
                self.push_diag(pos, err.into());
            }
        }

        if self.diags.is_empty() {
            Ok(self.builder.emit_to_image())
        } else {
            Err(Diagnostics {
                src: self.src,
                diags: self.diags,
            })
        }
    }

    fn assemble_pass(&mut self, pass: Pass) {
        while self.cursor < self.lexer.toks.len() {
            let tok = self.lexer.toks[self.cursor];
            self.cursor += 1;

            match tok {
                Token::String {
                    span: (st, _),
                    raw: _,
                } => {
                    self.push_diag(
                        st,
                        AsmError::UnexpectedToken {
                            token: tok.to_string(),
                        },
                    );
                }
                Token::Ident { span, raw } => self.assemble_ident(pass, span, raw),
            }
        }
    }

    fn assemble_ident(&mut self, pass: Pass, span: Span, raw: &'s str) {
        if raw == "segment" {
            self.assemble_segment(span);
            return;
        }

        if let Some(name) = raw.strip_suffix(':') {
            self.assemble_label(pass, span, name);
            return;
        }

        let mn = match find_mnemonic(raw) {
            Ok(m) => m,
            Err(err) => {
                self.push_diag(span.0, err);
                return;
            }
        };

        match mn {
            Mnemonic::Op(op) => match self.assemble_instruction(pass, span, op) {
                Ok(()) => {}
                Err(diag) => self.push_error(diag),
            },
            Mnemonic::Directive(di) => match self.assemble_directive(pass, span, di) {
                Ok(()) => {}
                Err(diag) => self.push_error(diag),
            },
        }
    }

    fn assemble_segment(&mut self, span: Span) {
        let first = match self.next_token(span.1, "segment name or `from`") {
            Ok(tok) => tok,
            Err(diag) => {
                self.push_error(diag);
                return;
            }
        };

        let (name, from_tok) = match first {
            Token::Ident { raw: "from", .. } => (None, first),
            Token::Ident { raw, .. } => {
                match self.next_token(first.logical_end(), "`from` after segment name") {
                    Ok(tok) => (Some(raw), tok),
                    Err(diag) => {
                        self.push_error(diag);
                        return;
                    }
                }
            }
            other => {
                self.push_diag(
                    other.logical_start(),
                    AsmError::ExpectedToken {
                        expected: "segment name or `from`",
                        found: other.to_string(),
                    },
                );
                return;
            }
        };

        if let Err(diag) = expect_ident(&from_tok, "from", "`from` in segment declaration") {
            self.push_error(diag);
            return;
        }

        let start_tok = match self.next_token(from_tok.logical_end(), "segment start address") {
            Ok(tok) => tok,
            Err(diag) => {
                self.push_error(diag);
                return;
            }
        };
        let to_tok = match self.next_token(start_tok.logical_end(), "`to` in segment declaration") {
            Ok(tok) => tok,
            Err(diag) => {
                self.push_error(diag);
                return;
            }
        };
        if let Err(diag) = expect_ident(&to_tok, "to", "`to` in segment declaration") {
            self.push_error(diag);
            return;
        }
        let end_tok = match self.next_token(to_tok.logical_end(), "segment end address") {
            Ok(tok) => tok,
            Err(diag) => {
                self.push_error(diag);
                return;
            }
        };

        let start = match self.resolve_value(&start_tok, false) {
            Ok(Some(value)) => ByteAddress(value),
            Ok(None) => unreachable!("unresolved values are disabled"),
            Err(diag) => {
                self.push_error(diag);
                return;
            }
        };
        let end = match self.resolve_value(&end_tok, false) {
            Ok(Some(value)) => ByteAddress(value),
            Ok(None) => unreachable!("unresolved values are disabled"),
            Err(diag) => {
                self.push_error(diag);
                return;
            }
        };

        match self.builder.define_segment(name, start, end) {
            Ok(id) => self.active_segment = Some(id),
            Err(err) => self.push_diag(span.0, err.into()),
        }
    }

    fn assemble_label(&mut self, pass: Pass, span: Span, name: &'s str) {
        if name.is_empty() {
            self.push_diag(span.0, AsmError::EmptyLabel);
            return;
        }

        let Some(segment) = self.active_segment else {
            self.push_diag(
                span.0,
                AsmError::OutsideSegment {
                    item: format!("label `{name}`"),
                },
            );
            return;
        };

        if pass == Pass::Layout {
            match self.builder.segment_head(segment) {
                Ok(addr) => {
                    if let Err(diag) = self.define_symbol(span.0, name, addr.0, SymbolKind::Label) {
                        self.push_error(diag);
                    }
                }
                Err(err) => self.push_diag(span.0, err.into()),
            }
        }
    }

    fn assemble_instruction(&mut self, pass: Pass, span: Span, op: Op) -> AsmResult<()> {
        let segment = self.require_active_segment(span.0, "instruction")?;
        let rest = &self.lexer.toks[self.cursor..];

        let inst = match op {
            Op::Noop => Instruction::NOOP,
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
                expect_token_count(span.0, op, rest, 3)?;
                let rr = parse_register_diag(&rest[0])?;
                let ra = parse_register_diag(&rest[1])?;
                let rb = parse_register_diag(&rest[2])?;
                self.cursor += 3;
                Instruction::r_type(op, rr, ra, rb)
            }
            Op::Debu => {
                expect_token_count(span.0, op, rest, 1)?;
                let rr = parse_register_diag(&rest[0])?;
                self.cursor += 1;
                is::debu(rr)
            }
            Op::Sys => {
                expect_token_count(span.0, op, rest, 0)?;
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
                expect_token_count(span.0, op, rest, 3)?;
                let rr = parse_register_diag(&rest[0])?;
                let ra = parse_register_diag(&rest[1])?;
                let imm = if pass == Pass::Emit {
                    self.resolve_u16(&rest[2])?
                } else {
                    0
                };
                self.cursor += 3;
                Instruction::i_type(op, rr, ra, imm)
            }
        };

        if pass == Pass::Emit {
            self.builder
                .write_word(segment, inst.encode())
                .map_err(|err| Diagnostic::new(span.0, err.into()))?;
            self.last_instruction = Some((segment, span.0));
        } else {
            self.builder
                .reserve_bytes(segment, 4)
                .map_err(|err| Diagnostic::new(span.0, err.into()))?;
        }
        Ok(())
    }

    fn assemble_directive(&mut self, pass: Pass, span: Span, di: Directive) -> AsmResult<()> {
        match di {
            Directive::Ascii => {
                let segment = self.require_active_segment(span.0, ".ascii directive")?;
                let string_tok = self.next_token(span.1, "string after .ascii directive")?;
                let (string_pos, string) = match string_tok {
                    Token::String { span, raw } => (span.0, raw),
                    t => {
                        return Err(Diagnostic::new(
                            t.logical_start(),
                            AsmError::ExpectedToken {
                                expected: "string after .ascii directive",
                                found: t.to_string(),
                            },
                        ));
                    }
                };

                let bytes = decode_string_literal(string, string_pos + 1)?;
                if pass == Pass::Emit {
                    self.builder
                        .write_bytes(segment, bytes.as_slice())
                        .map_err(|err| Diagnostic::new(span.0, err.into()))?;
                } else {
                    self.builder
                        .reserve_bytes(segment, bytes.len() as u32)
                        .map_err(|err| Diagnostic::new(span.0, err.into()))?;
                }
            }
            Directive::Space => {
                let segment = self.require_active_segment(span.0, ".space directive")?;
                let len_tok = self.next_token(span.1, "byte count after .space directive")?;
                let len = self
                    .resolve_value(&len_tok, false)?
                    .expect("unresolved values are disabled");
                self.builder
                    .reserve_bytes(segment, len)
                    .map_err(|err| Diagnostic::new(span.0, err.into()))?;
            }
            Directive::Equ => {
                let name_tok = self.next_token(span.1, "identifier after .equ directive")?;
                let name = match name_tok {
                    Token::Ident { raw, .. } => raw,
                    t => {
                        return Err(Diagnostic::new(
                            t.logical_start(),
                            AsmError::ExpectedToken {
                                expected: "identifier after .equ directive",
                                found: t.to_string(),
                            },
                        ));
                    }
                };
                let value_tok =
                    self.next_token(name_tok.logical_end(), "value after .equ directive")?;
                if pass == Pass::Layout {
                    let value = self
                        .resolve_value(&value_tok, false)?
                        .expect("unresolved values are disabled");
                    self.define_symbol(span.0, name, value, SymbolKind::Constant)?;
                }
            }
            Directive::SetRegister => {
                let segment = self.require_active_segment(span.0, ".setreg directive")?;
                let reg_tok = self.next_token(span.1, "register after .setreg directive")?;
                let reg = parse_register_diag(&reg_tok)?;
                let set_tok = self.next_token(
                    reg_tok.logical_end(),
                    "identifier or immediate after .setreg directive",
                )?;
                match set_tok {
                    Token::Ident { .. } => {}
                    t => {
                        return Err(Diagnostic::new(
                            t.logical_start(),
                            AsmError::ExpectedToken {
                                expected: "an identifier after .setreg directive",
                                found: t.to_string(),
                            },
                        ));
                    }
                }

                if pass == Pass::Layout {
                    let byte_len = self.setreg_instruction_count(&set_tok, true)? * 4;
                    self.setreg_widths.push((span.0, byte_len / 4));
                    self.builder
                        .reserve_bytes(segment, byte_len as u32)
                        .map_err(|err| Diagnostic::new(span.0, err.into()))?;
                    return Ok(());
                }

                let set = self
                    .resolve_value(&set_tok, false)?
                    .expect("unresolved values are disabled");
                self.builder
                    .write_word(segment, is::xori(reg, 0, set as u16).encode())
                    .map_err(|err| Diagnostic::new(set_tok.logical_start(), err.into()))?;
                self.last_instruction = Some((segment, span.0));
                if self.setreg_instruction_count_at(span.0, &set_tok, set)? == 2 {
                    self.builder
                        .write_word(segment, is::orui(reg, reg, (set >> 16) as u16).encode())
                        .map_err(|err| Diagnostic::new(set_tok.logical_start(), err.into()))?;
                    self.last_instruction = Some((segment, span.0));
                }
            }
        }

        Ok(())
    }

    fn setreg_instruction_count(
        &self,
        tok: &Token<'s>,
        allow_unresolved: bool,
    ) -> AsmResult<usize> {
        match tok {
            Token::Ident { raw, .. } => {
                if let Some(value) = parse_u32_literal(raw) {
                    return Ok(if value > u16::MAX as u32 { 2 } else { 1 });
                }
                let Some(symbol) = self.find_symbol(symbol_name(raw)) else {
                    if allow_unresolved {
                        return Ok(2);
                    }
                    return Err(Diagnostic::new(
                        tok.logical_start(),
                        AsmError::UnknownSymbol {
                            symbol: raw.to_string(),
                        },
                    ));
                };
                match symbol.kind {
                    SymbolKind::Constant => Ok(if symbol.value > u16::MAX as u32 { 2 } else { 1 }),
                    SymbolKind::Label => Ok(2),
                }
            }
            _ => Err(Diagnostic::new(
                tok.logical_start(),
                AsmError::ExpectedToken {
                    expected: "identifier or immediate",
                    found: tok.to_string(),
                },
            )),
        }
    }

    fn setreg_instruction_count_at(
        &self,
        pos: usize,
        tok: &Token<'s>,
        value: u32,
    ) -> AsmResult<usize> {
        if let Some((_, count)) = self
            .setreg_widths
            .iter()
            .find(|(setreg_pos, _)| *setreg_pos == pos)
        {
            return Ok(*count);
        }

        match tok {
            Token::Ident { raw, .. } if parse_u32_literal(raw).is_some() => {
                Ok(if value > u16::MAX as u32 { 2 } else { 1 })
            }
            Token::Ident { raw, .. } if self.find_symbol(symbol_name(raw)).is_some() => Ok(2),
            Token::Ident { raw, .. } => Err(Diagnostic::new(
                tok.logical_start(),
                AsmError::UnknownSymbol {
                    symbol: raw.to_string(),
                },
            )),
            _ => Err(Diagnostic::new(
                tok.logical_start(),
                AsmError::ExpectedToken {
                    expected: "identifier or immediate",
                    found: tok.to_string(),
                },
            )),
        }
    }

    fn resolve_u16(&self, tok: &Token<'s>) -> AsmResult<u16> {
        let value = self
            .resolve_value(tok, false)?
            .expect("unresolved values are disabled");
        u16::try_from(value).map_err(|_| {
            Diagnostic::new(tok.logical_start(), AsmError::ImmediateOutOfRange { value })
        })
    }

    fn resolve_value(&self, tok: &Token<'s>, allow_unresolved: bool) -> AsmResult<Option<u32>> {
        let raw = match tok {
            Token::Ident { raw, .. } => raw,
            t => {
                return Err(Diagnostic::new(
                    t.logical_start(),
                    AsmError::ExpectedToken {
                        expected: "identifier or immediate",
                        found: t.to_string(),
                    },
                ));
            }
        };

        if let Some(value) = parse_u32_literal(raw) {
            return Ok(Some(value));
        }

        if let Some(symbol) = self.find_symbol(symbol_name(raw)) {
            return Ok(Some(symbol.value));
        }

        if allow_unresolved {
            Ok(None)
        } else {
            Err(Diagnostic::new(
                tok.logical_start(),
                AsmError::UnknownSymbol {
                    symbol: raw.to_string(),
                },
            ))
        }
    }

    fn define_symbol(
        &mut self,
        pos: usize,
        name: &'s str,
        value: u32,
        kind: SymbolKind,
    ) -> AsmResult<()> {
        if self.find_symbol(name).is_some() {
            return Err(Diagnostic::new(
                pos,
                AsmError::DuplicateSymbol {
                    symbol: name.to_string(),
                },
            ));
        }
        self.symbols.push(Symbol { name, value, kind });
        Ok(())
    }

    fn find_symbol(&self, name: &str) -> Option<&Symbol<'s>> {
        self.symbols.iter().find(|symbol| symbol.name == name)
    }

    fn require_active_segment(&self, pos: usize, item: &'static str) -> AsmResult<SegmentId> {
        self.active_segment.ok_or_else(|| {
            Diagnostic::new(
                pos,
                AsmError::OutsideSegment {
                    item: item.to_string(),
                },
            )
        })
    }

    fn next_token(&mut self, pos: usize, expected: &'static str) -> AsmResult<Token<'s>> {
        let tok = *self
            .lexer
            .toks
            .get(self.cursor)
            .ok_or(Diagnostic::new(pos, AsmError::ExpectedEnd { expected }))?;
        self.cursor += 1;
        Ok(tok)
    }

    fn push_diag(&mut self, pos: usize, kind: AsmError) {
        self.diags.push(Diagnostic::new(pos, kind));
    }

    fn push_error(&mut self, diag: Diagnostic) {
        self.diags.push(diag);
    }
}

#[derive(Debug)]
enum Directive {
    Ascii,
    Equ,
    SetRegister,
    Space,
}

impl TryFrom<&str> for Directive {
    type Error = AsmError;

    fn try_from(value: &str) -> Result<Directive, Self::Error> {
        match value {
            ".ascii" => Ok(Directive::Ascii),
            ".equ" => Ok(Directive::Equ),
            ".setreg" => Ok(Directive::SetRegister),
            ".space" => Ok(Directive::Space),
            directive => Err(AsmError::UnknownDirective {
                directive: directive.to_string(),
            }),
        }
    }
}

enum Mnemonic {
    Op(Op),
    Directive(Directive),
}

fn find_mnemonic(ident: &str) -> Result<Mnemonic, AsmError> {
    if let Ok(op) = Op::try_from(ident) {
        return Ok(Mnemonic::Op(op));
    }

    if ident.starts_with('.') {
        return Directive::try_from(ident).map(Mnemonic::Directive);
    }

    Err(AsmError::UnknownMnemonic {
        ident: ident.to_string(),
    })
}

fn expect_token_count(pos: usize, op: Op, rest: &[Token], count: usize) -> AsmResult<()> {
    if rest.len() < count {
        return Err(Diagnostic::new(
            pos,
            AsmError::WrongTokenCount {
                instruction: op.name(),
                expected: count,
                found: rest.len(),
            },
        ));
    }

    Ok(())
}

fn expect_ident(tok: &Token, expected: &str, expected_desc: &'static str) -> AsmResult<()> {
    match tok {
        Token::Ident { raw, .. } if *raw == expected => Ok(()),
        t => Err(Diagnostic::new(
            t.logical_start(),
            AsmError::ExpectedToken {
                expected: expected_desc,
                found: t.to_string(),
            },
        )),
    }
}

fn parse_u32_literal(raw: &str) -> Option<u32> {
    let cleaned: String = raw.chars().filter(|ch| *ch != '_').collect();
    if let Some(hex) = cleaned
        .strip_prefix("0x")
        .or_else(|| cleaned.strip_prefix("0X"))
    {
        return u32::from_str_radix(hex, 16).ok();
    }
    cleaned.parse::<u32>().ok()
}

fn parse_register(reg: &Token) -> Option<usize> {
    match reg {
        Token::Ident { span: _, raw } => raw.strip_prefix('r')?.parse().ok(),
        _ => None,
    }
}

fn parse_register_diag(reg: &Token) -> AsmResult<usize> {
    parse_register(reg).ok_or_else(|| {
        Diagnostic::new(
            reg.logical_start(),
            AsmError::InvalidRegister {
                token: reg.to_string(),
            },
        )
    })
}

fn symbol_name(raw: &str) -> &str {
    raw.strip_prefix('@').unwrap_or(raw)
}

fn decode_string_literal(raw: &str, base_pos: usize) -> AsmResult<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut chars = raw.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        if ch != '\\' {
            let mut buf = [0u8; 4];
            bytes.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
            continue;
        }

        let Some((_, esc)) = chars.next() else {
            return Err(Diagnostic::new(
                base_pos + idx,
                AsmError::UnterminatedEscape,
            ));
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
                    return Err(Diagnostic::new(base_pos + idx, AsmError::ExpectedHexDigits));
                };
                let Some((_, lo)) = chars.next() else {
                    return Err(Diagnostic::new(base_pos + idx, AsmError::ExpectedHexDigits));
                };

                let hi = hi.to_digit(16).ok_or_else(|| {
                    Diagnostic::new(
                        base_pos + idx,
                        AsmError::InvalidHexEscapeDigit { digit: hi },
                    )
                })?;
                let lo = lo.to_digit(16).ok_or_else(|| {
                    Diagnostic::new(
                        base_pos + idx,
                        AsmError::InvalidHexEscapeDigit { digit: lo },
                    )
                })?;
                bytes.push(((hi << 4) | lo) as u8);
            }
            other => {
                return Err(Diagnostic::new(
                    base_pos + idx,
                    AsmError::UnknownEscapeSequence { escape: other },
                ));
            }
        }
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::assemble_str;
    use crate::{ByteAddress, Instruction};

    #[test]
    fn segments_labels_ascii_and_setreg_layout_across_memory() {
        let src = r#"
.equ SYSCALL_WRITE 1
.equ STDOUT_FD 1

segment data from 0x1000_0000 to 0x2000_0000
    hello:
    .ascii "Hello, Sailor!\n"

segment text from 0x0000_0000 to 0x1000_0000
    .setreg r1 SYSCALL_WRITE
    .setreg r2 STDOUT_FD
    .setreg r3 hello
    .setreg r4 15
    sys
"#;
        let image = assemble_str(src).expect("assembly should succeed");
        let mut machine = image.consume_to_machine();

        let mut buf = [0u8; 15];
        for (i, byte) in buf.iter_mut().enumerate() {
            *byte = machine.read_byte(ByteAddress(0x1000_0000 + i as u32));
        }
        assert_eq!(&buf, b"Hello, Sailor!\n");

        machine.exec_and_advance().unwrap();
        machine.exec_and_advance().unwrap();
        machine.exec_and_advance().unwrap();
        machine.exec_and_advance().unwrap();
        machine.exec_and_advance().unwrap();
        machine.exec_and_advance().unwrap();
        assert_eq!(machine.read_register(3), 0x1000_0000);
    }

    #[test]
    fn setreg_small_immediate_emits_one_instruction() {
        let image = assemble_str("segment text from 0 to 0x1000\n.setreg r4 15\nnoop")
            .expect("assembly should succeed");
        let mut machine = image.consume_to_machine();

        machine.exec_and_advance().unwrap();
        assert_eq!(machine.read_register(4), 15);

        let second = machine.current_instruction().unwrap();
        assert_eq!(second, crate::Instruction::NOOP);
    }

    #[test]
    fn forward_label_reference_across_segments_works() {
        let src = r#"
segment text from 0 to 0x1000
    .setreg r3 message

segment data from 0x1000_0000 to 0x2000_0000
    message:
    .ascii "ok"
"#;
        let image = assemble_str(src).expect("assembly should succeed");
        let mut machine = image.consume_to_machine();

        machine.exec_and_advance().unwrap();
        machine.exec_and_advance().unwrap();
        assert_eq!(machine.read_register(3), 0x1000_0000);
    }

    #[test]
    fn assembler_appends_halt_after_last_instruction() {
        let image =
            assemble_str("segment text from 0 to 0x1000\nnoop").expect("assembly should succeed");
        let mut machine = image.consume_to_machine();

        machine.exec_and_advance().unwrap();
        assert_eq!(machine.current_instruction().unwrap(), Instruction::HALT);
    }

    #[test]
    fn space_reserves_addresses_without_emitting_bytes() {
        let src = r#"
segment heap from 0x2000_0000 to 0x3000_0000
    heap_start:
    .space 0x10
    heap_end:

segment text from 0 to 0x1000
    .setreg r1 heap_start
    .setreg r2 heap_end
"#;
        let image = assemble_str(src).expect("assembly should succeed");
        let mut machine = image.consume_to_machine();

        machine.exec_and_advance().unwrap();
        machine.exec_and_advance().unwrap();
        machine.exec_and_advance().unwrap();
        machine.exec_and_advance().unwrap();
        assert_eq!(machine.read_register(1), 0x2000_0000);
        assert_eq!(machine.read_register(2), 0x2000_0010);
        assert_eq!(machine.read_byte(ByteAddress(0x2000_0000)), 0);
    }

    #[test]
    fn directive_outside_segment_reports_diagnostic() {
        let err = match assemble_str(".ascii \"nope\"") {
            Ok(_) => panic!("assembly should fail"),
            Err(err) => err,
        };
        let rendered = err.to_string();

        assert!(rendered.contains(".ascii directive appears outside any segment"));
        assert!(rendered.contains("1 | .ascii \"nope\""));
    }

    #[test]
    fn overlapping_segments_report_diagnostic() {
        let err = match assemble_str("segment a from 0 to 0x100\nsegment b from 0x80 to 0x200") {
            Ok(_) => panic!("assembly should fail"),
            Err(err) => err,
        };
        let rendered = err.to_string();

        assert!(rendered.contains("segment `b` overlaps existing segment"));
    }

    #[test]
    fn duplicate_symbol_reports_diagnostic() {
        let err = match assemble_str(".equ same 1\nsegment text from 0 to 0x100\nsame:\nnoop") {
            Ok(_) => panic!("assembly should fail"),
            Err(err) => err,
        };
        let rendered = err.to_string();

        assert!(rendered.contains("duplicate symbol 'same'"));
    }

    #[test]
    fn unknown_setreg_symbol_reports_diagnostic() {
        let err = match assemble_str("segment text from 0 to 0x100\n.setreg r1 missing") {
            Ok(_) => panic!("assembly should fail"),
            Err(err) => err,
        };
        let rendered = err.to_string();

        assert!(rendered.contains("unknown symbol 'missing'"));
    }

    #[test]
    fn segment_overflow_reports_diagnostic() {
        let err = match assemble_str("segment text from 0 to 4\nnoop") {
            Ok(_) => panic!("assembly should fail"),
            Err(err) => err,
        };
        let rendered = err.to_string();

        assert!(rendered.contains("segment `text` overflow"));
    }

    #[test]
    fn setreg_without_operands_reports_diagnostic_instead_of_panicking() {
        let err = match assemble_str("segment text from 0 to 0x100\n.setreg") {
            Ok(_) => panic!("assembly should fail"),
            Err(err) => err,
        };
        let rendered = err.to_string();

        assert!(rendered.contains("expected register after .setreg directive, found end of input"));
        assert!(rendered.contains(".setreg"));
    }
}
