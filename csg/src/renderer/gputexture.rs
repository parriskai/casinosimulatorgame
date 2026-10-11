use wgpu::{TextureDescriptor, TextureViewDescriptor};
use crate::{renderer::graphicscontrol::GraphicsControl};

pub struct GpuTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
} impl GpuTexture{
    pub fn create<B: TextureBuilder>(gc: &GraphicsControl, b: &B) -> GpuTexture{
        Self::create_with_desc(gc, b.get_texture_desc(), b.get_view_desc())
    }

    pub fn create_with_desc(gc: &GraphicsControl, tdesc: TextureDescriptor, vdesc: TextureViewDescriptor) -> GpuTexture{
        let texture = gc.device.create_texture(&tdesc);
        let view = texture.create_view(&vdesc);

        GpuTexture {
            texture,
            view
        }
    }
}

pub trait TextureBuilder {
    fn get_texture_desc<'a>(&self) -> TextureDescriptor<'a>;
    fn get_view_desc<'a>(&self) -> TextureViewDescriptor<'a>;
}