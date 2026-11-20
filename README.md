# polyglotic-brush

Polyglotic's canvas engine: brush strokes as data, re-made into ink on paper
by a simulation that runs on the learner's own device. Rust on wgpu, shaders
in WGSL. Private.

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

On Windows the toolchain is the GNU one (`docs/decisions/0011`): no
Microsoft build tools are needed, but the toolchain's own linker tools have
to be on PATH. Once per shell:

```
rustup set default-host x86_64-pc-windows-gnu
export PATH="$(rustc --print sysroot)/lib/rustlib/x86_64-pc-windows-gnu/bin/self-contained:$PATH"
```

(Git Bash; in PowerShell, `$env:PATH = "$(rustc --print sysroot)\lib\rustlib\x86_64-pc-windows-gnu\bin\self-contained;$env:PATH"`.)
Without it the build stops at `error calling dlltool 'dlltool.exe': program not found`.

```
cargo build --release -p brush-cli      # target/release/brush.exe
cargo build --release -p brush-host     # target/release/brush_host.dll, for a host app
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
