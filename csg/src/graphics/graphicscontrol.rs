use nalgebra::{Matrix4, Vector3};
use strum::EnumCount;
use wgpu::{Backends, InstanceDescriptor};
use std::sync::{Arc, Mutex, atomic::AtomicU32};
use crate::{graphics::RenderLayer, prelude::*, utils::coordinate_transform};

#[derive(Clone)]
pub struct GraphicsControl{
    /// WGPU Instance for interacting with the core api
    pub instance: wgpu::Instance,
    /// GPU adapter
    pub adapter: wgpu::Adapter,
    /// GPU controller
    pub device: wgpu::Device,
    /// Command queue
    pub queue: wgpu::Queue,
    /// Screen dimensions
    pub dim: Arc<(AtomicU32, AtomicU32)>
} impl GraphicsControl {
    /// Create the first instance of GraphicsControl
    /// SHOULD ONLY BE DONE ONCE! Use clone for all future instances
    pub async unsafe fn create(dim: (i32, i32)) -> GResult<GraphicsControl>{
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
            
        let dim = (dim.0.max(1) as u32, dim.1.max(1) as u32);

        Ok(GraphicsControl {
            instance,
            adapter,
            device,
            queue,
            dim: Arc::new((AtomicU32::new(dim.0), AtomicU32::new(dim.1)))
        })
    }

    pub fn set_dim(&self, dim: (u32, u32)){
        self.dim.0.store(dim.0, std::sync::atomic::Ordering::Relaxed);
        self.dim.1.store(dim.1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_dim(&self) -> (u32, u32){
        (
            self.dim.0.load(std::sync::atomic::Ordering::Relaxed),
            self.dim.1.load(std::sync::atomic::Ordering::Relaxed)
        )
    }

    pub fn get_world_to_cvv(&self) -> Matrix4<f32>{
        let dim = self.get_dim();
        coordinate_transform(
            Vector3::new(0., 0., 0.),
            Vector3::new(dim.0 as f32, dim.1 as f32, RenderLayer::COUNT as f32),
            Vector3::new(-1., 1., 1.),
            Vector3::new(1., -1., 0.)
        )
    }

    pub fn log_info(&self){
        let ainfo = self.adapter.get_info();
        tracing::info!(target: "GraphicsControl", "Adapter::Name {}", ainfo.name);
        tracing::info!(target: "GraphicsControl", "Adapter::Vendor x{:X}", ainfo.vendor);
        tracing::info!(target: "GraphicsControl", "Adapter::Device x{:X}", ainfo.device);
        tracing::info!(target: "GraphicsControl", "Adapter::Device Type {:?}", ainfo.device_type);
        tracing::info!(target: "GraphicsControl", "Adapter::Decive PCI {}", ainfo.device_pci_bus_id);
        tracing::info!(target: "GraphicsControl", "Adapter::Driver {}", ainfo.driver);
        tracing::info!(target: "GraphicsControl", "Adapter::Driver Info {}", ainfo.driver_info);
        tracing::info!(target: "GraphicsControl", "Adapter::Backend {:?}", ainfo.backend);
        tracing::info!(target: "GraphicsControl", "Adapter::Subgroup Min Size {}", ainfo.subgroup_min_size);
        tracing::info!(target: "GraphicsControl", "Adapter::Subgroup Max Size {}", ainfo.subgroup_max_size);
        
        match ainfo.transient_saves_memory{
            None => tracing::info!(target: "GraphicsControl", "Adapter::Transient Saves Memory UNKNOWN"),
            Some(false) => tracing::info!(target: "GraphicsControl", "Adapter::Transient Saves Memory NO"),
            Some(true) => tracing::info!(target: "GraphicsControl", "Adapter::Transient Saves Memory YES")
        }

        match ainfo.limit_bucket{
            None => tracing::info!(target: "GraphicsControl", "Adapter::Limit Bucket NONE"),
            Some(lb) => {
                tracing::info!(target: "GraphicsControl", "Adapter::Limit Bucket Name {}", lb.name);
                tracing::info!(target: "GraphicsControl", "Adapter::Limit Bucket Limits {:?}", lb.raw_limits);
                tracing::info!(target: "GraphicsControl", "Adapter::Limit Bucket Features {:?}", lb.raw_features);
                tracing::info!(target: "GraphicsControl", "Adapter::Limit Bucket Subgroup Min Size {}", lb.raw_subgroup_min_size);
                tracing::info!(target: "GraphicsControl", "Adapter::Limit Bucket Subgroup Max Size {}", lb.raw_subgroup_max_size);
            }
        }
    }
}