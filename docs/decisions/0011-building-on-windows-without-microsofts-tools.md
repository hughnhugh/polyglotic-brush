# 0011 On a Windows PC without Microsoft's build tools, build for `gnullvm` with llvm-mingw

- **Status:** accepted
- **Date:** 2026-11-21
- **Decided by:** the building agent (polyglotic-hq project 545), 2026-11-21; reversible, Claude reviews it

## Situation

The main PC and the second PC (gpubox) had no Rust and no Microsoft C++
build tools on 2026-11-21. The build tools need an installer run by an
administrator; rustup does not.

Rust's ordinary answer without them is the `x86_64-pc-windows-gnu` target,
whose toolchain carries a linker. It was tried first and does not work for
this engine (measured 2026-11-21, Rust 1.100.0):

- wgpu's DX12 backend uses the `windows` crate 0.62, whose imports are
  `raw-dylib`: rustc must run a `dlltool`.
- The `dlltool.exe` bundled with the toolchain is GNU's, which starts
  `as.exe`; the bundle has no `as.exe`. The build stops at
  `dlltool.exe: CreateProcess`.
- Pointing rustc at llvm-mingw's `llvm-dlltool` while linking with the
  bundled GNU `ld` builds, but every binary dies at start with an access
  violation (0xc0000005), `brush.exe` and the test programs alike.

## Decision

- `rust-toolchain.toml` pins Rust 1.100.0 and says nothing about the target.
- On a PC without Microsoft's tools: the `x86_64-pc-windows-gnullvm` target,
  linked by llvm-mingw's clang and lld, with llvm-mingw's `llvm-dlltool`
  (`tools/env-windows.sh`; llvm-mingw release 20261117, a zip that unpacks
  anywhere). The host toolchain stays `x86_64-pc-windows-gnu`, which is
  enough for build scripts and proc macros.
- On GitHub's Windows runner, which has Microsoft's tools: the MSVC target.
- No crate that needs a C compiler is let in (none is needed today).

## Why

No administrator, nothing bought, and it works: the tests pass, `brush.exe`
is one 6 MB file needing only Windows 10's own DLLs, and the overlay, the
bench and the C interface all ran from it (`docs/M0.md`).

## What would reverse it, and how

Shipping inside the app's installer (M5), where the MSVC target is the
ordinary choice: PDBs for crash reports, the toolchain every Windows library
is tested with. Install the build tools on the PC that builds releases and
drop `tools/env-windows.sh` from that build; no code changes are expected:
CI's Windows job builds the MSVC target.
