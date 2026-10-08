use crate::{
    graphics::{
        assets::{
            DefaultAsset, gputexture::GpuTexture, manager::AssetManager,
        }, atlasrender::AtlasRenderer, graphicscontrol::GraphicsControl, input_mgr::InputManager, linerenderer::LineRenderer
    }, prelude::*
};
use wgpu::{
    RenderPass,
    Texture,
    TextureFormat,
    TextureView
};
use std::sync::Arc;
#[cfg(any(debug_assertions, feature = "debug_tools"))]
use crate::debug_features::DebugFeatures;

pub struct Renderer{
    pub gc: GraphicsControl,
    pub asset_manager: AssetManager,
    pub atlas_renderer: AtlasRenderer,
    pub line_renderer: LineRenderer,
    
    pub depth_texture: Texture,
    pub depth_view: TextureView,

    pub input_manager: InputManager,

    #[cfg(any(debug_assertions, feature = "debug_tools"))]
    pub debug_features: DebugFeatures
} impl Renderer{
    pub fn create(gc: GraphicsControl, sf: TextureFormat, vfs: Arc<dyn CasinoFS>) -> Renderer{
        let mut asset_manager = AssetManager::create(vfs);
        asset_manager.set_default(GpuTexture::from_di(gc.clone(), DefaultAsset::default(), "MISSING TEXTURE".into()));

        let atlas_renderer  = AtlasRenderer::create(gc.clone(), sf.clone());
        let line_renderer = LineRenderer::create(gc.clone(), sf);

        let (depth_texture, depth_view) = Self::create_depth_texture(&gc);
        
        let input_manager  =InputManager::create();
        
        #[cfg(any(debug_assertions, feature = "debug_tools"))]
        let debug_features = DebugFeatures::default();

        Renderer {
            gc,
            asset_manager,
            atlas_renderer,
            line_renderer,
            depth_texture,
            depth_view,
            input_manager,
            #[cfg(any(debug_assertions, feature = "debug_tools"))]
            debug_features
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
        self.line_renderer.update_world_to_cvv();
    }

    pub fn finish(&mut self, rp: &mut RenderPass<'_>){
        self.atlas_renderer.render_all(rp, &self.asset_manager);
        
        #[cfg(any(debug_assertions, feature = "debug_tools"))]
        if self.debug_features.outline_quads{
            self.atlas_renderer.draw_debug_lines(&mut self.line_renderer);
        }

        self.line_renderer.render_all(rp);
    }
}