# Enigma Assembler

## Summary

`enigmaasm` is a purpose-built assembler for the Enigma VM. It assembles directly
into the existing sparse `.evm` image format and is intended to be expressive
enough for the low-level FORTH bootstrap work that currently lives in Rust.

The implementation is reusable from Rust through `enigma::asm`:

- `assemble_str(source: &str) -> Result<Image, Diagnostics>`
- `assemble_path(path: &Path) -> Result<Image, Diagnostics>`

## CLI

```text
enigmaasm <input-file> [-o output.evm]
```

If `-o` is omitted, the assembler writes the input stem with an `.evm`
extension.

## Implemented Syntax

### General

- Comments start with `;`
- Registers are `r0` through `r31`
- Literals support decimal, `0x` hex, and character literals like `'A'`
- Expressions support `+ - * / % << >> & | ^ ~` and current location `$`
- Labels use `name:`

### Directives

- `.org expr`
- `.align expr`
- `.equ NAME, expr`
- `.byte expr, ...`
- `.half expr, ...`
- `.word expr, ...`
- `.ascii "text", ...`
- `.asciz "text", ...`
- `.space expr`
- `.include "path"`
- `.macro NAME arg1, arg2, ...`
- `.endm`

### Macros

Macros are line-oriented and expanded before parsing/encoding. Macro-local
labels are supported by prefixing a label or symbol reference with `%%`.

Example:

```asm
.macro load32 dst, value
    xori dst, r0, value & 0xFFFF
    orui dst, dst, (value >> 16) & 0xFFFF
.endm
```

### Instructions

The assembler uses Enigma-native mnemonics:

- R-type: `add`, `sub`, `shl`, `shr`, `or`, `and`, `xor`, `slt`, `sltu`, `eql`
- Misc: `noop`, `halt`, `sys`, `deb`
- I-type arithmetic/logical: `addi`, `subi`, `shli`, `shri`, `ori`, `orui`,
  `andi`, `andui`, `xori`, `xorui`, `slti`, `sltui`
- Memory: `ldw`, `ldhw`, `ldhwu`, `ldb`, `ldbu`, `stw`, `sthw`, `stb`
- Control flow: `jmp`, `jmpr`, `beq`, `bne`

Memory operands use:

```asm
ldw r1, [r2]
stw r3, [r31 + 4]
ldb r4, [r5 - 1]
```

`jmp` accepts `jmp target` or `jmp rr, target`. Branch and jump label operands
are encoded as the VM's signed word-relative offsets.

## Current Scope

Implemented in this pass:

- reusable assembler library module
- `enigmaasm` binary
- sparse-image emission into `.evm`
- includes, macros, macro-local labels, expressions, labels, and data directives
- diagnostics with file/line/column rendering
- unit tests for encoding, labels, macros, includes, diagnostics, and sparse
  image round-trips

Not implemented yet:

- relocatable objects or a linker
- conditional assembly
- debug symbol outputs
- migration of the existing `enigmaforth.rs` bootstrap off the Rust-side
  `Putter` emitter
