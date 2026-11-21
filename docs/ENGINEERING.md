# Engineering rules

Polyglotic's standards (polyglotic-hq `ENGINEERING.md`), as they apply to a
Rust engine. Where this file is silent, that one holds.

## The same principles

- **Ports and adapters.** `brush-core` is the core: plain Rust that knows no
  GPU, window, file or clock. The simulation, the look and the host are the
  adapters. Here the rule is kept by Cargo: `brush-core` has no dependencies,
  and CI checks it.
- **Clean seams, not frameworks.** One implementation of the simulation, of
  the look and of the host exists, so there is no trait for them yet. A trait
  arrives with the second implementation.
- **The domain's words.** A *sample* is one reading of the pen. A *stroke* is
  the samples from pen-down to pen-up. A *piece* is a canvas's strokes and
  what was done to them. The *paper* is what ink soaks into. *Pointer ink* is
  a mark over captions that isn't simulated. *Setting* is ink drying. Use
  these; add to this list before adding a new word to the code.
- **Every structural choice has a decision record** (`docs/decisions/`):
  the situation, the decision, why, what would reverse it and how. A changed
  decision gets a new record.

## Change discipline

- Every change starts from a written intent: a brief in polyglotic-hq, or an
  issue here. One concern per pull request. Nothing is pushed to `main`.
- What you notice outside the intent is noted in the pull request, not fixed.
- Every bug fix comes with a test that fails without it.
- Delete as well as add.

## The checks (CI runs all of them)

CI runs them on GitHub's machines, on Linux, Windows (the MSVC target and
`gnullvm`) and macOS (`decisions/0014`); never on a runner of ours.

| Check | Command |
|---|---|
| Format | `cargo fmt --all --check` |
| Lints, warnings as errors | `cargo clippy --workspace --all-targets -- -D warnings` |
| Tests | `cargo test --workspace` |
| Docs build without warnings | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` |
| `brush-core` has no dependencies | `cargo tree -p brush-core --edges normal` prints one line |
| Licences and advisories | `cargo deny check` (`deny.toml`) |
| The C header is current | part of the tests (`crates/brush-host/tests/header.rs`) |
| Every `.rs` and `.wgsl` file carries the licence | its first two lines (`decisions/0013`) |
| `THIRD_PARTY_NOTICES.md` is current | `tools/third-party-notices.py` (its first lines) |

Tests that need a GPU or a screen don't run in CI yet: the commands in
`docs/M0.md` are run by hand on real devices. Golden pictures on a software
GPU arrive with M1.

## Tests

- `brush-core`: unit tests for every rule; property-style tests where a rule
  is "never more than" or "always the same".
- Shaders: from M1, each pass on a tiny grid against a plain-Rust reference
  of the same maths.
- Timings are budgets: a benchmark that passes its budget fails the build
  (from M1, on the runners with real GPUs).

## The review checklist

polyglotic-hq's, plus these for this repo:

1. Does anything simulate less than the whole canvas, or follow the brush
   with a region? (`decisions/0004` - refuse it.)
2. Does anything simulate bristles on the CPU beside the GPU's paint? (`0005`)
3. Does `brush-core` still depend on nothing, and hold everything that must
   replay exactly?
4. Does it change `include/brush_host.h`? Is `BRUSH_ABI_VERSION` raised if a
   caller would break?
5. Does a new dependency pass `cargo deny`, and build without a C compiler
   (`0011`)?
6. Does anything keep, log or send what was drawn? Measurements are numbers
   only.
7. Is every `unsafe` block in `brush-host`, with a `SAFETY:` comment that is
   true?
