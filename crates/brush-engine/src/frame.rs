// Copyright (c) 2026 Polyglotic. All rights reserved.

//! What is drawn each frame. M0: pointer ink over a clear (transparent)
//! target, and, when asked, the simulation stepping beside it so its cost is
//! in the frame's time.

use brush_core::{ribbon, Sample};
use brush_look::{LitPaper, PointerInk};
use brush_sim::FlowSim;

/// M0's ink: near-black, a little see-through, 2 to 14 pixels wide.
const INK: [f32; 4] = [0.05, 0.05, 0.07, 0.92];
const WIDTH: (f32, f32) = (2.0, 14.0);

/// The engine for one target.
#[derive(Debug)]
pub struct Engine {
    ink: PointerInk,
    sim: Option<(FlowSim, LitPaper)>,
    strokes: Vec<Vec<Sample>>,
    changed: bool,
    backdrop: wgpu::Color,
}

impl Engine {
    /// An engine drawing to targets of `format`. With `sim_grid`, a whole-canvas
    /// simulation of that size is kept and stepped by [`Self::frame`].
    #[must_use]
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat, sim_grid: Option<u32>) -> Self {
        let sim = sim_grid.map(|n| {
            let sim = FlowSim::new(device, queue, n);
            let look = LitPaper::new(device, n, sim.paper(), sim.sides());
            (sim, look)
        });
        Self {
            ink: PointerInk::new(device, format),
            sim,
            strokes: Vec::new(),
            changed: true,
            backdrop: wgpu::Color::TRANSPARENT,
        }
    }

    /// Start a new stroke; samples added next belong to it.
    pub fn stroke_begin(&mut self) {
        self.strokes.push(Vec::new());
    }

    /// Add a sample to the stroke in progress (starting one if there is none).
    pub fn stroke_add(&mut self, sample: Sample) {
        if self.strokes.is_empty() {
            self.strokes.push(Vec::new());
        }
        if let Some(stroke) = self.strokes.last_mut() {
            stroke.push(sample);
            self.changed = true;
        }
    }

    /// A white wash under the ink, 0.0 (none, the default) to 1.0, so a test
    /// can see where a transparent canvas is.
    pub fn set_backdrop(&mut self, wash: f32) {
        let a = f64::from(wash.clamp(0.0, 1.0));
        // Premultiplied: white at alpha `a`.
        self.backdrop = wgpu::Color { r: a, g: a, b: a, a };
    }

    /// The target's size changed: the ink is laid out again next frame.
    pub fn resized(&mut self) {
        self.changed = true;
    }

    /// Remove every stroke.
    pub fn clear(&mut self) {
        self.strokes.clear();
        self.changed = true;
    }

    /// Samples held, over all strokes.
    #[must_use]
    pub fn samples(&self) -> usize {
        self.strokes.iter().map(Vec::len).sum()
    }

    /// Encode one frame into `target` (cleared to the backdrop first): the
    /// simulation's `sim_steps` steps and its picture, if there is a
    /// simulation, then the ink. `size` is the target's size in pixels.
    pub fn frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        size: (u32, u32),
        sim_steps: u32,
    ) -> wgpu::CommandBuffer {
        if self.changed {
            let vertices: Vec<[f32; 3]> = self
                .strokes
                .iter()
                .flat_map(|stroke| ribbon(stroke, WIDTH.0, WIDTH.1))
                .map(|v| [v.x, v.y, v.ink])
                .collect();
            self.ink.set(device, queue, &vertices, [size.0 as f32, size.1 as f32], INK);
            self.changed = false;
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        if let Some((sim, look)) = self.sim.as_mut() {
            if sim_steps > 0 {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                for _ in 0..sim_steps {
                    sim.step(&mut pass);
                }
                look.draw(&mut pass, sim.current());
            }
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ink"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(self.backdrop), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.ink.draw(&mut pass);
        }
        encoder.finish()
    }
}
