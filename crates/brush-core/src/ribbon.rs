// Copyright (c) 2026 Polyglotic. All rights reserved.

//! M0's mark: a stroke's samples as a ribbon of triangles, its width following
//! pressure. It stands in for the brush footprint until M1; nothing is
//! simulated here.

use crate::sample::Sample;

/// A corner of a triangle, in pixels, with how strongly it is inked (0 to 1).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub ink: f32,
}

/// Triangles (three vertices each) for the ribbon through `samples`.
///
/// The width at a sample runs from `min_width` at no pressure to `max_width`
/// at full pressure. Samples on top of the one before are skipped; a stroke
/// with fewer than two distinct points gives a square dot.
#[must_use]
pub fn ribbon(samples: &[Sample], min_width: f32, max_width: f32) -> Vec<Vertex> {
    let mut points: Vec<(f32, f32, f32)> = Vec::with_capacity(samples.len());
    for s in samples {
        let (x, y) = s.pixels();
        let half = 0.5 * (min_width + (max_width - min_width) * s.pressure_unit());
        match points.last() {
            Some(&(px, py, _)) if (px - x).abs() < 1e-3 && (py - y).abs() < 1e-3 => {}
            _ => points.push((x, y, half)),
        }
    }
    let mut out = Vec::new();
    match points.as_slice() {
        [] => {}
        [(x, y, half)] => {
            let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
            let at = |i: usize| Vertex { x: x + corners[i].0 * half, y: y + corners[i].1 * half, ink: 1.0 };
            out.extend([at(0), at(1), at(2), at(0), at(2), at(3)]);
        }
        many => {
            let mut edges: Vec<(Vertex, Vertex)> = Vec::with_capacity(many.len());
            for (i, &(x, y, half)) in many.iter().enumerate() {
                let before = many[i.saturating_sub(1)];
                let after = many[(i + 1).min(many.len() - 1)];
                let (dx, dy) = (after.0 - before.0, after.1 - before.1);
                let len = (dx * dx + dy * dy).sqrt().max(1e-6);
                let (nx, ny) = (-dy / len * half, dx / len * half);
                edges.push((Vertex { x: x + nx, y: y + ny, ink: 1.0 }, Vertex { x: x - nx, y: y - ny, ink: 1.0 }));
            }
            for pair in edges.windows(2) {
                let ((a, b), (c, d)) = (pair[0], pair[1]);
                out.extend([a, b, c, b, d, c]);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::Source;

    fn at(x: f32, y: f32, pressure: f32) -> Sample {
        Sample::from_pixels(x, y, pressure, 0, Source::Pen)
    }

    #[test]
    fn no_samples_no_triangles() {
        assert!(ribbon(&[], 1.0, 9.0).is_empty());
    }

    #[test]
    fn one_point_is_a_dot_as_wide_as_its_pressure_says() {
        let v = ribbon(&[at(10.0, 10.0, 1.0), at(10.0, 10.0, 1.0)], 2.0, 8.0);
        assert_eq!(v.len(), 6);
        let (min, max) = v.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| (lo.min(p.x), hi.max(p.x)));
        assert_eq!((min, max), (6.0, 14.0));
    }

    #[test]
    fn a_straight_line_is_two_triangles_a_segment_and_as_wide_as_asked() {
        let v = ribbon(&[at(0.0, 0.0, 0.0), at(10.0, 0.0, 0.0), at(20.0, 0.0, 1.0)], 2.0, 6.0);
        assert_eq!(v.len(), 12);
        // No pressure: 2 px wide, so 1 px either side of the line.
        assert!((v[0].y.abs() - 1.0).abs() < 1e-5);
        // Full pressure at the end: 6 px wide.
        let end: Vec<&Vertex> = v.iter().filter(|p| (p.x - 20.0).abs() < 1e-5).collect();
        assert!(end.iter().all(|p| (p.y.abs() - 3.0).abs() < 1e-5));
    }

    #[test]
    fn the_ribbon_never_reaches_further_than_half_its_width_from_the_samples() {
        let samples: Vec<Sample> = (0..40)
            .map(|i| at(i as f32 * 7.0, if i % 2 == 0 { 0.0 } else { 30.0 }, (i % 5) as f32 / 4.0))
            .collect();
        for v in ribbon(&samples, 1.0, 12.0) {
            let near = samples.iter().any(|s| {
                let (x, y) = s.pixels();
                ((x - v.x).powi(2) + (y - v.y).powi(2)).sqrt() <= 6.0 + 1e-3
            });
            assert!(near, "{v:?} is off the stroke");
        }
    }
}
