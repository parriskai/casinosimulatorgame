use crate::prelude::*;

#[derive(Clone)]
pub struct GraphicsControl{
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue
} impl GraphicsControl {
    pub async fn create() -> GResult<GraphicsControl>{
        let instance =wgpu::Instance::default();
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

        println!("Using GPU: {}", adapter.get_info().name);

        Ok(GraphicsControl {
            instance,
            adapter,
            device,
            queue
        })
    }
}