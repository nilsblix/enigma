# Enigma

> Some thoughts about the current design:

32-bit, big-endian Virtual Machine.

32 registers (r0 is read/write 0, and r31 is initialized as a stack-pointer).

This VM is OS-less, i.e it simply emulates some imaginary cpu-architecture.

## We have __two__ types of instructions:

### R-type

Bit packed as such:
```ini
+------------------------------------+
|   6  |  5  |  5  |  5  |    11     |
+------------------------------------+
 opcode  rr    ra    rb     packing (unused)
```

Note that packing is unused.

### I-type
Bit packed as such:
```text
+------------------------------------+
|   6  |  5  |  5  |       16        |
+------------------------------------+
 opcode  rr    ra       immediate
```

## Address space

Load/store instructions can access the entire memory space. It is up to the
caller to manage that memory. This paradigm can lead to some sketchy programs
if not handled correctly, given that there is no internal distinction between
different segments, such as _text_, _data_, _heap_ or _stack_.

Register no. 32 (r31) is the designated stack-pointer, and therefore gets
initialized to the stack beginning, which is `0xEFFFFFC`. That is pretty much
the only segment known in the source-code of the VM. MMIO gets mapped above the
stack, i.e addresses starting with `0xF...` are inherently IO.

The above is only a convention however. Your program can do whatever it wants.
Everything is mutable here.

## Memory mapping

This VM is 32-bit, i.e `~4GB` address space. We don't allocate the entire 4GB
upfront, but instead allocate __blocks__ of `2^16 bytes` each. This way we can
designate what each block does.

Currently each block can be __Empty__, __Memory__ mapped (64 KB), or __Io__
mapped.

__Io__ means that that address block (2^16 bytes) is mapped to some
__IoController__ which on read/write/tick performs some side-effect (actually
only on write currently, but might be changed in the future).
