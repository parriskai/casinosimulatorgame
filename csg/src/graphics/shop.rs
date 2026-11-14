use nalgebra::Vector2;
use crate::simulation::{
    world::{Money},
    casino_games::GameType,
};
use crate::graphics::{
    RenderLayer, UvBox, 
    assets::{gputexture::GpuTexture, manager::AssetKey},
    textren::Font,
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
            scale: 7.,
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
        ren.atlas_renderer.draw_atlas(
            self.bg,
            UvBox{u0: 1., u1: 0., v0: 1., v1: 0.5},
            (
                Vector2::new(left, top + h),
                Vector2::new(left + width, (bottom)/2.),
                RenderLayer::Hud,
            ),
        );
        ren.atlas_renderer.draw_atlas(
            self.bg,
            UvBox{u0: 1., u1: 0., v0: 2., v1: 0.},
            (
                Vector2::new(left, bottom/2. ),
                Vector2::new(left + width, bottom - h),
                RenderLayer::Hud,
            ),
        );
        for (i, shop_item) in self.items.iter().enumerate(){
            let size = ren.asset_manager.get_asset(shop_item.icon).map(|t| t.texture.size());
            if size == None{
                continue;
            }
            let gap = 10.;
            let x = left + (width - self.button_size)/2.;
            let y= top + h + gap + i as f32 * (self.button_size + gap);
            ren.atlas_renderer.draw_atlas(
                shop_item.icon,
                UvBox::FULL,
                (
                    Vector2::new(x,y),
                    Vector2::new(self.button_size + x, self.button_size+ y),
                    RenderLayer::HudText,
                ),
            );
        }
    }
}

pub struct ShopItem{
    game_type: GameType,
    icon: AssetKey<GpuTexture>,
    name: String,
    cost: Money,
}impl ShopItem{
    pub fn new(game_type: GameType, icon: AssetKey<GpuTexture>,name: String, cost: Money) -> Self{
        Self{
            game_type: game_type,
            icon: icon,
            name: name,
            cost: cost,
        }
    }
}
