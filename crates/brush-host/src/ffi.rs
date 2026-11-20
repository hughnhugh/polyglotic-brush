// Copyright (c) 2026 Polyglotic. All rights reserved.

//! The C interface (`include/brush_host.h`, written by cbindgen; a test fails
//! when the header is stale).
//!
//! The rules of this interface:
//! - a canvas is an opaque pointer, made by `brush_canvas_create` and ended
//!   by `brush_canvas_destroy`, and by nothing else;
//! - every call may come from any one thread of the host's; the canvas does
//!   its own work on its own thread;
//! - the host is never called back: it asks, with `brush_canvas_stats`;
//! - a change that breaks a caller raises `BRUSH_ABI_VERSION`.

/// The interface's version. A host checks it before anything else.
pub const BRUSH_ABI_VERSION: u32 = 1;

/// The canvas lets the pen and the mouse through to what is underneath.
pub const BRUSH_INPUT_OFF: u32 = 0;
/// The canvas takes the pen and the mouse, and draws.
pub const BRUSH_INPUT_ON: u32 = 1;

/// A canvas. Opaque to the host.
#[derive(Debug)]
pub struct BrushCanvas {
    #[cfg(windows)]
    overlay: crate::overlay::Overlay,
}

/// A canvas's numbers since it opened. Times are in microseconds.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BrushStats {
    /// Frames presented, and frames the surface refused.
    pub frames: u64,
    pub refused: u64,
    /// Samples drawn, from any source.
    pub samples: u64,
    /// Present to present: the median and the 99th percentile.
    pub frame_interval_p50_us: u32,
    pub frame_interval_p99_us: u32,
    /// A sample's own time to `Present` returning: median, 95th percentile, longest.
    pub input_to_present_p50_us: u32,
    pub input_to_present_p95_us: u32,
    pub input_to_present_max_us: u32,
    /// The compositor's refresh rate, in thousandths of a hertz.
    pub refresh_millihertz: u32,
}

impl From<&crate::Stats> for BrushStats {
    fn from(stats: &crate::Stats) -> Self {
        let us = |ms: f64| (ms * 1000.0).round().clamp(0.0, f64::from(u32::MAX)) as u32;
        let interval = stats.frame_interval.summary();
        let input = stats.input_to_present.summary();
        Self {
            frames: stats.frames,
            refused: stats.refused,
            samples: stats.pen_samples + stats.mouse_samples + stats.touch_samples + stats.injected_samples,
            frame_interval_p50_us: interval.map_or(0, |s| us(s.p50_ms)),
            frame_interval_p99_us: interval.map_or(0, |s| us(s.p99_ms)),
            input_to_present_p50_us: input.map_or(0, |s| us(s.p50_ms)),
            input_to_present_p95_us: input.map_or(0, |s| us(s.p95_ms)),
            input_to_present_max_us: input.map_or(0, |s| us(s.max_ms)),
            refresh_millihertz: us(stats.refresh_hz),
        }
    }
}

/// The interface's version: `BRUSH_ABI_VERSION` of the library that was loaded.
#[no_mangle]
pub extern "C" fn brush_abi_version() -> u32 {
    BRUSH_ABI_VERSION
}

