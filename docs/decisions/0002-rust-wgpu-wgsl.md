# 0002 Rust on wgpu, shaders in WGSL, one Cargo workspace

- **Status:** accepted
- **Date:** 2026-11-21
- **Decided by:** the user with Claude, 2026-11-20 (polyglotic-hq project 520's plan, section 2; DECISIONS #232)

## Situation

The engine has to serve Windows now and the Mac, phones and the web later, with nothing interpreted between the pen and the GPU.

## Decision

Rust, wgpu 30, WGSL; the workspace in `ARCHITECTURE.md`. Dependencies are MIT, Apache-2.0, BSD, ISC, Zlib or Unicode only (`deny.toml`).

## Why

One engine for DX12, Metal, Vulkan and WebGPU.

## What would reverse it, and how

wgpu's transparent DX12 path proving unusable (M0 measures it). The seam is `brush-engine`'s `frame(device, queue, target)`: another GPU layer would replace `brush-sim` and `brush-look`, not `brush-core` or the file format.
