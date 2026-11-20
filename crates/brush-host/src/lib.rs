// Copyright (c) 2026 Polyglotic. All rights reserved.

//! The host's side of the engine.
//!
//! - [`ffi`]: the C interface an app loads (`include/brush_host.h`).
//! - `overlay` (Windows): the canvas as its own transparent window, owned by
//!   the host's window, with the pen read on the engine's own thread.
//!
//! Later hosts (the Mac, the web, phones) get their own modules beside
//! `overlay`; the engine underneath is the same.

pub mod ffi;
pub mod stats;

#[cfg(windows)]
pub mod overlay;

pub use stats::Stats;
