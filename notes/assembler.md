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
- Literals support decimal and `0x` hex, with optional `_` separators
- Constants use `.equ NAME VALUE`
- Labels use `name:` and resolve to the current segment head

### Segments

Segments are compile-time layout ranges. They prevent assembler-time overlap and
overflow, but do not provide runtime memory protection.

```esm
segment data from 0x1000_0000 to 0x2000_0000
    quote:
    .ascii "Hello, Sailor!\n"

segment text from 0x0000_0000 to 0x1000_0000
    .setreg r3 quote
```

Segment names are optional and non-semantic:

```esm
segment from 0x2000_0000 to 0x3000_0000
    heap_start:
    .space 0x1000
    heap_end:
```

### Directives

- `.equ NAME VALUE`
- `.ascii "text"`
- `.space BYTES`
- `.setreg rN VALUE`

The assembler automatically appends a halt instruction after the last emitted
instruction.

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

The low-level instruction syntax currently accepts the raw VM operand shape.
I-type immediates may be decimal, hex, constants, or labels when the final value
fits in `u16`.

## Current Scope

Implemented:

- reusable assembler library module
- sparse-image emission into `.evm`
- segment-based layout, labels, constants, `.ascii`, `.space`, and `.setreg`
- diagnostics with file/line/column rendering

Not implemented yet:

- relocatable objects or a linker
- conditional assembly
- debug symbol outputs
- includes, macros, and expressions
