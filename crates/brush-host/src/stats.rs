// Copyright (c) 2026 Polyglotic. All rights reserved.

//! What a canvas measured about itself. Numbers only: nothing drawn or
//! written is kept here.

use brush_core::Timings;

/// A canvas's measurements since it opened.
#[derive(Clone, Debug, Default)]
pub struct Stats {
    /// The adapter drawing the canvas.
    pub adapter: String,
    /// The target's format and how its alpha is composited.
    pub format: String,
    pub alpha: String,
    /// The canvas's size in pixels.
    pub size: (u32, u32),
    /// The compositor's refresh rate, as Windows reports it.
    pub refresh_hz: f64,
    /// Frames presented.
    pub frames: u64,
    /// Frames the surface refused (a timeout, a lost or outdated surface).
    pub refused: u64,
    /// Present to present.
    pub frame_interval: Timings,
    /// Waiting for the swap chain to take a new frame.
    pub acquire_wait: Timings,
    /// From having the frame's target to `Present` returning.
    pub frame_work: Timings,
    /// A sample's own time (the pen's, from Windows; or when it was posted)
    /// to `Present` returning for the frame that draws it.
    pub input_to_present: Timings,
    /// The same, to the vertical sync DXGI reports for that present.
    pub input_to_sync: Timings,
    /// Samples by what made them.
    pub pen_samples: u64,
    pub mouse_samples: u64,
    pub touch_samples: u64,
    pub injected_samples: u64,
    /// The pen's ranges as Windows gave them: pressure 0 to 1024, tilt in degrees.
    pub pressure: Option<(u32, u32)>,
    pub tilt_x: Option<(i32, i32)>,
    pub tilt_y: Option<(i32, i32)>,
    /// Pen samples a second while the pen was down.
    pub pen_rate_hz: f64,
    /// DXGI's own count over the run: frames presented, and screen refreshes
    /// that passed meanwhile. More refreshes than presents means frames were
    /// shown twice.
    pub dxgi_presents: Option<u32>,
    pub dxgi_refreshes: Option<u32>,
    /// Anything that did not go as planned, in words.
    pub notes: Vec<String>,
}

impl Stats {
    /// The measurements as lines for a report file.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        let timing = |name: &str, t: &Timings| match t.summary() {
            Some(s) => format!("{name}: {}", s.line()),
            None => format!("{name}: nothing measured"),
        };
        let mut out = vec![
            format!("adapter: {}", self.adapter),
            format!("target: {} x {} {} alpha {}", self.size.0, self.size.1, self.format, self.alpha),
            format!("compositor refresh: {:.2} Hz", self.refresh_hz),
            format!("frames presented: {} (refused {})", self.frames, self.refused),
            timing("frame interval", &self.frame_interval),
            timing("wait for the swap chain", &self.acquire_wait),
            timing("frame work (target to Present returning)", &self.frame_work),
            timing("input to Present returning", &self.input_to_present),
            timing("input to DXGI's sync time", &self.input_to_sync),
            format!(
                "samples: pen {} mouse {} touch {} injected {}",
                self.pen_samples, self.mouse_samples, self.touch_samples, self.injected_samples
            ),
        ];
        if self.refresh_hz > 0.0 {
            let period = 1000.0 / self.refresh_hz;
            out.push(format!(
                "frame intervals over 1.5 refreshes ({:.2} ms): {} of {}",
                period * 1.5,
                self.frame_interval.over(period * 1.5),
                self.frame_interval.len()
            ));
        }
        if let (Some(p), Some(r)) = (self.dxgi_presents, self.dxgi_refreshes) {
            out.push(format!("DXGI: {p} presents over {r} refreshes"));
        }
        if let Some((lo, hi)) = self.pressure {
            out.push(format!("pen pressure seen: {lo} to {hi} (of 1024)"));
        }
        if let (Some(x), Some(y)) = (self.tilt_x, self.tilt_y) {
            out.push(format!("pen tilt seen: x {} to {}, y {} to {} degrees", x.0, x.1, y.0, y.1));
        }
        if self.pen_rate_hz > 0.0 {
            out.push(format!("pen samples a second while down: {:.0}", self.pen_rate_hz));
        }
        out.extend(self.notes.iter().map(|n| format!("note: {n}")));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::Stats;

    #[test]
    fn a_report_counts_slow_frames_against_the_refresh() {
        let mut s = Stats { refresh_hz: 50.0, ..Stats::default() };
        for micros in [20_000, 20_100, 41_000, 19_900] {
            s.frame_interval.push_micros(micros);
        }
        let lines = s.lines();
        assert!(lines.iter().any(|l| l == "frame intervals over 1.5 refreshes (30.00 ms): 1 of 4"), "{lines:?}");
        assert!(lines.iter().any(|l| l == "input to Present returning: nothing measured"));
    }
}
