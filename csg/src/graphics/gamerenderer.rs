use crate::{
    graphics::{
        assets::{
            gputexture::{
                GpuTexture,
                load_texture
            },
            manager::AssetKey
        },
        renderer::Renderer,
        sprite::{
            Sprite,
            SpriteInstance
        }
    },
    prelude::*,
    simulation::{
        world::World
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

    dot: AssetKey<GpuTexture>
} impl GameRenderer {
    pub fn create(ren: &mut Renderer) -> GResult<GameRenderer>{
        let atlas  =ren.asset_manager.create_asset("assets/player.png", load_texture(ren.gc.clone(), "PLAYER".into()))?;
        let player_json = ren.asset_manager.create_json_asset("assets/player.json")?;
        let ps = Sprite::create(atlas, player_json);
        let pi = SpriteInstance::create(4., Vector2::zeros(), super::RenderLayer::Debug, Player::South);

        let dot = ren.asset_manager.create_asset("assets/dot.png", load_texture(ren.gc.clone(), "DOT".into()))?;
        Ok(
            GameRenderer {
                ps,
                pi,
                dot
            }
        )
    }
    pub fn frame(&mut self, ren: &mut Renderer, rp: &mut RenderPass, world: &mut World){
        let mspeed = 1200. * ren.input_manager.get_delta_time() as f32;
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
        
        world.grid.render(rp);
        self.pi.render(&self.ps, ren);
    } 
}
