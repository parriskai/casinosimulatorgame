use crate::graphics::{RenderLayer, asset_mgr::TextureKey, atlasrender::AtlasRenderer};
use nalgebra::Vector2;

pub trait SpriteJSON {
    type ENUM;

    fn size(&self) -> Vector2<f32>;
    fn render(&self, ar: &mut AtlasRenderer, tk: TextureKey, state: &Self::ENUM, pos: (Vector2<f32>, Vector2<f32>, RenderLayer));
}

pub struct Sprite<S: SpriteJSON>{
    pub tk: TextureKey,
    pub json: S
} impl<S: SpriteJSON> Sprite<S>{
    pub fn create(tk: TextureKey, json: S) -> Sprite<S>{
        Sprite {
            tk,
            json
        }
    }

    pub fn render(&self, ar: &mut AtlasRenderer, state: &S::ENUM, pos: (Vector2<f32>, Vector2<f32>, RenderLayer)){
        self.json.render(ar, self.tk, state, pos);
    }
}

pub struct SpriteInstance<S: SpriteJSON>{
    pub scale: f32,
    size: Vector2<f32>,
    pub pos: Vector2<f32>,
    pub layer: RenderLayer,
    pub state: S::ENUM
} impl<S: SpriteJSON> SpriteInstance<S>  {
    pub fn create(sprite: &Sprite<S>, scale: f32, pos: Vector2<f32>, layer: RenderLayer, state: S::ENUM) -> SpriteInstance<S>{
        SpriteInstance {
            scale,
            size: sprite.json.size(),
            pos,
            layer,
            state
        }
    }

    pub fn render(&self, sp: &Sprite<S>, ar: &mut AtlasRenderer){
        sp.render(ar, &self.state, (self.pos, self.pos + self.size * self.scale, self.layer));
    }
}