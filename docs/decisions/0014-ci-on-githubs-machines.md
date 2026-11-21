# 0014 CI runs on GitHub's own machines: Linux, Windows (MSVC and gnullvm), macOS

- **Status:** accepted; supersedes 0012
- **Date:** 2026-11-21
- **Decided by:** the user, 2026-11-21 (polyglotic-hq DECISIONS #254, with the repository going public, 0013); built by polyglotic-hq project 589

## Situation

CI ran on a runner of our own (0012) because a private repository's jobs on
GitHub's machines cost money. Public, they are free. A self-hosted runner on
a public repository is a risk: anyone can open a pull request, and its code
would run on our machine.

## Decision

- **Every job runs on GitHub's machines;** none on a self-hosted runner.
  `.github/workflows/ci.yml`, on every pull request and every push to `main`:
  - **Linux** (`ubuntu-24.04`): format, the dependency rule, clippy
    `-D warnings`, the build, the tests, docs `-D warnings`, cargo-deny.
  - **Windows** (`windows-latest`), twice: the MSVC target, which GitHub's
    images can build (they carry Microsoft's build tools), and `gnullvm`
    with llvm-mingw (0011). Clippy, the tests, docs, and the release builds
    of `brush` and `brush_host.dll`.
  - **macOS** (`macos-15`): clippy, the build, the tests, docs.
- **Rust** comes from the images' rustup, at the version `rust-toolchain.toml`
  pins. llvm-mingw and cargo-deny are pinned in the workflow and checked
  against their SHA-256 before use. The checkout action is pinned to a commit.
- **Pull requests from forks** run with the event's read-only token and no
  secrets; no job uses a secret, and every job's token can only read. The
  repository asks for approval before a first-time contributor's workflows
  run (its Actions settings).
- **No GPU and no window in CI**, as before: no test opens an adapter.

## Why

Free, three systems instead of one, and it closes 0012's gaps: the MSVC
target and Linux were built nowhere. Nothing of ours runs a stranger's code.

## What it leaves open

- **GPU tests and timing budgets** (docs/ENGINEERING.md, from M1) need real
  graphics cards; GitHub's free machines have none. They stay manual
  (`docs/M0.md`) until a decision says where they run.
- **macOS has no graphics backend compiled in** (wgpu's `metal` feature is
  off): it checks that the engine's code builds and its tests pass, not that
  it draws.

## What would reverse it, and how

The repository going private again (CI would cost money again; 0012 has the
self-hosted design). Never point `runs-on` at a self-hosted runner while the
repository is public.
