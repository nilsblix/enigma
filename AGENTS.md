# Agent Development Guide

A file for guiding coding agents.

## Commands

- **Build**: `cargo build`
    - Separate projects:
    - **enigmavm** (EVM): `cargo build --bin enigmavm`
    - **enigmaforth**: `cargo build --bin enigmaforth`
- **Test**: `cargo test`
- **Formatting**: `cargo fmt`

## Structure

This project contains libraries and binaries.

`enigmalib.rs` is a single-file Rust library to construct and load a custom
32-bit Virtual Machine.

`enigmavm.rs` is a single-file Virtual Machine using **enigmalib**. This
implementation of the EVM (Enigma Virtual Machine) currently expects a
**Posix** compatible host due to systemcalls being dependant on common **libc**
functions.

`enigmaforth.rs` is a single-file Forth compiler which emits an EVM image (with
a `.evm` extension). It uses **jonesforth** as its main inspiration for key
implementation details.
