// Copyright (c) 2026 Polyglotic. All rights reserved.

//! The paper and the ink: GPU passes on a fixed step.
//!
//! **The whole canvas is simulated every step.** There is no region that
//! follows the brush, and there must never be one: `docs/decisions/0004`.
//! M0 holds the flow pass alone, seeded with a square of wet ink, so that its
//! cost can be measured on real devices.

pub mod half;

const FLOW: &str = include_str!("../../../shaders/flow.wgsl");
const STATE: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// The flow simulation on an `n` by `n` grid.
#[derive(Debug)]
pub struct FlowSim {
    n: u32,
    pipeline: wgpu::ComputePipeline,
    /// `[a -> b, b -> a]`: each step reads one set of state and writes the other.
    steps: [wgpu::BindGroup; 2],
    /// The state's views, `[a, b]`, each `[s0, s1, s2, pigment]`.
    views: [[wgpu::TextureView; 4]; 2],
    paper: wgpu::TextureView,
    done: u64,
}

impl FlowSim {
    /// A grid of `n` by `n` cells: paper with a fixed grain, a square of wet
    /// ink a sixth of the canvas wide, still water everywhere.
    #[must_use]
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, n: u32) -> Self {
        let size = wgpu::Extent3d { width: n, height: n, depth_or_array_layers: 1 };
        let texture = |label: &str| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: STATE,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
        };
        let write = |t: &wgpu::Texture, cells: &[[f32; 4]]| {
            let bytes: Vec<u8> =
                cells.iter().flatten().flat_map(|&v| half::from_f32(v).to_le_bytes()).collect();
            queue.write_texture(
                t.as_image_copy(),
                &bytes,
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(n * 8), rows_per_image: Some(n) },
                size,
            );
        };

        // D2Q9's weights: still water at density 0.2, 1.2 inside the square.
        const W: [f32; 9] =
            [4.0 / 9.0, 1.0 / 9.0, 1.0 / 9.0, 1.0 / 9.0, 1.0 / 9.0, 1.0 / 36.0, 1.0 / 36.0, 1.0 / 36.0, 1.0 / 36.0];
        let cells = (n * n) as usize;
        let blob = |i: usize| {
            let (x, y) = (i as u32 % n, i as u32 / n);
            let inside = |v: u32| v >= n / 3 && v < n / 2;
            if inside(x) && inside(y) {
                1.0
            } else {
                0.0
            }
        };
        let rho: Vec<f32> = (0..cells).map(|i| 0.2 + blob(i)).collect();
        let s0: Vec<[f32; 4]> = rho.iter().map(|r| [W[0] * r, W[1] * r, W[2] * r, W[3] * r]).collect();
        let s1: Vec<[f32; 4]> = rho.iter().map(|r| [W[4] * r, W[5] * r, W[6] * r, W[7] * r]).collect();
        let s2: Vec<[f32; 4]> = rho.iter().map(|r| [W[8] * r, 0.0, 0.0, 0.0]).collect();
        let pigment: Vec<[f32; 4]> = (0..cells).map(|i| [blob(i), 0.0, 0.0, 0.0]).collect();
        // The paper: r is how much the fibres block (0 to 0.4), g its grain.
        let mut seed = 0x0545_2026_u32;
        let mut noise = || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed >> 8) as f32 / (1u32 << 24) as f32
        };
        let grain: Vec<[f32; 4]> = (0..cells).map(|_| [noise() * 0.4, noise(), 0.0, 0.0]).collect();

        let paper_texture = texture("paper");
        write(&paper_texture, &grain);
        let paper = paper_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let views = ["a", "b"].map(|side| {
            [("s0", &s0), ("s1", &s1), ("s2", &s2), ("pigment", &pigment)].map(|(name, data)| {
                let t = texture(&format!("{name} {side}"));
                write(&t, data);
                t.create_view(&wgpu::TextureViewDescriptor::default())
            })
        });

        let module = device
            .create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("flow"), source: wgpu::ShaderSource::Wgsl(FLOW.into()) });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("flow"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("pigment, linear"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let layout = pipeline.get_bind_group_layout(0);
        let step = |from: usize, to: usize| {
            let mut entries: Vec<wgpu::BindGroupEntry<'_>> = Vec::with_capacity(10);
            let all = views[from].iter().chain(std::iter::once(&paper)).chain(views[to].iter());
            for (binding, view) in all.enumerate() {
                entries
                    .push(wgpu::BindGroupEntry { binding: binding as u32, resource: wgpu::BindingResource::TextureView(view) });
            }
            entries.push(wgpu::BindGroupEntry { binding: 9, resource: wgpu::BindingResource::Sampler(&sampler) });
            device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("flow step"), layout: &layout, entries: &entries })
        };
        let steps = [step(0, 1), step(1, 0)];
        Self { n, pipeline, steps, views, paper, done: 0 }
    }

    /// The grid's side, in cells.
    #[must_use]
    pub fn size(&self) -> u32 {
        self.n
    }

    /// Steps taken so far.
    #[must_use]
    pub fn steps_done(&self) -> u64 {
        self.done
    }

    /// One fixed step over the whole canvas.
    pub fn step(&mut self, pass: &mut wgpu::ComputePass<'_>) {
        let groups = self.n.div_ceil(8);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.steps[(self.done % 2) as usize], &[]);
        pass.dispatch_workgroups(groups, groups, 1);
        self.done += 1;
    }

    /// The paper: r how much the fibres block, g the grain's height.
    #[must_use]
    pub fn paper(&self) -> &wgpu::TextureView {
        &self.paper
    }

    /// Both sets of state the look can read: `[pigment, water]` for side a
    /// and side b. [`Self::current`] says which one the last step wrote.
    #[must_use]
    pub fn sides(&self) -> [[&wgpu::TextureView; 2]; 2] {
        [[&self.views[0][3], &self.views[0][2]], [&self.views[1][3], &self.views[1][2]]]
    }

    /// Which side holds the newest state: 0 before any step, then 1, 0, 1...
    #[must_use]
    pub fn current(&self) -> usize {
        (self.done % 2) as usize
    }
}
