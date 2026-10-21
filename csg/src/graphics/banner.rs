use nalgebra::Vector2;

use crate::graphics::{RenderLayer, UvBox, assets::{gputexture::GpuTexture, manager::AssetKey}, atlasrender::AtlasRenderer};

pub struct Banner{
    pub asset: AssetKey<GpuTexture>,
    pub size: (f32, f32),
    pub buffer: f32,
    pub pos: (Vector2<f32>, RenderLayer),
} impl Banner{
    pub fn draw(&self, ar: &mut AtlasRenderer){
        ar.draw_atlas(
            self.asset,
            UvBox{u0: 0., u1: 0.5, v0: 0., v1: 1.},
            (
                self.pos.0,
                self.pos.0 + Vector2::new(self.size.0.min(self.buffer), self.size.1),
                self.pos.1
            )
        );

        if self.size.0 > self.buffer{
            ar.draw_atlas(
                self.asset,
                UvBox{u0: 0.5, u1: 1., v0: 0., v1: 1.},
                (
                    self.pos.0 + Vector2::new(self.buffer, 0.),
                    self.pos.0 + Vector2::new(self.size.0, self.size.1),
                    self.pos.1
                )
            );
        }
    }
}