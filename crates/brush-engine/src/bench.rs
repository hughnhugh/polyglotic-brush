// Copyright 2026 Polyglotic
// SPDX-License-Identifier: Apache-2.0

//! How long one step of the simulation takes on a device.
//!
//! The method is 520's (`lbm_bench520.py`), so the numbers compare: every step
//! is its own submission; a run ends by waiting for the last one; the figure
//! is the best of three runs, after a warm-up.

use std::time::{Duration, Instant};

use brush_look::LitPaper;
use brush_sim::FlowSim;

/// One grid's result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BenchRow {
    /// The grid's side, in cells.
    pub n: u32,
    /// Steps in each timed run.
    pub steps: u32,
    /// One flow step, in milliseconds.
    pub flow_ms: f64,
    /// One flow step and one lit picture, in milliseconds.
    pub both_ms: f64,
    /// The share of the picture that is dark at the end: a check that the
    /// passes did their work (the wet square is 1/36 of the canvas at first).
    pub inked: f64,
}

impl BenchRow {
    /// How many steps fit in half of a frame at `hz`, with the picture lit once.
    #[must_use]
    pub fn steps_per_frame(&self, hz: f64) -> f64 {
        let half_frame_ms = 500.0 / hz;
        let lit_ms = (self.both_ms - self.flow_ms).max(0.0);
        ((half_frame_ms - lit_ms) / self.flow_ms).max(0.0)
    }
}

/// Time an `n` by `n` simulation. Each timed run lasts about a third of
/// `budget` (8 to 240 steps), so a slow device still finishes.
#[must_use]
pub fn bench(device: &wgpu::Device, queue: &wgpu::Queue, n: u32, budget: Duration) -> BenchRow {
    let mut sim = FlowSim::new(device, queue, n);
    let look = LitPaper::new(device, n, sim.paper(), sim.sides());

    let mut run = |count: u32, lit: bool| -> f64 {
        let start = Instant::now();
        let mut last = None;
        for _ in 0..count {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                sim.step(&mut pass);
                if lit {
                    look.draw(&mut pass, sim.current());
                }
            }
            last = Some(queue.submit([encoder.finish()]));
        }
        let _ = device.poll(wgpu::PollType::Wait { submission_index: last, timeout: None });
        start.elapsed().as_secs_f64() * 1000.0 / f64::from(count.max(1))
    };

    // Warm up: pipelines compiled, memory touched. Then size the runs.
    run(4, true);
    let estimate_ms = run(4, true).max(0.001);
    let per_run_ms = budget.as_secs_f64() * 1000.0 / 6.0;
    let steps = ((per_run_ms / estimate_ms) as u32).clamp(8, 240);
    let best = |run: &mut dyn FnMut() -> f64| (0..3).map(|_| run()).fold(f64::MAX, f64::min);
    let flow_ms = best(&mut || run(steps, false));
    let both_ms = best(&mut || run(steps, true));

    BenchRow { n, steps, flow_ms, both_ms, inked: dark_share(device, queue, look.picture(), n) }
}

/// The share of the picture's pixels whose red is under a half.
fn dark_share(device: &wgpu::Device, queue: &wgpu::Queue, picture: &wgpu::Texture, n: u32) -> f64 {
    let row = (n * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("the picture, read back"),
        size: u64::from(row) * u64::from(n),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_texture_to_buffer(
        picture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(n) },
        },
        wgpu::Extent3d { width: n, height: n, depth_or_array_layers: 1 },
    );
    let index = queue.submit([encoder.finish()]);
    buffer.map_async(wgpu::MapMode::Read, .., |_| {});
    let _ = device.poll(wgpu::PollType::Wait { submission_index: Some(index), timeout: None });
    let Ok(bytes) = buffer.get_mapped_range(..) else {
        return f64::NAN;
    };
    let dark = bytes
        .chunks_exact(row as usize)
        .flat_map(|line| line[..(n * 4) as usize].as_chunks::<4>().0)
        .filter(|pixel| pixel[0] < 128)
        .count();
    dark as f64 / f64::from(n) / f64::from(n)
}

#[cfg(test)]
mod tests {
    use super::BenchRow;

    #[test]
    fn steps_per_frame_leaves_half_the_frame_and_pays_for_the_picture_once() {
        let row = BenchRow { n: 1024, steps: 240, flow_ms: 0.5, both_ms: 0.8, inked: 0.03 };
        // Half of a 60 Hz frame is 8.33 ms; less 0.3 ms for the picture; 0.5 ms a step.
        assert!((row.steps_per_frame(60.0) - 16.066).abs() < 0.01);
        let slow = BenchRow { flow_ms: 40.0, both_ms: 60.0, ..row };
        assert_eq!(slow.steps_per_frame(60.0), 0.0);
    }
}
