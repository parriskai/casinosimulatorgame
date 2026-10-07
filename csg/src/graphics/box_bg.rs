use crate::{
    graphics::{
        RenderLayer,
        UvBox,
        assets::{
            AssetDefault,
            DefaultAsset,
            JsonLoadable,
            ReloadableAsset,
            gputexture::GpuTexture,
            manager::AssetKey
        },
        renderer::Renderer
    },
    prelude::*
};
use nalgebra::Vector2;
use serde::Deserialize;


#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub struct BoxBgJSON{
    size: [f32; 2],
    
    nw: UvBox,
    n:  UvBox,
    ne: UvBox,

    w: UvBox,
    c: UvBox,
    e: UvBox,

    sw: UvBox,
    s:  UvBox,
    se: UvBox
} impl ReloadableAsset for BoxBgJSON{
    fn reload(&mut self, data: &[u8]) -> GResult<()> {
        match Self::load(data){
            Ok(data) => {
                let _ = std::mem::replace(self, data);
                Ok(())
            }
            Err(e) => {
                let _ = std::mem::replace(self, <BoxBgJSON as DefaultAsset>::default());
                Err(e)
            }
        }
    }
}
impl DefaultAsset for BoxBgJSON{
    fn default() -> Self {
        BoxBgJSON{size: [16.,16.], .. Default::default()}
    }
}
inventory::submit!(AssetDefault::create::<BoxBgJSON>());

pub struct BoxBg{
    atlas: AssetKey<GpuTexture>,
    json: AssetKey<BoxBgJSON>
} impl BoxBg{
    pub fn create(atlas: AssetKey<GpuTexture>, json: AssetKey<BoxBgJSON>) -> BoxBg{
        BoxBg {
            atlas,
            json
        }
    }

    pub fn draw(&self, pos: (Vector2<f32>, Vector2<f32>, RenderLayer), scale: f32, ren: &mut Renderer) -> GResult<()>{
        let json = ren.asset_manager.get_asset_or_default(self.json).ok_or(GError::GenericErrror)?;

        let tiles = (pos.1 - pos.0) / scale;

        let mut scale = Vector2::new(scale, scale);

        let itiles = tiles.map(|v| v.round().max(2.) as usize);
        if itiles.map(|v| v as f32) != tiles{
            scale = (pos.1 - pos.0).component_div(&tiles);
        }

        let gsize = Vector2::from(json.size).component_mul(&scale);

        let mut current = pos.0;

        // TOP ROW
        ren.atlas_renderer.draw_atlas(self.atlas, json.nw, (current, current + gsize, pos.2));

        current.x += gsize.x;
        for _ in 0..(itiles.x - 2){
            ren.atlas_renderer.draw_atlas(self.atlas, json.n, (current, current + gsize, pos.2));
            current.x += gsize.x;
        }
        ren.atlas_renderer.draw_atlas(self.atlas, json.ne, (current, current + gsize, pos.2));

        current = Vector2::new(pos.0.x, current.y + gsize.y);

        // MIDDLE ROWS

        for _ in 0..(itiles.y - 2){
            ren.atlas_renderer.draw_atlas(self.atlas, json.w, (current, current + gsize, pos.2));

            current.x += gsize.x;
            for _ in 0..(itiles.x - 2){
                ren.atlas_renderer.draw_atlas(self.atlas, json.c, (current, current + gsize, pos.2));
                current.x += gsize.x;
            }
            ren.atlas_renderer.draw_atlas(self.atlas, json.e, (current, current + gsize, pos.2));

            current = Vector2::new(pos.0.x, current.y + gsize.y);
        }

        // Bottom row

        ren.atlas_renderer.draw_atlas(self.atlas, json.sw, (current, current + gsize, pos.2));

        current.x += gsize.x;
        for _ in 0..(itiles.x - 2){
            ren.atlas_renderer.draw_atlas(self.atlas, json.s, (current, current + gsize, pos.2));
            current.x += gsize.x;
        }
        ren.atlas_renderer.draw_atlas(self.atlas, json.se, (current, current + gsize, pos.2));

        Ok(())
    }
}