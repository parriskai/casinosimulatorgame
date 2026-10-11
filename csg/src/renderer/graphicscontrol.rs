use wgpu::{Backends, InstanceDescriptor};

use crate::prelude::*;

pub struct GraphicsControl{
    /// WGPU Instance for interacting with the core api
    pub instance: wgpu::Instance,
    /// GPU adapter
    pub adapter: wgpu::Adapter,
    /// GPU controller
    pub device: wgpu::Device,
    /// Command queue
    pub queue: wgpu::Queue
} impl GraphicsControl{
    pub async fn create() -> GResult<GraphicsControl>{
        let instance =wgpu::Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .g_err()?;
        
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Main Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .g_err()?;

        Ok(GraphicsControl {
            instance,
            adapter,
            device,
            queue
        })
    }
}