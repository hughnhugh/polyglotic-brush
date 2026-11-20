// Copyright (c) 2026 Polyglotic. All rights reserved.

//! The engine: what is drawn each frame, and how long the simulation takes.
//! It is given a wgpu device and a target; it knows no window system.

pub mod bench;
pub mod frame;
pub mod gpu;

pub use bench::{bench, BenchRow};
pub use frame::Engine;
