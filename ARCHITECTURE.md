# Architecture

Polyglotic's canvas engine. A stroke is data first: pen samples and tool
settings. The picture is always re-made from that data by a paper-and-ink
simulation on a fixed step, lit from a height field, on the learner's own
device. The plan is polyglotic-hq's `projects/520-the-brush-engine/PLAN.md`;
this file is what the code is today (M0).

## The crates

```
crates/
  brush-core/    samples, the ribbon made from them, timing arithmetic.
                 Plain Rust: no GPU, no I/O, no clock, no dependencies.
  brush-sim/     the paper and the ink: GPU passes on a fixed step, over the
                 whole canvas (FlowSim).
  brush-look/    the picture: the lit paper (LitPaper), pointer ink (PointerInk).
  brush-engine/  ties them: Engine::frame(device, queue, target), the
                 benchmark, finding adapters. Knows no window system.
  brush-host/    the C interface (ffi.rs, include/brush_host.h). On Windows:
                 the overlay window, WM_POINTER, the DX12 swap chain (overlay.rs).
  brush-cli/     `brush`: adapters, bench, overlay, pen, floor.
shaders/         WGSL, one file per pass, each headed by its maths and source.
include/         brush_host.h, written by cbindgen.
docs/            decisions/, ENGINEERING.md, M0.md.
```

## The rules between them

Dependencies point one way:

```
brush-cli -> brush-host -> brush-engine -> brush-sim, brush-look -> (wgpu)
                                      \-> brush-core <- brush-host, brush-cli
```

1. **`brush-core` depends on nothing** - no other crate of ours, no crate
   from outside. Everything that must be the same on every device (the file
   format, stroke checks, ink used) lives there. CI fails if its dependency
   list is not empty.
2. **`brush-sim` and `brush-look` don't know each other.** The look is given
   texture views; the engine introduces them.
3. **Only `brush-host` knows a window system.** `brush-engine` is given a
   device and a target.
4. **The host is never called back.** It calls the C interface and polls.
5. **The whole canvas is simulated, always** (`docs/decisions/0004`), and no
   bristle physics runs on a second processor (`0005`). These two are legal
   rules, not style.
6. **`unsafe` only in `brush-host`** (Win32 and the C interface), each block
   with a `SAFETY:` comment; clippy fails the build without one.

## A frame on Windows

The canvas's own thread (`overlay.rs`):

1. wait until the swap chain takes a frame (its waitable object, one frame of
   latency);
2. read every message that arrived, so the pen is read as late as possible:
   `WM_POINTER` with `GetPointerPenInfoHistory` (every reading, with pressure,
   tilt and Windows' own time for it), the mouse, and samples a test posted;
3. `Engine::frame`: the simulation's steps if there is one, then the ink;
4. present; note the times.

## What is measured

`Stats` (and `BrushStats` across the C interface): frame intervals, the wait
for the swap chain, the frame's work, each sample's time to `Present`
returning and to the sync time DXGI reports, and what the pen sent. Numbers
only: nothing drawn is kept or logged.

## Not built yet

The stroke file format (`.pbs`), the deposit pass (ink from the stroke into
the simulation), smoothing and prediction, tips, the anchors that let pointer
ink ride a caption line, drawing only when something changed. M1 onwards.
