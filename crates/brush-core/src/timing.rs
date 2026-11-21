// Copyright 2026 Polyglotic
// SPDX-License-Identifier: Apache-2.0

//! Arithmetic on measured times. The clock belongs to the caller: this only
//! holds the numbers, so a gate is judged the same way everywhere.

/// A set of measured durations, in microseconds.
#[derive(Clone, Debug, Default)]
pub struct Timings {
    micros: Vec<u64>,
}

/// What a set of timings comes to, in milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Summary {
    pub count: usize,
    pub min_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub max_ms: f64,
    pub mean_ms: f64,
}

impl Timings {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_micros(&mut self, micros: u64) {
        self.micros.push(micros);
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.micros.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.micros.is_empty()
    }

    /// How many timings are over `limit_ms`.
    #[must_use]
    pub fn over(&self, limit_ms: f64) -> usize {
        self.micros.iter().filter(|&&m| m as f64 / 1000.0 > limit_ms).count()
    }

    /// The summary, or `None` when nothing was measured.
    /// Percentiles are nearest-rank: p95 of 100 timings is the 95th smallest.
    #[must_use]
    pub fn summary(&self) -> Option<Summary> {
        if self.micros.is_empty() {
            return None;
        }
        let mut sorted = self.micros.clone();
        sorted.sort_unstable();
        let ms = |m: u64| m as f64 / 1000.0;
        let rank = |p: usize| {
            let n = sorted.len();
            let index = (n * p).div_ceil(100).clamp(1, n) - 1;
            ms(sorted[index])
        };
        let total: u64 = sorted.iter().sum();
        Some(Summary {
            count: sorted.len(),
            min_ms: ms(sorted[0]),
            p50_ms: rank(50),
            p95_ms: rank(95),
            p99_ms: rank(99),
            max_ms: ms(sorted[sorted.len() - 1]),
            mean_ms: total as f64 / 1000.0 / sorted.len() as f64,
        })
    }
}

impl Summary {
    /// One line for a report.
    #[must_use]
    pub fn line(&self) -> String {
        format!(
            "n={} min {:.2} p50 {:.2} p95 {:.2} p99 {:.2} max {:.2} mean {:.2} ms",
            self.count, self.min_ms, self.p50_ms, self.p95_ms, self.p99_ms, self.max_ms, self.mean_ms
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_measured_has_no_summary() {
        assert!(Timings::new().summary().is_none());
    }

    #[test]
    fn percentiles_are_nearest_rank() {
        let mut t = Timings::new();
        for ms in (1..=100).rev() {
            t.push_micros(ms * 1000);
        }
        let s = t.summary().unwrap();
        assert_eq!((s.min_ms, s.p50_ms, s.p95_ms, s.p99_ms, s.max_ms), (1.0, 50.0, 95.0, 99.0, 100.0));
        assert_eq!(s.mean_ms, 50.5);
        assert_eq!(t.over(95.0), 5);
    }

    #[test]
    fn one_timing_is_every_percentile() {
        let mut t = Timings::new();
        t.push_micros(2500);
        let s = t.summary().unwrap();
        assert_eq!((s.p50_ms, s.p99_ms, s.max_ms), (2.5, 2.5, 2.5));
    }
}
