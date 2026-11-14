use crate::{
    graphics::{
        RenderLayer, UvBox, assets::{
            gputexture::{
                GpuTexture,
                load_texture
            },
            manager::AssetKey
        }, renderer::Renderer, 
        hud::Hud,
        textren::{AtlasJSON, Font},
        sprite::{
            Sprite,
            SpriteInstance
        },
        shop::{ShopItem, Shop},
    }, prelude::*, 
    simulation::{
        world::World,
        npc::{DirectionFacing},
        casino_games::{GameType, RouletteTable, SlotMachine},
    }
};

use csg_macros::build_sprite_state_enum;
use nalgebra::Vector2;
use glfw::Key;
use wgpu::RenderPass;

build_sprite_state_enum!{
    pub enum Player{
        North,
        South,
        East,
        West,
    } -> PlayerJSON
}
pub struct GameRenderer{
    ps: Sprite<PlayerJSON>,
    pi: SpriteInstance<PlayerJSON>,

    hud: Hud,
    slot_machine: AssetKey<GpuTexture>,
    roulette_table: AssetKey<GpuTexture>,
    shop: Shop,
} impl GameRenderer {
    pub fn create(ren: &mut Renderer) -> GResult<GameRenderer>{
        let atlas  =ren.asset_manager.create_asset("assets/player.png", load_texture(ren.gc.clone(), "PLAYER".into()))?;
        let player_json = ren.asset_manager.create_json_asset("assets/player.json")?;
        let ps = Sprite::create(atlas, player_json);
        let pi = SpriteInstance::create(4., Vector2::zeros(), super::RenderLayer::Sprites, Player::South);
        let font_texture = ren.asset_manager.create_asset("assets/pixelta.png", load_texture(ren.gc.clone(), "FONT".into()))?;
        let font_json: AtlasJSON = serde_json::from_str(include_str!("../../../generated-assets/pixelta.json")).map_err(|_| GError::GenericErrror)?;
        let font = Font::create(font_json, font_texture);
        let hud_asset = ren.asset_manager.create_asset("assets/hud.png", load_texture(ren.gc.clone(), "HUD".into()))?;
        let shop_bg_asset = ren.asset_manager.create_asset("assets/menu.png", load_texture(ren.gc.clone(), "BOX_BG".into()))?;
        let hud = Hud{asset: hud_asset, size: (800., 64.), pos: (Vector2::new(400.,0.), RenderLayer::Hud), font: font.clone(), buffer: 64. };
        let slot_machine = ren.asset_manager.create_asset("assets/slot_machine.png", load_texture(ren.gc.clone(), "SLOT_MACHINE".into()))?;
        let roulette_table = ren.asset_manager.create_asset("assets/roulette_table.png", load_texture(ren.gc.clone(), "ROULETTE_TABLE".into()))?;
        let items = vec![
            ShopItem::new(GameType::Slots(SlotMachine {}), slot_machine, "Slots".to_string(), 100),
        ];
        let shop = Shop::create(items, shop_bg_asset, font);

        Ok(
            GameRenderer {
                ps,
                pi,
                hud,
                shop,
                slot_machine,
                roulette_table
            }
        )
    }
    pub fn frame(&mut self, ren: &mut Renderer, rp: &mut RenderPass, world: &mut World){
        let mspeed = 1000. * ren.input_manager.get_delta_time() as f32;
        if ren.input_manager.key_down(&Key::Right){
            self.pi.pos.x += mspeed;
            self.pi.state = Player::East;
        } else if ren.input_manager.key_down(&Key::Left){
            self.pi.pos.x -= mspeed;
            self.pi.state = Player::West;
        } else if ren.input_manager.key_down(&Key::Down){
            self.pi.pos.y += mspeed;
            self.pi.state = Player::South;
        } else if ren.input_manager.key_down(&Key::Up){
            self.pi.pos.y -= mspeed;
            self.pi.state = Player::North;
        }

        let dim = ren.gc.get_dim();
        self.hud.draw(&mut ren.atlas_renderer, dim, world);
        self.shop.draw(ren,dim);

        for machine in &world.games{
            let (texture, size) = match machine.kind{
                GameType::Slots(_) => (self.slot_machine, Vector2::new(18. * 4., 24. * 4.)),
                GameType::Roulette(_) => (self.roulette_table, Vector2::new(40. *4., 24. * 4.)),
            };
            let machine_pos = machine.pos;
            ren.atlas_renderer.draw_atlas(texture, UvBox::FULL, (machine_pos, machine_pos + size, RenderLayer::Tile));
        }

        for npc in &world.npcs{
            let direction_facing = match npc.facing{
                DirectionFacing::North => Player::North,
                DirectionFacing::South=> Player::South,
                DirectionFacing::East => Player::East,
                DirectionFacing::West => Player::West,

            };
            let npc_instance = SpriteInstance::create(4., npc.pos, super::RenderLayer::Sprites, direction_facing); 
            npc_instance.render(&self.ps, ren);

        }
        
        world.grid.render(rp);
        self.pi.render(&self.ps, ren);
    } 
}