/// Open a canvas above `owner` (an `HWND`, or 0 for none) at a rectangle in
/// screen pixels, with `input` one of `BRUSH_INPUT_*`.
/// Returns null when it can't be opened (and always, off Windows).
#[no_mangle]
pub extern "C" fn brush_canvas_create(
    owner: isize,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    input: u32,
) -> *mut BrushCanvas {
    #[cfg(windows)]
    {
        use crate::overlay::{Input, Options, Overlay};
        let input = if input == BRUSH_INPUT_ON { Input::On } else { Input::Off };
        let options = Options { owner, rect: (x, y, width, height), input, ..Options::default() };
        match Overlay::open(options) {
            Ok(overlay) => Box::into_raw(Box::new(BrushCanvas { overlay })),
            Err(_) => std::ptr::null_mut(),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (owner, x, y, width, height, input);
        std::ptr::null_mut()
    }
}

/// The canvas's own window (an `HWND`), or 0.
///
/// # Safety
/// `canvas` is null or a pointer from `brush_canvas_create` not yet destroyed.
#[no_mangle]
pub unsafe extern "C" fn brush_canvas_window(canvas: *const BrushCanvas) -> isize {
    // SAFETY: the caller's contract above.
    let Some(canvas) = (unsafe { canvas.as_ref() }) else {
        return 0;
    };
    #[cfg(windows)]
    {
        canvas.overlay.hwnd()
    }
    #[cfg(not(windows))]
    {
        let _ = canvas;
        0
    }
}

/// Move and size the canvas, in screen pixels.
///
/// # Safety
/// `canvas` is null or a pointer from `brush_canvas_create` not yet destroyed.
#[no_mangle]
pub unsafe extern "C" fn brush_canvas_set_rect(canvas: *const BrushCanvas, x: i32, y: i32, width: i32, height: i32) {
    // SAFETY: the caller's contract above.
    let Some(canvas) = (unsafe { canvas.as_ref() }) else {
        return;
    };
    #[cfg(windows)]
    canvas.overlay.set_rect((x, y, width, height));
    #[cfg(not(windows))]
    let _ = (canvas, x, y, width, height);
}

/// Show the canvas (`visible` true) or hide it. A host that hides its own
/// window hides the canvas too: Windows does not do it for an owned window.
///
/// # Safety
/// `canvas` is null or a pointer from `brush_canvas_create` not yet destroyed.
#[no_mangle]
pub unsafe extern "C" fn brush_canvas_set_visible(canvas: *const BrushCanvas, visible: bool) {
    // SAFETY: the caller's contract above.
    let Some(canvas) = (unsafe { canvas.as_ref() }) else {
        return;
    };
    #[cfg(windows)]
    canvas.overlay.set_visible(visible);
    #[cfg(not(windows))]
    let _ = (canvas, visible);
}

/// Take the pen and the mouse (`BRUSH_INPUT_ON`) or let them through.
///
/// # Safety
/// `canvas` is null or a pointer from `brush_canvas_create` not yet destroyed.
#[no_mangle]
pub unsafe extern "C" fn brush_canvas_set_input(canvas: *const BrushCanvas, input: u32) {
    // SAFETY: the caller's contract above.
    let Some(canvas) = (unsafe { canvas.as_ref() }) else {
        return;
    };
    #[cfg(windows)]
    {
        use crate::overlay::Input;
        canvas.overlay.set_input(if input == BRUSH_INPUT_ON { Input::On } else { Input::Off });
    }
    #[cfg(not(windows))]
    let _ = (canvas, input);
}

/// Write the canvas's numbers to `out`. False if either pointer is null.
///
/// # Safety
/// `canvas` is null or a pointer from `brush_canvas_create` not yet destroyed;
/// `out` is null or points to a `BrushStats` the caller owns.
#[no_mangle]
pub unsafe extern "C" fn brush_canvas_stats(canvas: *const BrushCanvas, out: *mut BrushStats) -> bool {
    // SAFETY: the caller's contract above.
    let (Some(canvas), Some(out)) = (unsafe { canvas.as_ref() }, unsafe { out.as_mut() }) else {
        return false;
    };
    #[cfg(windows)]
    {
        *out = BrushStats::from(&canvas.overlay.stats());
    }
    #[cfg(not(windows))]
    {
        let _ = canvas;
        *out = BrushStats::default();
    }
    true
}

/// Close the canvas and free it. Null does nothing.
///
/// # Safety
/// `canvas` is null or a pointer from `brush_canvas_create`, and is not used again.
#[no_mangle]
pub unsafe extern "C" fn brush_canvas_destroy(canvas: *mut BrushCanvas) {
    if !canvas.is_null() {
        // SAFETY: the caller's contract: the pointer came from `Box::into_raw`
        // in `brush_canvas_create` and is given up here.
        drop(unsafe { Box::from_raw(canvas) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_canvases_are_refused_not_crashed_on() {
        // SAFETY: null is allowed by every function's contract.
        unsafe {
            assert_eq!(brush_canvas_window(std::ptr::null()), 0);
            brush_canvas_set_rect(std::ptr::null(), 0, 0, 1, 1);
            brush_canvas_set_input(std::ptr::null(), BRUSH_INPUT_ON);
            brush_canvas_set_visible(std::ptr::null(), false);
            let mut stats = BrushStats::default();
            assert!(!brush_canvas_stats(std::ptr::null(), &mut stats));
            brush_canvas_destroy(std::ptr::null_mut());
        }
        assert_eq!(brush_abi_version(), BRUSH_ABI_VERSION);
    }

    #[test]
    fn stats_cross_as_microseconds() {
        let mut stats =
            crate::Stats { frames: 3, pen_samples: 2, mouse_samples: 1, refresh_hz: 59.94, ..Default::default() };
        for micros in [16_600, 16_700, 16_800] {
            stats.frame_interval.push_micros(micros);
        }
        stats.input_to_present.push_micros(4_250);
        let c = BrushStats::from(&stats);
        assert_eq!((c.frames, c.samples), (3, 3));
        assert_eq!((c.frame_interval_p50_us, c.frame_interval_p99_us), (16_700, 16_800));
        assert_eq!((c.input_to_present_p50_us, c.input_to_present_max_us), (4_250, 4_250));
        assert_eq!(c.refresh_millihertz, 59_940);
    }
}
