// Copyright (c) 2026 Polyglotic. All rights reserved.

//! Finding the graphics adapters and opening one.

/// wgpu's entry point. On Windows the DX12 swap chain is made from a
/// DirectComposition visual, which is what lets a window be transparent
/// (`docs/decisions/0006`).
#[must_use]
pub fn instance(backends: wgpu::Backends) -> wgpu::Instance {
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
    desc.backends = backends;
    desc.backend_options.dx12.presentation_system = wgpu::Dx12SwapchainKind::DxgiFromVisual;
    wgpu::Instance::new(desc)
}

/// Every adapter the instance can see, in the system's order.
#[must_use]
pub fn adapters(instance: &wgpu::Instance) -> Vec<wgpu::Adapter> {
    pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()))
}

/// One line naming an adapter: `NVIDIA GeForce ... | Dx12 | DiscreteGpu | driver`.
#[must_use]
pub fn describe(adapter: &wgpu::Adapter) -> String {
    let info = adapter.get_info();
    format!("{} | {:?} | {:?} | {} {}", info.name, info.backend, info.device_type, info.driver, info.driver_info)
}

/// Open an adapter with wgpu's default limits and no optional features.
///
/// # Errors
/// When the adapter refuses a device.
pub fn open(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
}
