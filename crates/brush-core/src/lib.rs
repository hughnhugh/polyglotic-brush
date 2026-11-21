// Copyright 2026 Polyglotic
// SPDX-License-Identifier: Apache-2.0

//! Strokes as data: pen samples, the geometry made from them, and the
//! arithmetic on timings. Plain Rust: no GPU, no I/O and no clock, so every
//! result here is the same on every device.

pub mod ribbon;
pub mod sample;
pub mod timing;

pub use ribbon::{ribbon, Vertex};
pub use sample::{Sample, Source, UNITS_PER_PIXEL};
pub use timing::{Summary, Timings};
