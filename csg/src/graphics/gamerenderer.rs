use glfw::Key;
use nalgebra::Vector2;

use crate::{graphics::{assets::gputexture::load_texture, renderer::Renderer, sprite::{Sprite, SpriteInstance}}, prelude::*};
use csg_macros::build_sprite_state_enum;

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
    pi: SpriteInstance<PlayerJSON>
} impl GameRenderer {
    pub fn create(ren: &mut Renderer) -> GResult<GameRenderer>{
        let atlas  =ren.asset_manager.create_asset("assets/player.png", load_texture(ren.gc.clone(), "PLAYER".into()))?;
        let player_json = ren.asset_manager.create_json_asset("assets/player.json")?;
        let ps = Sprite::create(atlas, player_json);
        let pi = SpriteInstance::create(5., Vector2::zeros(), super::RenderLayer::Debug, Player::South);

        Ok(
            GameRenderer {
                ps,
                pi
            }
        )
    }
    pub fn frame(&mut self, ren: &mut Renderer){
        if ren.input_manager.key_down(&Key::Right){
            self.pi.pos.x += 10.;
            self.pi.state = Player::East;
        } else if ren.input_manager.key_down(&Key::Left){
            self.pi.pos.x -= 10.;
            self.pi.state = Player::West;
        } else if ren.input_manager.key_down(&Key::Down){
            self.pi.pos.y += 10.;
            self.pi.state = Player::South;
        } else if ren.input_manager.key_down(&Key::Up){
            self.pi.pos.y -= 10.;
            self.pi.state = Player::North;
        }

        self.pi.render(&self.ps, ren);
    }
}