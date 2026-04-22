# Enigma

Enigma is a 32-bit, big-endian virtual machine with a flat 4 GiB address space
backed by sparse 64 KiB blocks.

> [!IMPORTANT]
> Small and experimental. Do not expect this project to be mature and have
> reasonable standards. The EVM (Enigma Virtual Machine) was made for learning
> purposes, and has served as a fun side-project.

## Current Stage

The VM core is implemented and runnable. `enigmavm` can load `.evm` images,
execute the instruction set below, and expose sparse RAM, MMIO, and a small
POSIX-oriented syscall layer. The assembler and higher-level tooling are still
in progress.

**Note** that the `enigmavm` is currently focused on POSIX-style syscalls, and
might be changed in the future to accomodate more systems. The core library,
namely `enigmalib`, is OS-agnostic, and all IO/Syscall implementations decide
what system the VM is meant for.

## Machine Model

- 32 registers.
- `r0` is hard-wired to zero.
- `r1` is the syscall number on `sys`, and also the primary return register.
- `r2..r7` are syscall argument registers.
- `r31` is the stack pointer and starts at `0xEFFFFFFC`.
- Execution starts at byte address `0x00000000`.
- MMIO begins at `0xF0000000`.

## Instruction Set

All instructions are 32-bit words.

- R-type: `add`, `sub`, `shl`, `shr`, `or`, `and`, `xor`, `slt`, `sltu`,
  `eql`, `deb`.
- I-type: `sys`, `add_i`, `sub_i`, `shl_i`, `shr_i`, `or_i`, `oru_i`,
  `and_i`, `andu_i`, `xor_i`, `xoru_i`, `slt_i`, `sltu_i`, `ldw_i`,
  `ldhw_i`, `ldhwu_i`, `ldb_i`, `ldbu_i`, `stw_i`, `sthw_i`, `stb_i`,
  `jmp_i`, `jmpr_i`, `beq_i`, `bne_i`.

Notes:

- Arithmetic is wrapping 32-bit arithmetic.
- `ldw/ldhw/ldhwu/ldb/ldbu` and `stw/sthw/stb` use `ra + sign_extend(imm16)`
  as a byte address.
- `ldhw` and `ldb` sign-extend; `ldhwu` and `ldbu` zero-extend.
- `jmp`, `jmpr`, `beq`, and `bne` use signed 16-bit word offsets.
- `jmp` and `jmpr` write the return address (`pc + 4`) to `rr`.

## Encoding And Decoding

The top 6 bits always store the opcode.

### R-type

```text
+------------------------------------+
|   6  |  5  |  5  |  5  |    11     |
+------------------------------------+
 opcode  rr    ra    rb     unused
```

### I-type

```text
+------------------------------------+
|   6  |  5  |  5  |       16        |
+------------------------------------+
 opcode  rr    ra       immediate
```

- Encoding is `opcode << 26 | payload`.
- Decoding first reads the opcode, then selects the payload shape:
  - `0x00`: `noop`
  - `0x01..=0x1F`: R-type
  - `0x20..=0x3F`: I-type
- The 16-bit immediate is stored verbatim; signedness depends on the
  instruction.

## Image Format

`.evm` is a sparse chunked image format.

- Header: ASCII magic `EVM1`
- Body: repeated chunks of
  - `addr: u32` big-endian
  - `len: u32` big-endian
  - `len` raw bytes

Notes:

- `addr` is an absolute byte address in the VM address space.
- There is no relocation table, section table, or entry-point field.
- Loading writes each chunk directly into sparse memory.
- Dumping skips zero-filled gaps, so the image only stores populated ranges.
