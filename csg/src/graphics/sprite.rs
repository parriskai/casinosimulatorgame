use crate::graphics::{
    RenderLayer,
    assets::{
        gputexture::GpuTexture,
        manager::AssetKey
    },
    atlasrender::AtlasRenderer,
    renderer::Renderer
};
use nalgebra::Vector2;

pub trait SpriteJSON {
    type ENUM;

    fn size(&self) -> Vector2<f32>;
    fn render(&self, ar: &mut AtlasRenderer, tk: AssetKey<GpuTexture>, state: &Self::ENUM, pos: (Vector2<f32>, Vector2<f32>, RenderLayer));
}

pub struct Sprite<S: SpriteJSON + 'static>{
    pub tk: AssetKey<GpuTexture>,
    pub json: AssetKey<S>
} impl<S: SpriteJSON + 'static> Sprite<S>{
    pub fn create(tk: AssetKey<GpuTexture>, json: AssetKey<S>) -> Sprite<S>{
        Sprite {
            tk,
            json
        }
    }

    pub fn render(&self, renderer: &mut Renderer, state: &S::ENUM, pos: (Vector2<f32>, Vector2<f32>, RenderLayer)){
        self.render_with_json(renderer.asset_manager.get_asset_or_default(self.json).unwrap(), &mut renderer.atlas_renderer, state, pos);
    }

    fn render_with_json(&self, json: &S, ar: &mut AtlasRenderer, state: &S::ENUM, pos: (Vector2<f32>, Vector2<f32>, RenderLayer)){
        json.render(ar, self.tk, state, pos);
    }
}

pub struct SpriteInstance<S: SpriteJSON>{
    pub scale: f32,
    pub pos: Vector2<f32>,
    pub layer: RenderLayer,
    pub state: S::ENUM
} impl<S: SpriteJSON> SpriteInstance<S>  {
    pub fn create(scale: f32, pos: Vector2<f32>, layer: RenderLayer, state: S::ENUM) -> SpriteInstance<S>{
        SpriteInstance {
            scale,
            pos,
            layer,
            state
        }
    }

    pub fn render(&self, sp: &Sprite<S>, renderer: &mut Renderer){
        let json = renderer.asset_manager.get_asset_or_default(sp.json).unwrap();
        let size = json.size();

        sp.render_with_json(json,&mut renderer.atlas_renderer, &self.state, (self.pos, self.pos + size * self.scale, self.layer));
    }
}