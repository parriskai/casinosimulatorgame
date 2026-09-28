use nalgebra::Vector2;
use serde::Deserialize;

use crate::graphics::{RenderLayer, UvBox, asset_mgr::TextureKey, atlasrender::AtlasRenderer};

#[derive(Debug, Clone, Copy, Deserialize)]
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
} impl BoxBgJSON{
    pub fn from_include(s: &str) -> BoxBgJSON{
        serde_json::from_str(s).unwrap()
    }
}

pub struct BoxBg{
    atlas: TextureKey,
    json: BoxBgJSON
} impl BoxBg{
    pub fn create(atlas: TextureKey, json: BoxBgJSON) -> BoxBg{
        BoxBg {
            atlas,
            json
        }
    }

    pub fn draw(&self, pos: (Vector2<f32>, Vector2<f32>, RenderLayer), scale: f32, ar: &mut AtlasRenderer){
        let tiles = (pos.1 - pos.0) / scale;

        let mut scale = Vector2::new(scale, scale);

        let itiles = tiles.map(|v| v.round().max(2.) as usize);
        if itiles.map(|v| v as f32) != tiles{
            scale = (pos.1 - pos.0).component_div(&tiles);
        }

        let gsize = Vector2::from(self.json.size).component_mul(&scale);

        let mut current = pos.0;

        // TOP ROW
        ar.draw_atlas(self.atlas, self.json.nw, (current, current + gsize, pos.2));

        current.x += gsize.x;
        for _ in 0..(itiles.x - 2){
            ar.draw_atlas(self.atlas, self.json.n, (current, current + gsize, pos.2));
            current.x += gsize.x;
        }
        ar.draw_atlas(self.atlas, self.json.ne, (current, current + gsize, pos.2));

        current = Vector2::new(pos.0.x, current.y + gsize.y);

        // MIDDLE ROWS

        for _ in 0..(itiles.y - 2){
            ar.draw_atlas(self.atlas, self.json.w, (current, current + gsize, pos.2));

            current.x += gsize.x;
            for _ in 0..(itiles.x - 2){
                ar.draw_atlas(self.atlas, self.json.c, (current, current + gsize, pos.2));
                current.x += gsize.x;
            }
            ar.draw_atlas(self.atlas, self.json.e, (current, current + gsize, pos.2));

            current = Vector2::new(pos.0.x, current.y + gsize.y);
        }

        // Bottom row

        ar.draw_atlas(self.atlas, self.json.sw, (current, current + gsize, pos.2));

        current.x += gsize.x;
        for _ in 0..(itiles.x - 2){
            ar.draw_atlas(self.atlas, self.json.s, (current, current + gsize, pos.2));
            current.x += gsize.x;
        }
        ar.draw_atlas(self.atlas, self.json.se, (current, current + gsize, pos.2));
    }
}