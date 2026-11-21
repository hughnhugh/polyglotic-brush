# polyglotic-brush

Polyglotic's canvas engine: brush strokes as data, re-made into ink on paper
by a simulation that runs on the learner's own device. Rust on wgpu, shaders
in WGSL. Open source, under the Apache License 2.0.

It is at **M0**: the repository, and the measurements that say whether the
plan holds - a transparent canvas over the caption window, the simulation's
cost on real graphics cards, and the pen's delay. No brush yet.
`docs/M0.md` has what was measured and how to run each test.

- What is where, and the rules between the parts: [ARCHITECTURE.md](ARCHITECTURE.md)
- How changes are made and checked: [docs/ENGINEERING.md](docs/ENGINEERING.md)
- Why it is built this way: [docs/decisions/](docs/decisions/)
- The plan: polyglotic-hq, `projects/520-the-brush-engine/PLAN.md`

## Build

Rust comes from [rustup](https://rustup.rs), at user level; the version is
pinned in `rust-toolchain.toml` and rustup fetches it on the first build.

On Windows, with Microsoft's C++ build tools installed, nothing more is
needed. Without them (they need an administrator; the main PC has none),
build for the `gnullvm` target with llvm-mingw, which unpacks anywhere
(`docs/decisions/0011`):

```
rustup target add x86_64-pc-windows-gnullvm
export LLVM_MINGW=/c/path/to/llvm-mingw-20261117-ucrt-x86_64   # github.com/mstorsjo/llvm-mingw
. tools/env-windows.sh
```

Output then lands under `target/x86_64-pc-windows-gnullvm/`. Don't build the
plain `x86_64-pc-windows-gnu` target: without a full MinGW its binaries
either fail to link (`dlltool.exe: program not found`) or crash at start.

```
cargo build --release -p brush-cli      # .../release/brush.exe
cargo build --release -p brush-host     # .../release/brush_host.dll, for a host app
```

## Check

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

## Run

```
brush adapters                 # the graphics adapters
brush bench --all              # the simulation's cost on each
brush overlay --script         # the transparent canvas, with a made-up stroke
brush pen                      # the pen test
brush floor                    # the two-minute test for integrated graphics
```

Every command prints numbers and adapter names, and nothing else about the
PC or the person. `brush` alone prints the options.

## The C interface

A host app loads `brush_host.dll` and calls the functions in
`include/brush_host.h` (written by cbindgen; a test keeps it current). From
Python, `ctypes.CDLL` is enough: polyglotic-hq's
`projects/545-the-brush-engine-m0/work/owner545.py` does it from a Qt app.

## Licence

Apache License 2.0: [LICENSE](LICENSE) and [NOTICE](NOTICE)
(`docs/decisions/0013`). Each source file says so in its first lines, and CI
checks that every one does. The crates the engine is built from keep their
own licences, listed in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md),
which CI keeps current. Stroke data, when it comes, is separate files under
their own licences (`docs/decisions/0009`).
