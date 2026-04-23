# Agent Development Guide

Short guidance for coding agents working in this repository.

## Commands

- Build: `cargo build`
- Test: `cargo test`
- Format: `cargo fmt`
- Run the CLI: `cargo run --bin enigmacli -q -- <subcommand> ...`

Useful CLI subcommands:

- `run-image <file.evm>`
- `run-asm <file.esm>`
- `emit-image <file.esm> [out.evm]`

## Project Layout

- `lib/`: core library code for the VM, instruction set, image handling, and
  assembler support
- `enigmacli.rs`: CLI entry point and host-side syscall wiring
- `hello.esm`: small assembly example for quick end-to-end checks
- `notes/`: design notes and plans; treat these as intent, not source of truth

## Working Rules

- Prefer the current Rust code and `Cargo.toml` over older notes when they
  disagree.
- Keep changes aligned with the current public behavior of the CLI and library.
- When changing assembler or image behavior, verify it with a focused test or a
  `run-asm` check.
- Prefer small, targeted changes over broad rewrites unless the task requires
  them.
