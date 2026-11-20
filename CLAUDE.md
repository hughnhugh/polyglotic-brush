# Working in polyglotic-brush

Polyglotic's canvas engine (Rust, wgpu). Read `ARCHITECTURE.md` and
`docs/ENGINEERING.md` before changing anything; structural choices are in
`docs/decisions/`.

- **Two rules are legal, not style:** the whole canvas is simulated every
  step, never a region that follows the brush (`docs/decisions/0004`); no
  bristle physics on a second processor (`0005`). Don't add a tile, an active
  area or a dirty rectangle to the simulation, even as an optimisation.
- **`brush-core` has no dependencies.** Don't add one.
- **Pull requests only;** never push to `main`, never force-push, never
  `--no-verify`. One concern per pull request; say what was tested and what
  wasn't.
- **Before pushing:** `cargo fmt --all --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --workspace`, and
  `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`.
- **On the main PC:** builds at below-normal priority with at most 4 jobs
  (`cargo build -j 4`); GPU runs last seconds, because the same card serves
  the live captions; a window that needs a screen runs on a private desktop
  (polyglotic-desktop's `tools/private_desktop.py`), never on the user's.
- **The toolchain** is pinned in `rust-toolchain.toml`. On Windows it is the
  GNU one and needs its `self-contained` folder on PATH (README).
- Plain language; absolute dates.
