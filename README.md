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
- Execution starts at byte address `0x00000000`.
- MMIO begins at `0xF0000000`.

### Register ABI

Enigma is RISC-like and uses a fixed 32-register ABI:

| Register | Alias | Purpose |
| --- | --- | --- |
| `r0` | `zero` | Hard-wired zero register. |
| `r1` | `v0` | Primary return register; syscall number on `sys`. |
| `r2` | `v1` / `a0` | Secondary return register; first argument register. |
| `r3` | `a1` | Second argument register. |
| `r4` | `a2` | Third argument register. |
| `r5` | `a3` | Fourth argument register. |
| `r6` | `a4` | Fifth argument register. |
| `r7` | `a5` | Sixth argument register. |
| `r8` | `t0` | Caller-saved temporary. |
| `r9` | `t1` | Caller-saved temporary. |
| `r10` | `t2` | Caller-saved temporary. |
| `r11` | `t3` | Caller-saved temporary. |
| `r12` | `t4` | Caller-saved temporary. |
| `r13` | `t5` | Caller-saved temporary. |
| `r14` | `t6` | Caller-saved temporary. |
| `r15` | `t7` | Caller-saved temporary. |
| `r16` | `s0` | Callee-saved register. |
| `r17` | `s1` | Callee-saved register. |
| `r18` | `s2` | Callee-saved register. |
| `r19` | `s3` | Callee-saved register. |
| `r20` | `s4` | Callee-saved register. |
| `r21` | `s5` | Callee-saved register. |
| `r22` | `s6` | Callee-saved register. |
| `r23` | `s7` | Callee-saved register. |
| `r24` | `s8` | Callee-saved register. |
| `r25` | `s9` | Callee-saved register. |
| `r26` | `s10` | Callee-saved register. |
| `r27` | `s11` | Callee-saved register. |
| `r28` | `gp` | Global pointer register. |
| `r29` | `lr` | Link register by convention for calls and returns. |
| `r30` | `fp` | Frame pointer register. |
| `r31` | `sp` | Stack pointer; initialized to `0xEFFFFFFC`. |

Notes:

- Only `r0` has enforced special semantics in the register file.
- `r31` is initialized by the machine at reset.
- `sys` uses `r1` as the syscall number, `r2..r7` as up to six arguments, and
  returns results in `r1` and optionally `r2`.

## Instruction Set

All instructions are 32-bit words.

- R-type: `add`, `sub`, `shl`, `shr`, `or`, `and`, `xor`, `slt`, `sltu`, `eql`,
  `deb`.
- I-type: `sys`, `add_i`, `sub_i`, `shl_i`, `shr_i`, `or_i`, `oru_i`, `and_i`,
  `andu_i`, `xor_i`, `xoru_i`, `slt_i`, `sltu_i`, `ldw_i`, `ldhw_i`, `ldhwu_i`,
  `ldb_i`, `ldbu_i`, `stw_i`, `sthw_i`, `stb_i`, `jmp_i`, `jmpr_i`, `beq_i`,
  `bne_i`.

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
