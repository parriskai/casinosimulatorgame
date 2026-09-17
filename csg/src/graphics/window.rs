use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use wgpu::{CurrentSurfaceTexture, Surface, SurfaceConfiguration};

use super::graphicscontrol::GraphicsControl;
use crate::prelude::*;

pub struct Window{
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    event: GlfwReceiver<(f64, WindowEvent)>,

    pwindow: PWindow,

    gc: GraphicsControl,
    glfw: Glfw,
} impl Window {
    pub fn create() -> GResult<Window>{
        let mut glfw = glfw::init(glfw::fail_on_errors).g_err()?;
        let gc = pollster::block_on(GraphicsControl::create())?;
        let (pwindow, event) = Self::create_glfw_window(&mut glfw)?;
        let (surface, config) = Self::create_surface_unsafe(&gc, &pwindow)?;
        Ok(
            Window {
                glfw,
                gc,
                pwindow,
                event,
                surface,
                config
            }
        )
    }

    fn create_glfw_window(glfw: &mut glfw::Glfw) -> GResult<(PWindow, glfw::GlfwReceiver<(f64, WindowEvent)>)>{
        let (mut window, events) = glfw.create_window(
            512,
            512,
            "Casino Simulator Game",
            glfw::WindowMode::Windowed).ok_or(
                GError::GLFWError(
                    GLFWError::WindowError("Failed to create window!".into())
                )
            )?;
        window.set_all_polling(true);
        window.make_current();
        Ok((window, events))
    }

    fn create_surface_unsafe(gc: &GraphicsControl, window: &glfw::PWindow) -> GResult<(wgpu::Surface<'static>, wgpu::SurfaceConfiguration)>{
        #[allow(deprecated)]
        let surface = unsafe {
            gc.instance
                .create_surface_unsafe(
                    wgpu::SurfaceTargetUnsafe::RawHandle {
                        raw_display_handle: Some(window.raw_display_handle().g_err()?),
                        raw_window_handle: window.raw_window_handle().g_err()?
                    },
                )
                .expect("Failed to create surface")
        };

        let capabilities = surface.get_capabilities(&gc.adapter);

        println!("capabilities: {capabilities:#?}");
        
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let size = window.get_framebuffer_size();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.0.max(1) as u32,
            height: size.1.max(1) as u32,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        surface.configure(&gc.device, &config);

        Ok((surface, config))
    }

    fn render(&mut self){
        // GET Render Target
        let output = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(tex) => tex,
            CurrentSurfaceTexture::Occluded => return,
            _ => panic!("Failed to acquire surface texture"),
        };

        // Create Command Encoder
        let mut encoder = self.gc.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Main Encoder"),
            },
        );

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("Main Render Pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(
                                    wgpu::Color {
                                        r: 0.,
                                        g: 0.,
                                        b: 0.1,
                                        a: 1.0,
                                    },
                                ),
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                },
            ).forget_lifetime();
        }

        self.gc.queue.submit(Some(encoder.finish()));
        self.gc.queue.present(output);
    }

    pub fn frame(&mut self){
        self.render();
        
        self.pwindow.swap_buffers();
    }
}