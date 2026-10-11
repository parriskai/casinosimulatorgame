use crate::{prelude::*, renderer::graphicscontrol::GraphicsControl};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use glfw::PWindow;
use wgpu::{CurrentSurfaceTexture, Surface, SurfaceConfiguration, SurfaceTexture};

pub struct RenderSurface<'a>{
    surface: Surface<'a>,
    config: SurfaceConfiguration,
} impl<'a> RenderSurface<'a>{
    pub fn create(gc: &GraphicsControl, pwin: &PWindow, wsize: WindowSize) -> GResult<RenderSurface<'a>>{
        #[allow(deprecated)]
        let surface = unsafe {
            gc.instance
                .create_surface_unsafe(
                    wgpu::SurfaceTargetUnsafe::RawHandle {
                        raw_display_handle: Some(pwin.raw_display_handle().g_err()?),
                        raw_window_handle: pwin.raw_window_handle().g_err()?
                    },
                ).g_err()?
        };

        let capabilities = surface.get_capabilities(&gc.adapter);
        
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: wsize.width,
            height: wsize.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 0,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        surface.configure(&gc.device, &config);

        Ok(
            RenderSurface {
                surface,
                config
            }
        )
    }

    pub fn configure<F: FnOnce(&mut SurfaceConfiguration) -> R, R>(&mut self, gc: &GraphicsControl, f: F) -> R{
        let ret = f(&mut self.config);
        self.surface.configure(&gc.device, &self.config);
        ret
    }

    pub fn get_current_texture(&self, gc: &GraphicsControl, retry: bool) -> GResult<Option<SurfaceTexture>>{
        match self.surface.get_current_texture(){
            CurrentSurfaceTexture::Success(tex) => Ok(Some(tex)),
            CurrentSurfaceTexture::Suboptimal(tex) => {
                tracing::warn!("Error while atemting to fetch surface texture (configuration desync)");
                tracing::info!("Atempting to reconfigure...");
                self.surface.configure(&gc.device, &self.config);
                Ok(Some(tex))
            }
            CurrentSurfaceTexture::Outdated => {
                tracing::warn!("Error while atemting to fetch surface texture (configuration desync / outdated)");
                if retry{
                    tracing::info!("Atempting to reconfigure...");
                    self.surface.configure(&gc.device, &self.config);
                    tracing::warn!("Retrying texture fetch...");
                    self.get_current_texture(&gc, false)
                } else {
                    tracing::warn!("Not retrying...");
                    panic!("Failed to acquire surface texture: configuration desync")
                }
            }
            CurrentSurfaceTexture::Occluded => {
                tracing::info!("Failed to fetch surface texutre (occluded)");
                Ok(None)
            }
            CurrentSurfaceTexture::Timeout => {
                tracing::info!("Failed to fetch surface texutre (occluded)");
                Ok(None)
            }
            cst => panic!("Failed to acquire surface texture: {cst:?}")
        }
    }
}