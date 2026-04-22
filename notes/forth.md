# Forth
> Forth implementation using the Enigma Virtual Machine (evm)

This implementation is heavily inspired by Jonesforth.

## Overview

Here is how the forth's source code gets compiled:

### 1. Lexing
Get the next **word**, which is easy due to the grammar being split
by whitespace.

### 2. Lookup
Match the word to a word in the **dictionary**.

The dictionary is basically a reversed-linked-list of words, with a
pseudo-structure such as below:

Keep in mind that this would in reality be stored densly in memory,
as a contiguous array of bytes. This makes it easy for the eventual
forth-rewrite of the forth implementation...

```text
struct Word {
    link: Option<Box<Word>>,
    // These two are stored as a single u8
    flags: u3,
    length: u5,
    // The word's name. 2^5 == 32, therefore the name has to be less than 32
    // bytes.
    name: [u8; unknown_size_less_than_32],
    // The raw instructions to carry out when performing this word.
    definition: [u8; unknown_size],
}
```

The example above is pseudo rust-code, and would not compile due to the unknown
sizes. In our implementation, the unknown-size issue would be resolved with the
usage of `Vec`, and is even simpler in the forth-rewrite due to words being
immutable, and can simply be put in memory during compilation.

**Anyways**, during lookup we run through the dictionary and perform byte-by-byte
comparison of the word to the word from the dictionary, and on match we perform
the instructions laid out in the word's definition, i.e jump the machine's
program counter to the address of the word's definition.

**On no match**, we throw away the word, and panic the program.

### 3. Lex the next word

We lex the next word, therefore performing an iterative process.

## Pipeline

I imagine the compiler working in these steps:

1. Source code (program.forth).
2. The compiler compiles the program to an image (program.evm). This process
   uses enigmalib to build an EVM-Image.
3. The EVM, which uses enigmalib, parses (quite a strong word) the image and
   creates a Machine, and then promply runs the program.

When we in the future want to self-host this Forth, we compile the compiler
(now written in forth) to an EVM-Image. Once we're able to compile the compiler
using the EVM-Image, then we can call the compiler self-hosted.

## TODO

List of things we need to write until we can begin with writing the compiler
(Rust version).

- [ ] Image: We need to be able to dump and load an image onto disk. In our
  case, the image is quite simple and only consists of instructions, which
  start at ByteAddress 0.

  Here is the functionality we need from the Image:
  - 
- [ ] EVM: We need to write the EVM:
    - [ ] Be able to use the 
    - [ ] System calls

## Jonesforth notes

We have a dictionary, which is a linked list of **dictionary entries**.

A variable called LATEST contains a pointer to the most **recently defined word**.

Dictionary entry layout:
- 4 bytes: Pointer to the previous entry.
- 1 byte : Top 3 are flags, and bottom 5 are name length, which implies that
  word names can be maximum 32 u8s.
- N bytes: The actual name layed out in bytes.
- ... bts: The definition, which doesn't contain an end-marker.

### Program and next

Our program consists of a list of function calls, i.e a list of 4 byte
addresses to call. It might look something like this:

addr1
addr2
addr3
...

Let us assign some registers to these addresses, and lets say that we're
executing at addr1.

addr1 <-- i.p. and r10 is at this address
addr2 <-- r11
addr3

We perform LODSL: which sets r10 to r11, and increments r11 by 4. In our IS
this becomes 'xor r10 r11 r0'.

addr1 <-- i.p.
addr2 <-- r10
addr3 <-- r11

We then 'jmp r10'.

addr1
addr2 <-- i.p. and r10
addr3 <-- r11

Note that i.p is at the addresses, and the registers simply contain the
addresses. The machine will therefore never try to execute the addresses as
instructions.

```text
; fn next()
xori r10 11 0
addi r11 r11 4
jmp r10
```

### Return stack (r30)

Each FORTH word's definition will be layed out like this:

- 4 bytes: The codeword, i.e a pointer to some assembly to kickstart word's execution
- ... bts: The rest.

The codeword is word specific, but all FORTH defined words share the same
codeword, with that being `do_colon`.

THe builtin words have a codeword which simply points to the assembly
implementation of the word (i.e codeword_ptr + 4).

```text
; fn push_return_stack(reg: usize)
stw  reg r30 0
subi r30 r30 4
```

```text
; fn pop_return_stack(reg: usize)
addi r30 r30 4
ldw  reg r30 0
```

Here r30 denotes the **return stack ptr**, remember that r31 is the **parameter
stack**.

Note that r30 grows **downwards**.

### Codewords

```text
; fn do_colon()
push_return_stack(r11)
addi r11 r10 4
next()
```

