use wgpu::{RenderPass, TextureFormat};

use crate::graphics::{asset_mgr::AssetManager, atlasrender::{self, AtlasRenderer}, graphicscontrol::GraphicsControl};

pub struct Rendeerer{
    pub gc: GraphicsControl,
    pub asset_manager: AssetManager,
    pub atlas_renderer: AtlasRenderer
} impl Rendeerer{
    pub fn create(gc: GraphicsControl, sf: TextureFormat) -> Rendeerer{
        let asset_manager = AssetManager::create(gc.clone());
        let atlas_renderer  = AtlasRenderer::create(gc.clone(), sf);

        Rendeerer {
            gc,
            asset_manager,
            atlas_renderer
        }
    }

    pub fn finish<'a> (&'a mut self, rp: &mut RenderPass<'a>){
        self.atlas_renderer.render_all(rp, &self.asset_manager);
    }
}