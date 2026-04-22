# Enigma Assembler Plan

## Summary

Build a new enigmaasm that assembles a purpose-built Enigma assembly language
directly into the existing sparse .evm image format. The v1 goal is not gas
compatibility; it is enough expressive power to write the low-level FORTH
bootstrap in assembly with a JonesForth-like feature set: labels, constants,
data emission, includes, and macros.

The first real consumer should be a handwritten bootstrap kernel that replaces
the manual image-building in enigmaforth.rs for low-level primitives such as
NEXT, DOCOL, EXIT, LIT, stack shuffles, memory words, variables, and
syscall-backed I/O. Higher- level FORTH remains a separate next step.

## Public Interfaces

- Add a new binary: enigmaasm.
- CLI contract:
  - enigmaasm <input-file> [-o output.evm]
  - Default output is the input stem with .evm.
  - Non-zero exit on diagnostics; diagnostics include file, line, column, and
    the failing source span.
- Add a reusable library API under enigma::asm:
  - assemble_str(source: &str) -> Result<Image, Diagnostics>
  - assemble_path(path: &Path) -> Result<Image, Diagnostics>
- Assembly syntax for v1:
  - Labels: name:
  - Registers: r0 through r31
  - Literals: decimal, 0x hex, character literals like 'A'
  - Expressions: symbols plus + - * / % << >> & | ^ ~ and current location $
  - Comments: ; to end of line
  - Core directives: .org, .align, .equ, .byte, .half, .word, .ascii, .asciz,
    .space, .include, .macro, .endm
  - ISA mnemonics stay Enigma-native; load/store use address syntax like ldw
    r1, [r2 + 4]
  - Branch/jump operands may be labels; assembler computes the VM’s
    word-relative offsets and rejects out-of-range targets
  - jmp label should default the link register to r0; the full register form
    remains available

## Implementation Changes

- Implement the assembler as a small pipeline:
  - Lex and parse into a span-carrying statement stream.
  - Resolve .include first, then expand macros into normal statements while
    preserving original spans for diagnostics.
  - Pass 1 builds the symbol table and advances a single absolute location
    counter.
  - Pass 2 encodes instructions and data directly into Image using existing
    is::* constructors and Image::write_*.
- Keep v1 absolute-addressed:
  - No object files, relocations, linker, or multi-section layout.
  - .org is the mechanism for sparse placement in the VM address space.
- Enforce strict validation:
  - Unknown/duplicate symbols
  - Wrong operand counts or operand kinds
  - Immediate overflow or signed-range violations
  - Misaligned .word/instruction placement
  - Branch/jump offsets outside i16 word range
  - Include cycles and runaway macro recursion
- Ship a standard macro include for FORTH bootstrap assembly:
  - NEXT
  - parameter/return stack push-pop helpers
  - 32-bit literal/address load helper
  - dictionary-header helpers expressed as normal macros, not assembler-special
    directives
- Use that include to port the low-level FORTH kernel currently emitted by
  Rust:
  - NEXT, DOCOL, EXIT, LIT
  - stack, arithmetic, compare, bitwise, memory, and syscall-backed primitives
  - variables/constants such as STATE, LATEST, HERE-equivalent runtime cells
- Treat the assembler-port kernel as the acceptance target for v1. After that
  port starts, stop extending the Rust-side Putter
DSL for low-level primitives.

## Test Plan

- Parser and expression tests:
  - literals, registers, labels, memory operands, precedence, current-location
    expressions
  - macro definitions/invocations, nested includes, and macro-local labels
- Encoding tests:
  - each instruction form assembles to the same word as the existing is::*
    helpers
  - forward and backward branch/jump label resolution uses correct word offsets
  - sparse .org output round-trips through Image::dump_chunks and
    Image::from_chunk_bytes
- Diagnostic tests:
  - undefined symbol
  - duplicate label
  - immediate out of range
  - unaligned instruction/data
  - include cycle
  - wrong operand shape
- VM integration tests:
  - assemble and run tiny programs covering arithmetic, memory, branches, and
    syscalls
  - assemble a hello-world style program that writes through the existing
    syscall interface
- Bootstrap acceptance test:
  - assemble the FORTH kernel source and verify that key primitives behave the
    same as the current Rust-emitted image for
    overlapping words

## Assumptions And Defaults

- v1 is image-only and writes .evm directly.
- v1 is purpose-built Enigma assembly, not gas-compatible.
- v1 uses one absolute address space with .org; no linker and no relocatable
  intermediate format.
- Dictionary convenience stays in user-space macros for now; there are no
  built-in defcode/defword/defvar directives.
- Entry remains address 0x00000000, matching current VM behavior.
- Out of scope for v1: conditional assembly, debug symbol files, disassembler
  support, and full self-hosting of FORTH.
