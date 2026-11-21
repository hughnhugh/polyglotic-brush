// Copyright 2026 Polyglotic
// SPDX-License-Identifier: Apache-2.0

//! One reading of the pen, in fixed point so a replay never depends on how a
//! float was parsed (the plan's section 4).

/// Positions are kept in 1/64 of a pixel.
pub const UNITS_PER_PIXEL: i32 = 64;

/// What made the sample.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Source {
    Pen = 0,
    Mouse = 1,
    Touch = 2,
}

/// One reading of the pen, mouse or finger.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sample {
    /// Position in 1/64 pixel.
    pub x: i32,
    pub y: i32,
    /// 0 (no pressure) to 65535 (full).
    pub pressure: u16,
    /// Tilt in 1/100 degree, -9000 to 9000.
    pub tilt_x: i16,
    pub tilt_y: i16,
    /// Rotation about the pen's own axis, in 1/100 degree, 0 to 35999.
    pub twist: u16,
    /// Microseconds since the stroke began.
    pub t_us: u32,
    pub source: Source,
}

impl Sample {
    /// A sample from window pixels and a pressure of 0.0 to 1.0.
    /// A mouse has no pressure: pass 0.5 and let the tool decide the width.
    #[must_use]
    pub fn from_pixels(x: f32, y: f32, pressure: f32, t_us: u32, source: Source) -> Self {
        let unit = UNITS_PER_PIXEL as f32;
        let clamped = pressure.clamp(0.0, 1.0);
        Self {
            x: (x * unit).round() as i32,
            y: (y * unit).round() as i32,
            pressure: (clamped * f32::from(u16::MAX)).round() as u16,
            tilt_x: 0,
            tilt_y: 0,
            twist: 0,
            t_us,
            source,
        }
    }

    /// The same sample with the pen's tilt, in degrees (-90 to 90).
    #[must_use]
    pub fn with_tilt(mut self, tilt_x_deg: f32, tilt_y_deg: f32) -> Self {
        let fixed = |deg: f32| (deg.clamp(-90.0, 90.0) * 100.0).round() as i16;
        self.tilt_x = fixed(tilt_x_deg);
        self.tilt_y = fixed(tilt_y_deg);
        self
    }

    /// Position in pixels.
    #[must_use]
    pub fn pixels(&self) -> (f32, f32) {
        let unit = UNITS_PER_PIXEL as f32;
        (self.x as f32 / unit, self.y as f32 / unit)
    }

    /// Pressure as 0.0 to 1.0.
    #[must_use]
    pub fn pressure_unit(&self) -> f32 {
        f32::from(self.pressure) / f32::from(u16::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixels_round_trip_to_a_64th() {
        let s = Sample::from_pixels(10.5, -3.25, 0.5, 0, Source::Mouse);
        assert_eq!((s.x, s.y), (672, -208));
        assert_eq!(s.pixels(), (10.5, -3.25));
    }

    #[test]
    fn pressure_and_tilt_are_clamped_to_their_ranges() {
        let s = Sample::from_pixels(0.0, 0.0, 7.0, 0, Source::Pen).with_tilt(-200.0, 45.5);
        assert_eq!(s.pressure, u16::MAX);
        assert_eq!((s.tilt_x, s.tilt_y), (-9000, 4550));
        let none = Sample::from_pixels(0.0, 0.0, -1.0, 0, Source::Pen);
        assert_eq!(none.pressure, 0);
    }
}
