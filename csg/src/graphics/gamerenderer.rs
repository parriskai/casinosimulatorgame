use glfw::Key;
use nalgebra::Vector2;

use crate::{graphics::{RenderLayer, assets::gputexture::load_texture, renderer::Renderer, sprite::{Sprite, SpriteInstance}},  prelude::*, simulation::{tiles::{FloorType, TileGrid, TILE_SIZE}, world::World}};
use csg_macros::build_sprite_state_enum;

build_sprite_state_enum!{
    pub enum Player{
        North,
        South,
        East,
        West,
    } -> PlayerJSON
}
build_sprite_state_enum!{
    pub enum Floor{
        Carpet = "floor",
    } -> FloorJSON
}

pub struct GameRenderer{
    floor_sprite: Sprite<FloorJSON>,
    ps: Sprite<PlayerJSON>,
    pi: SpriteInstance<PlayerJSON>
} impl GameRenderer {
    pub fn create(ren: &mut Renderer) -> GResult<GameRenderer>{
        let atlas  =ren.asset_manager.create_asset("assets/player.png", load_texture(ren.gc.clone(), "PLAYER".into()))?;
        let player_json = ren.asset_manager.create_json_asset("assets/player.json")?;
        let ps = Sprite::create(atlas, player_json);
        let pi = SpriteInstance::create(5., Vector2::zeros(), super::RenderLayer::Debug, Player::South);

        let floor_atlas = ren.asset_manager.create_asset("assets/floor.png", load_texture(ren.gc.clone(), "FLOOR".into()))?;
        let floor_json = ren.asset_manager.create_json_asset("assets/floor.json")?;
        let floor_sprite = Sprite::create(floor_atlas, floor_json);
        Ok(
            GameRenderer {
                ps,
                pi,
                floor_sprite
            }
        )
    }
    pub fn frame(&mut self, ren: &mut Renderer, world: &World){
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

        self.draw_floors(ren, world);
        self.pi.render(&self.ps, ren);
    }
    fn draw_floors(&self, ren: &mut Renderer, world: &World){
        for y in 0..world.grid.height as i32{
            for x in 0..world.grid.width as i32{
                let tile = world.grid.get(x, y).unwrap();

                let state = match tile.floor{
                    FloorType::None => continue,
                    FloorType::Carpet => Floor::Carpet,
                };
                let pos = TileGrid::tile_to_world(Vector2::new(x,y));
                self.floor_sprite.render(ren, &state, (pos, pos + Vector2::repeat(TILE_SIZE), RenderLayer::Floor));
            }
        }
    }
}
