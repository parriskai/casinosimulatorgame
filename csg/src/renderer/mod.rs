pub mod graphicscontrol;
pub mod rendersurface;
pub mod renderlayer;
pub mod gputexture;

use wgpu::wgt::TextureDescriptor;

use crate::{commands::{RendererCommand, RendererCommandReciver}, prelude::*, renderer::{gputexture::{GpuTexture, TextureBuilder}, graphicscontrol::GraphicsControl, rendersurface::RenderSurface}, simulation::world::World, window::Window};

pub struct DepthTextureBuilder(TextureSize);
impl TextureBuilder for DepthTextureBuilder{
    fn get_texture_desc<'a>(&self) -> wgpu::TextureDescriptor<'a> {
        TextureDescriptor{
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: self.0.width,
                height: self.0.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        }
    }
    fn get_view_desc<'a>(&self) -> wgpu::TextureViewDescriptor<'a> {
        Default::default()
    }
}

pub struct Renderer{
    display_surface: RenderSurface<'static>,
    depth_texture: GpuTexture,

    window_size: WindowSize,
    gc: GraphicsControl,
    cmd: RendererCommandReciver,
} impl Renderer{
    pub fn create(window: &Window, cmd: RendererCommandReciver) -> GResult<Renderer>{
        let window_size = window.get_framebuffer_size();
        let gc = pollster::block_on(GraphicsControl::create())?;
        let display_surface = RenderSurface::create(&gc, &window.pwindow, window_size)?;
        let depth_texture = GpuTexture::create(&gc, &DepthTextureBuilder(window_size));

        Ok(
            Renderer {
                display_surface,
                depth_texture,

                window_size,
                gc,
                cmd
            }
        )
    }

    pub fn set_framebuffer_size(&mut self, wsize: WindowSize){
        tracing::info!("Framebuffer set to {wsize:?}");

        self.display_surface.configure(&self.gc, move |cfg|{
            cfg.width = wsize.width;
            cfg.height = wsize.height;
        });
    }

    pub fn update(&mut self){
        let commands: Vec<RendererCommand> = self.cmd.iter().collect();
        for rc in commands{
            match rc{
                RendererCommand::SetFrameBufferSize(wsize) =>{
                    self.set_framebuffer_size(wsize);
                }
            }
        }
    }


    pub fn render(&mut self, world: &mut World) -> GResult<()>{
        // GET Render Target
        let output = match self.display_surface.get_current_texture(&self.gc, true){
            Ok(Some(t)) => t,
            Ok(None) => return Ok(()),
            Err(e) => return Err(e),
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
                                        b: 0.,
                                        a: 1.0,
                                    },
                                ),
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
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
        }

        self.gc.queue.submit(Some(encoder.finish()));
        self.gc.queue.present(output);

        Ok(())
    }
}