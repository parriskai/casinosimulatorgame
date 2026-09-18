use wgpu::{RenderPass, Texture, TextureFormat, TextureView};

use crate::graphics::{asset_mgr::AssetManager, atlasrender::{self, AtlasRenderer}, graphicscontrol::GraphicsControl};

pub struct Renderer{
    pub gc: GraphicsControl,
    pub asset_manager: AssetManager,
    pub atlas_renderer: AtlasRenderer,
    
    pub depth_texture: Texture,
    pub depth_view: TextureView
} impl Renderer{
    pub fn create(gc: GraphicsControl, sf: TextureFormat) -> Renderer{
        let asset_manager = AssetManager::create(gc.clone());
        let atlas_renderer  = AtlasRenderer::create(gc.clone(), sf);
        let (depth_texture, depth_view) = Self::create_depth_texture(&gc);

        Renderer {
            gc,
            asset_manager,
            atlas_renderer,
            depth_texture,
            depth_view
        }
    }

    fn create_depth_texture(gc: &GraphicsControl) -> (Texture, TextureView){
        let dim = gc.get_dim();
        let depth_texture = gc.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: dim.0,
                height: dim.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&Default::default());
        (depth_texture, depth_view)
    }

    pub fn set_dim(&mut self, dim: (u32, u32)){
        self.gc.set_dim(dim);
        (self.depth_texture, self.depth_view) = Self::create_depth_texture(&self.gc);
        self.atlas_renderer.update_world_to_cvv();
    }

    pub fn finish<'a> (&'a mut self, rp: &mut RenderPass<'a>){
        self.atlas_renderer.render_all(rp, &self.asset_manager);
    }
}