All FORTH words have a shared codeword of `do_colon` and all builtin words have a codeword
which simply points to the builtin's assembly implementation. Note that all builtins have
to end with `next`.

### FORTH builtins

List of builtin words:

- `DROP`
- `SWAP`
- `DUP`
- `OVER`
- `ROT`
- `-ROT`
- `2DROP`, drop two elements of the stack.
- `2DUP`, dupe the top pair of the stack.
- `2SWAP`, swap the two top pairs of the stack.
- `?DUP`, dupe the top of the stack, if non-zero.
- `1+`, increment top of stack.
- `1-`, decrement...
- `4+`
- `4-`
- `+`
- `-`
- `*`
- `/MOD`
- `=`
- `<>`
- `<`
- `>`
- `<=`
- `>=`
- `0=`
- `0<>`
- `0<`
- `0>`
- `0<=`
- `0>=`
- `AND`
- `OR`
- `XOR`
- `INVERT`, bitwise not

As mentioned above, all of these builtins end with `next`.

### Exit

What happens when we want to exit a FORTH word? With builtins we simply call
`next`, but here we perform:

```text
; fn exit
pop_return_stack(r11)
next()
```

### Special builtins

What happens when we want to compile the following FORTH?

```text
: DOUBLE 2 * ;
```

We use a special word called LIT:

```text
+---------------------------+-------+-------+-------+-------+-------+
| (usual header of DOUBLE)  | DOCOL | LIT   | 2     | *     | EXIT  |
+---------------------------+-------+-------+-------+-------+-------+
```

When we encounter a literal, we manipulate r11 to capture and skip the literal.

```text
; fn push_param_stack(reg: usize)
stw  reg r31 0
subi r31 r31 4

; fn pop_param_stack(reg: usize)
addi r31 r31 4
ldw  reg r31 0

; fn literal()
push_param_stack(r11)
addi r11 r11 4
next()
```

Keep in mind that in the diagram above, `LIT` corresponds to the address of the
literal function. We therefore straight up execute the literal assembly, which
puts r11 at `*` which on `next()` performs `*`.

### More words!

Manipulate machine-words:

- `!`, store at addr
- `@`, fetch from addr
- `+!`, store with some offset (add)
- `-!`, store with some offset (sub)

Manipulate individual bytes:

- `!u8`
- `@u8`

## IO

The word `KEY` reads one byte from stdin, and pushes it on the param stack.

If stdin has closed, KEY exits the program by calling ```jmp r0```, which is
why ^D should cleanly exit.
(`-- byte`)

The word `EMIT` writes a single byte to stdout.

The word `WORD` pushes the next word onto the stack. It works by calling `KEY`,
and ignoring all whitespace at first, until it encounters a non-whitespace
byte. It then continues with `KEY` and puts bytes into an internal buffer until
a whitespace is met again, which it then calculates the length of the word, and
returns the address and length of the word on the param stack.
(`-- addr len`)

The word `NUMBER` tries to parse the word at the top of the stack (addr + len)
as a number with some BASE (which is a constant).
(`addr len -- (partial)value num_unconverted`)
On success, num_unconverted is equal to 0.

The word `FIND` looks in the dictionary to find the current word on the
param stack.
(`addr len -- header_addr`)
It basically simply iterates over a linked list, and does byte-by-byte
comparison to the names. One note is that `FIND` ignores dictionary entries
which are marked as FLAG_HIDDEN (part of the three top-most bits of the
flag+name_len byte).

The word `<CFA` converts from a dictionary-header pointer (which is what `FIND`
returns) and returns the codeword pointer.
(`header_addr -- codeword_addr`)

The word `>DFA` converts from a dictionary-header pointer and returns the first
data-field in the definition.
(`header_addr -- data_addr`)

### Summary

List of IO I need to implement for the Enigma Virtual Machine:

- `read` from fd,
- `write` from fd,

## Compilation

Lets define some variables:

- STATE: The current state, can be 0 (immediate-mode) or non-zero (compiling-mode).
- HERE: Points to the next byte of free user-memory.

FORTH has an `INTERPRET` function which takes in words using `WORD`, looking
them up using `FIND`, using `<CFA` to turn them into codewords and deciding
what to do with them based on STATE.

If STATE is zero, FORTH simply executes the words on the fly.

If STATE is non-zero, the interpreter appends the codeword pointer to the next
free byte of user-memory, i.e the variable HERE.

### Flags

Lets discuss the flags used in the flags+len field of the dictionary's entries.

Lets define these flags:
* FLAG_HID = 0b00100000: `FIND` skips these entries in the dictionary.
* FLAG_IMM = 0b01000000: `INTERPRETER` always executes these words, even in
  compile mode.
* FLAG_COM = 0b10000000: 
