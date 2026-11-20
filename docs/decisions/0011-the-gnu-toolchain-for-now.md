# 0011 Built with Rust's `x86_64-pc-windows-gnu` toolchain, pinned

- **Status:** accepted
- **Date:** 2026-11-21
- **Decided by:** the building agent (polyglotic-hq project 545), 2026-11-21; reversible, Claude reviews it

## Situation

The main PC had no Rust and no Microsoft C++ build tools on 2026-11-21, and the build tools need an installer with administrator rights. rustup installs at user level; its GNU toolchain carries its own linker.

## Decision

`rust-toolchain.toml` pins Rust 1.100.0. On Windows the host triple is `x86_64-pc-windows-gnu`; the linker and `dlltool` come from the toolchain's own `self-contained` folder (it has to be on PATH: see the README). No crate that needs a C compiler is let in (none is needed today).

## Why

No administrator rights, nothing bought, and wgpu, DX12 and the Win32 calls all build and run (`docs/M0.md`).

## What would reverse it, and how

Shipping in the app's installer, where the MSVC target is the ordinary choice (PDBs for crash reports, no GNU runtime questions): install the build tools and change the triple - no code changes. Decide before M5.
