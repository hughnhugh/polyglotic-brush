# 0012 CI runs on our own runner on gpubox, building `gnullvm`; Linux waits

- **Status:** superseded by 0014 (2026-11-21: the repository went public, and CI moved to GitHub's machines)
- **Date:** 2026-11-21
- **Decided by:** Claude (polyglotic-hq DECISIONS #253, reversible), built by polyglotic-hq project 582, 2026-11-21

## Situation

GitHub refused every job of this repository's first CI run (pull request 1,
2026-11-21) in 3-5 seconds: "recent account payments have failed or your
spending limit needs to be increased". The repository is private, the
account's included Actions minutes are spent, and its spending limit is 0.
Polyglotic's other repositories already run on runners we own.

## Decision

- **One runner, this repository's alone:** `gpubox-brush` on the second PC
  (gpubox: Windows 10, Ryzen 5 5600X, GTX 1080). A Windows service under its
  own standard account; labels `self-hosted, Windows, X64, gpubox, brush`.
  The workflow asks for `[self-hosted, brush]`. Its setup and upkeep are in
  polyglotic-hq (`scripts/RUNNERS.md`).
- **It builds `x86_64-pc-windows-gnullvm` with llvm-mingw** (0011): the PC
  has no Microsoft build tools.
- **The toolchain is pinned here and installed by the job:** Rust in
  `rust-toolchain.toml`; rustup, llvm-mingw and cargo-deny in the workflow's
  `env`, each download checked against its SHA-256
  (`tools/ci/windows-toolchain.ps1`). They live in the runner's tool cache;
  PATH changes only for the job.
- **It yields to the rest of gpubox** (`tools/ci/gpubox-gate.ps1`): Idle
  priority for every step, 4 compilers at most, and before each heavy step a
  wait while the PC is booked for a timing run, another Actions job (the
  desktop's) runs there, an agent's test run holds its lock, or under 2 GB
  of memory is free.
- **No GPU in CI:** the card belongs to gpubox's GPU queue. No test opens an
  adapter or a window.
- **Linux is not checked** until we own a Linux runner. The checks that were
  Linux-only (format, the dependency rule, docs, cargo-deny) run in the
  Windows job.

## Why

Free, already set up beside the desktop's runner on the same PC, and the
whole check is a few minutes of CPU. The alternative, raising GitHub's
spending limit, is money and the user's call.

## What it leaves open

- **The MSVC target is built nowhere** until a PC has the build tools
  (before M5, 0011).
- **Linux is built nowhere.**
- **GPU tests and timing budgets** (docs/ENGINEERING.md, from M1) need a
  runner that may use a GPU; this one may not.

## What would reverse it, and how

GitHub's machines paid for (the spending limit raised), or a runner with
Microsoft's tools: point `runs-on` back at them and drop the gate and the
toolchain script there (GitHub's images carry Rust). Removing the runner is
in `scripts/RUNNERS.md` in polyglotic-hq.
