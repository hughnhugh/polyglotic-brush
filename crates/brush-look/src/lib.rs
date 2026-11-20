// Copyright (c) 2026 Polyglotic. All rights reserved.

//! The picture. Two looks in M0:
//! - [`LitPaper`]: the simulated ink over its paper, lit from a height field;
//! - [`PointerInk`]: a stroke's ribbon drawn straight to a target with
//!   premultiplied alpha, with no simulation.
//!
//! Neither knows the simulation's types or any window system: they take
//! texture views and a target.

const LOOK: &str = include_str!("../../../shaders/look.wgsl");
const INK: &str = include_str!("../../../shaders/ink.wgsl");

/// The lit picture of an `n` by `n` simulation, as an `Rgba8Unorm` texture.
#[derive(Debug)]
pub struct LitPaper {
    n: u32,
    pipeline: wgpu::ComputePipeline,
    sides: [wgpu::BindGroup; 2],
    picture: wgpu::Texture,
}

impl LitPaper {
    /// `sides` are the simulation's two sets of `[pigment, water]` views; the
    /// caller says which to draw from each time.
    #[must_use]
    pub fn new(device: &wgpu::Device, n: u32, paper: &wgpu::TextureView, sides: [[&wgpu::TextureView; 2]; 2]) -> Self {
        let picture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("the lit picture"),
            size: wgpu::Extent3d { width: n, height: n, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = picture.create_view(&wgpu::TextureViewDescriptor::default());
        let module = device
            .create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("look"), source: wgpu::ShaderSource::Wgsl(LOOK.into()) });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("look"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let layout = pipeline.get_bind_group_layout(0);
        let group = |[pigment, water]: [&wgpu::TextureView; 2]| {
            let entry = |binding, v| wgpu::BindGroupEntry { binding, resource: wgpu::BindingResource::TextureView(v) };
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("look"),
                layout: &layout,
                entries: &[entry(0, pigment), entry(1, paper), entry(2, water), entry(3, &view)],
            })
        };
        Self { n, pipeline, sides: [group(sides[0]), group(sides[1])], picture }
    }

    /// Light the whole picture from the simulation's side `side` (0 or 1).
    pub fn draw(&self, pass: &mut wgpu::ComputePass<'_>, side: usize) {
        let groups = self.n.div_ceil(8);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.sides[side % 2], &[]);
        pass.dispatch_workgroups(groups, groups, 1);
    }

    /// The picture, `n` by `n`, `Rgba8Unorm`.
    #[must_use]
    pub fn picture(&self) -> &wgpu::Texture {
        &self.picture
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct View {
    size: [f32; 2],
    _pad: [f32; 2],
    colour: [f32; 4],
}

/// Pointer ink: triangles in window pixels, drawn with premultiplied alpha.
#[derive(Debug)]
pub struct PointerInk {
    pipeline: wgpu::RenderPipeline,
    view: wgpu::Buffer,
    group: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    capacity: u64,
    count: u32,
}

impl PointerInk {
    /// Ink for a target of `format`.
    #[must_use]
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let module = device
            .create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("ink"), source: wgpu::ShaderSource::Wgsl(INK.into()) });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ink"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 12,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3],
                })],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let view = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ink view"),
            size: std::mem::size_of::<View>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ink view"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: view.as_entire_binding() }],
        });
        let capacity = 64 * 1024;
        let vertices = Self::buffer(device, capacity);
        Self { pipeline, view, group, vertices, capacity, count: 0 }
    }

    fn buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ink vertices"),
            size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// Replace what is drawn: `vertices` are `[x, y, ink]` in window pixels,
    /// three to a triangle. `colour` is straight (not premultiplied) RGBA.
    pub fn set(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, vertices: &[[f32; 3]], size: [f32; 2], colour: [f32; 4]) {
        let bytes: &[u8] = bytemuck::cast_slice(vertices);
        if bytes.len() as u64 > self.capacity {
            self.capacity = (bytes.len() as u64).next_power_of_two();
            self.vertices = Self::buffer(device, self.capacity);
        }
        queue.write_buffer(&self.vertices, 0, bytes);
        queue.write_buffer(&self.view, 0, bytemuck::bytes_of(&View { size, _pad: [0.0; 2], colour }));
        self.count = vertices.len() as u32;
    }

    /// Draw the ink into `pass`.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if self.count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.group, &[]);
        pass.set_vertex_buffer(0, self.vertices.slice(..u64::from(self.count) * 12));
        pass.draw(0..self.count, 0..1);
    }
}
