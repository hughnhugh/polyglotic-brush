/* Copyright (c) 2026 Polyglotic. All rights reserved. */
/* Written by cbindgen from crates/brush-host/src/ffi.rs. Don't edit. */

#ifndef POLYGLOTIC_BRUSH_HOST_H
#define POLYGLOTIC_BRUSH_HOST_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * The interface's version. A host checks it before anything else.
 */
#define BRUSH_ABI_VERSION 1

/**
 * The canvas lets the pen and the mouse through to what is underneath.
 */
#define BRUSH_INPUT_OFF 0

/**
 * The canvas takes the pen and the mouse, and draws.
 */
#define BRUSH_INPUT_ON 1

/**
 * A canvas. Opaque to the host.
 */
typedef struct BrushCanvas BrushCanvas;

/**
 * A canvas's numbers since it opened. Times are in microseconds.
 */
typedef struct BrushStats {
  /**
   * Frames presented, and frames the surface refused.
   */
  uint64_t frames;
  uint64_t refused;
  /**
   * Samples drawn, from any source.
   */
  uint64_t samples;
  /**
   * Present to present: the median and the 99th percentile.
   */
  uint32_t frame_interval_p50_us;
  uint32_t frame_interval_p99_us;
  /**
   * A sample's own time to `Present` returning: median, 95th percentile, longest.
   */
  uint32_t input_to_present_p50_us;
  uint32_t input_to_present_p95_us;
  uint32_t input_to_present_max_us;
  /**
   * The compositor's refresh rate, in thousandths of a hertz.
   */
  uint32_t refresh_millihertz;
} BrushStats;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * The interface's version: `BRUSH_ABI_VERSION` of the library that was loaded.
 */
uint32_t brush_abi_version(void);

/**
 * Open a canvas above `owner` (an `HWND`, or 0 for none) at a rectangle in
 * screen pixels, with `input` one of `BRUSH_INPUT_*`.
 * Returns null when it can't be opened (and always, off Windows).
 */
struct BrushCanvas *brush_canvas_create(intptr_t owner,
                                        int32_t x,
                                        int32_t y,
                                        int32_t width,
                                        int32_t height,
                                        uint32_t input);

/**
 * The canvas's own window (an `HWND`), or 0.
 *
 * # Safety
 * `canvas` is null or a pointer from `brush_canvas_create` not yet destroyed.
 */
intptr_t brush_canvas_window(const struct BrushCanvas *canvas);

/**
 * Move and size the canvas, in screen pixels.
 *
 * # Safety
 * `canvas` is null or a pointer from `brush_canvas_create` not yet destroyed.
 */
void brush_canvas_set_rect(const struct BrushCanvas *canvas,
                           int32_t x,
                           int32_t y,
                           int32_t width,
                           int32_t height);

/**
 * Show the canvas (`visible` true) or hide it. A host that hides its own
 * window hides the canvas too: Windows does not do it for an owned window.
 *
 * # Safety
 * `canvas` is null or a pointer from `brush_canvas_create` not yet destroyed.
 */
void brush_canvas_set_visible(const struct BrushCanvas *canvas, bool visible);

/**
 * Take the pen and the mouse (`BRUSH_INPUT_ON`) or let them through.
 *
 * # Safety
 * `canvas` is null or a pointer from `brush_canvas_create` not yet destroyed.
 */
void brush_canvas_set_input(const struct BrushCanvas *canvas, uint32_t input);

/**
 * Write the canvas's numbers to `out`. False if either pointer is null.
 *
 * # Safety
 * `canvas` is null or a pointer from `brush_canvas_create` not yet destroyed;
 * `out` is null or points to a `BrushStats` the caller owns.
 */
bool brush_canvas_stats(const struct BrushCanvas *canvas, struct BrushStats *out);

/**
 * Close the canvas and free it. Null does nothing.
 *
 * # Safety
 * `canvas` is null or a pointer from `brush_canvas_create`, and is not used again.
 */
void brush_canvas_destroy(struct BrushCanvas *canvas);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* POLYGLOTIC_BRUSH_HOST_H */
