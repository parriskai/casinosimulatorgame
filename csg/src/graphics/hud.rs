
use nalgebra::Vector2;
use crate::simulation::{
    world::World,
};
use crate::graphics::{
    RenderLayer, UvBox, 
    assets::{gputexture::GpuTexture, manager::AssetKey},
    atlasrender::AtlasRenderer,
    textren::Font,
};

pub struct Hud{
    pub asset: AssetKey<GpuTexture>,
    pub font: Font,
    pub size: (f32, f32),
    pub buffer: f32,
    pub pos: (Vector2<f32>, RenderLayer),
} impl Hud{
    pub fn draw(&self, ar: &mut AtlasRenderer, screen: (u32, u32), world: &World){
        let local_pos = Vector2::new(((screen.0 as f32) - self.size.0) / 2., 10.);
          

        ar.draw_atlas(
            self.asset,
                UvBox{u0: 0., u1: 0.5, v0: 0., v1: 1.},
                (
                    local_pos,
                    local_pos + Vector2::new(self.buffer, self.size.1),
                    self.pos.1)
        );
        ar.draw_atlas(
            self.asset,
                UvBox{u0: 0.5, u1: 1., v0: 0., v1: 1.},
                (
                    local_pos + Vector2::new(self.buffer, 0.),
                    local_pos + Vector2::new(self.size.0 - self.buffer, self.size.1),
                    self.pos.1)
        );
        ar.draw_atlas(
            self.asset,
                UvBox{u0: 0.5, u1: 0., v0: 0., v1: 1.},
                (
                    local_pos + Vector2::new(self.size.0 - self.buffer, 0.),
                    local_pos + Vector2::new(self.size.0, self.size.1),
                    self.pos.1)
        );
        let text = format!("House Money: {}    NPCs: {}    Reputation: {}", world.house_money, world.npcs.len(), world.reputation);
        self.font.write_text(
            &text,
            local_pos + Vector2::new(self.buffer, 14.),
            1.,
            RenderLayer::HudText,
            ar,
        )
    }
}
