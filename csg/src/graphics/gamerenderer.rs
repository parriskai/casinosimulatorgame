use crate::{
    graphics::{
        RenderLayer, assets::{
            gputexture::{
                GpuTexture,
                load_texture
            },
            manager::AssetKey
        }, banner::Banner, renderer::Renderer, sprite::{
            Sprite,
            SpriteInstance
        }
    }, prelude::*, simulation::world::World
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

    banner: Banner
} impl GameRenderer {
    pub fn create(ren: &mut Renderer) -> GResult<GameRenderer>{
        let atlas  =ren.asset_manager.create_asset("assets/player.png", load_texture(ren.gc.clone(), "PLAYER".into()))?;
        let player_json = ren.asset_manager.create_json_asset("assets/player.json")?;
        let ps = Sprite::create(atlas, player_json);
        let pi = SpriteInstance::create(4., Vector2::zeros(), super::RenderLayer::Sprites, Player::South);

        let banner_asset = ren.asset_manager.create_asset("assets/gold_banner.png", load_texture(ren.gc.clone(), "GOLD_BANNER".into()))?;
        let banner = Banner{asset: banner_asset, size: (200., 64.), pos: (Vector2::new(500., 0.), RenderLayer::Debug), buffer: 64.};
        
        Ok(
            GameRenderer {
                ps,
                pi,
                banner
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

        self.banner.draw(&mut ren.atlas_renderer);
        
        for npc in &world.npcs{
            let npc_instance = SpriteInstance::create(4., npc.pos, super::RenderLayer::Debug, Player::South); 
            npc_instance.render(&self.ps, ren);

        }
        
        world.grid.render(rp);
        self.pi.render(&self.ps, ren);
    } 
}
