use crate::{graphics::{RenderLayer, UvBox, asset_mgr::TextureKey, flipbook::{Flipbook, FlipbookJSON}, renderer::Renderer, textren::{Font, FontTextureAtlas}}, prelude::*};
use nalgebra::Vector2;
// Were going to let this slide
#[allow(deprecated)]
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use slotmap::Key;
use wgpu::{CurrentSurfaceTexture, Surface, SurfaceConfiguration};
use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use super::graphicscontrol::GraphicsControl;
use std::{io::Cursor, sync::Arc, time::{Instant, SystemTime}};

/// The window, and everything on it
pub struct Window{
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    event: GlfwReceiver<(f64, WindowEvent)>,

    pwindow: PWindow,

    pub renderer: Renderer,
    glfw: Glfw,

    fb: Flipbook,
    t: Instant
} impl Window {
    pub fn create() -> GResult<Window>{
        let mut glfw = glfw::init(glfw::fail_on_errors).g_err()?;

        let (pwindow, event) = Self::create_glfw_window(&mut glfw)?;

        // Saftey: one window means this is the first call
        let gc = pollster::block_on(unsafe{GraphicsControl::create(pwindow.get_framebuffer_size())})?;

        let (surface, config) = Self::create_surface_unsafe(&gc, &pwindow)?;
        let mut renderer = Renderer::create(gc, surface.get_configuration().unwrap().format);

        let fb_json = FlipbookJSON::from_include(include_str!("../../../assets/bob_idle.json"));
        let fb_atlas = renderer.asset_manager.create_texture_from_bytes(
             include_bytes!("../../../assets/bob_idle.png"),
             "ATLAS[BobIDLE]".into());

        let mut fb = Flipbook::create(fb_atlas, fb_json);
        fb.resume(0.);

        Ok(
            Window {
                glfw,
                pwindow,
                event,
                surface,
                config,
                renderer,
                fb,
                t: Instant::now()
            }
        )
    }

    fn create_glfw_window(glfw: &mut glfw::Glfw) -> GResult<(PWindow, glfw::GlfwReceiver<(f64, WindowEvent)>)>{
        let (mut window, events) = glfw.create_window(
            704,
            318,
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
            desired_maximum_frame_latency: 0,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        surface.configure(&gc.device, &config);

        Ok((surface, config))
    }

    fn handle_events(&mut self){
        self.glfw.poll_events();

        let events: Vec<_> = glfw::flush_messages(&self.event).collect();
        
        for event in events{
            match event.1 {
                WindowEvent::Close => {
                    tracing::info!("Window Close Requested");
                    self.pwindow.set_should_close(true)
                },

                WindowEvent::FramebufferSize(_, _) => {
                    let (w, h) = self.pwindow.get_framebuffer_size();
                    let (w, h) = (w.max(1) as u32, h.max(1) as u32);
                    tracing::info!("Framebuffer set to {w}x{h}");

                    self.config.width = w;
                    self.config.height = h;
                    self.surface.configure(&self.renderer.gc.device, &self.config);
                    self.renderer.set_dim((w, h));
                },

                WindowEvent::Size(_, _) => {
                    // We already handle FramebufferSize which is more acurate for what we need it for
                }

                WindowEvent::Refresh => {
                    // Alredy runneing every frame
                }
                
                e => {
                    tracing::warn!("Unhandled window event {e:?}");
                }
            }
        }
    }

    fn render(&mut self){
        // GET Render Target
        let output = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(tex) => tex,
            CurrentSurfaceTexture::Occluded => return,
            CurrentSurfaceTexture::Timeout => return,
            cst => panic!("Failed to acquire surface texture: {cst:?}"),
        };

        // Create Command Encoder
        let mut encoder = self.renderer.gc.device.create_command_encoder(
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
                                        b: 0.,
                                        a: 1.0,
                                    },
                                ),
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.renderer.depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                },
            ).forget_lifetime();
            let ss = self.pwindow.get_framebuffer_size();

            self.renderer.atlas_renderer.draw_atlas(TextureKey::null(), UvBox::FULL, (Vector2::zeros(), Vector2::new(ss.0 as f32, ss.1 as f32), RenderLayer::CLEAR));

            let time = self.t.elapsed().as_secs_f64();
            self.fb.update(time);
            for x in 0..10{
                self.fb.draw(Vector2::new((x * 100) as f32, 0.), 5.0, RenderLayer::TOP, &mut self.renderer.atlas_renderer);
                self.fb.draw(Vector2::new((x * 100) as f32, 200.), 5.0, RenderLayer::TOP, &mut self.renderer.atlas_renderer);
            }

            self.renderer.finish(&mut render_pass);
        }

        self.renderer.gc.queue.submit(Some(encoder.finish()));
        self.renderer.gc.queue.present(output);
    }

    pub fn frame(&mut self){
        self.handle_events();

        self.render();
        
        //self.pwindow.swap_buffers();
    }

    pub fn should_close(&self) -> bool{
        self.pwindow.should_close()
    }
}