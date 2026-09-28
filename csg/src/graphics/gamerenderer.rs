use glfw::Key;
use nalgebra::Vector2;

use crate::graphics::{input_mgr::InputManager, renderer::Renderer, sprite::{Sprite, SpriteInstance}};

crate::build_sprite_state_enum!{
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
    pub fn create(ren: &mut Renderer) -> GameRenderer{
        let atlas = ren.asset_manager.create_texture_from_bytes(include_bytes!("../../../temp-assets/player.png"), "PLAYER".into());
        let ps = Sprite::create(atlas, PlayerJSON::from_include(include_str!("../../../temp-assets/player.json")));
        let pi = SpriteInstance::create(&ps, 5., Vector2::zeros(), super::RenderLayer::Debug, Player::South);

        GameRenderer {
            ps,
            pi
        }
    }

    pub fn frame(&mut self, ren: &mut Renderer, im: &InputManager){
        if im.key_down(&Key::Right){
            self.pi.pos.x += 10.;
            self.pi.state = Player::East;
        } else if im.key_down(&Key::Left){
            self.pi.pos.x -= 10.;
            self.pi.state = Player::West;
        } else if im.key_down(&Key::Down){
            self.pi.pos.y += 10.;
            self.pi.state = Player::South;
        } else if im.key_down(&Key::Up){
            self.pi.pos.y -= 10.;
            self.pi.state = Player::North;
        }

        self.pi.render(&self.ps, &mut ren.atlas_renderer);
    }
}