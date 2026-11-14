use nalgebra::Vector2;
use crate::prelude::*;
use crate::simulation::{
    world::{World, Money},
    casino_games::GameType,
};
use crate::graphics::{
    RenderLayer, UvBox, 
    assets::{gputexture::GpuTexture, manager::AssetKey},
    atlasrender::AtlasRenderer,
    textren::Font,
    box_bg::BoxBg,
    renderer::Renderer,
};

pub struct Shop{
    items: Vec<ShopItem>,
    selected: Option<usize>,
    was_down: bool,
    font: Font,
    bg: AssetKey<GpuTexture>,
    scale: f32,
    button_size: f32,
}impl Shop{
    pub fn create(items: Vec<ShopItem>, bg: AssetKey<GpuTexture>, font: Font)->Self{
        Self{
            items: items,
            selected: None,
            was_down: false,
            font: font,
            bg: bg,
            scale: 1.,
            button_size: 90.,
        }
    }
    pub fn draw(&self, ren: &mut Renderer, screen: (u32, u32)){ 
        let width = 32. * self.scale;
        let h = 16. * self.scale;
        let left = screen.0 as f32 - width;
        let top = 80.;
        let bottom = screen.1 as f32;

        ren.atlas_renderer.draw_atlas(
            self.bg,
            UvBox{u0: 0., u1: 1., v0: 0., v1: 0.5},
            (
                Vector2::new(left, top),
                Vector2::new(left + width, top + h),
                RenderLayer::Hud,
            ),
        );
    }
}

pub struct ShopItem{
    game_type: GameType,
    icon: AssetKey<GpuTexture>,
    name: String,
    cost: Money,
}impl ShopItem{
    pub fn new(game_type: GameType, icon: AssetKey<GpuTexture>, name: String, cost: Money) -> Self{
        Self{
            game_type: game_type,
            icon: icon,
            name: name,
            cost: cost,
        }
    }
}
