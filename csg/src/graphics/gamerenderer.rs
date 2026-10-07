use crate::{
    graphics::{
        RenderLayer,
        UvBox,
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
        tiles::{
            FloorType,
            TILE_SIZE,
            TileGrid
        },
        world::World
    }
};

use csg_macros::build_sprite_state_enum;
use nalgebra::Vector2;
use glfw::Key;

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
    pi: SpriteInstance<PlayerJSON>,

    dot: AssetKey<GpuTexture>
} impl GameRenderer {
    pub fn create(ren: &mut Renderer) -> GResult<GameRenderer>{
        let atlas  =ren.asset_manager.create_asset("assets/player.png", load_texture(ren.gc.clone(), "PLAYER".into()))?;
        let player_json = ren.asset_manager.create_json_asset("assets/player.json")?;
        let ps = Sprite::create(atlas, player_json);
        let pi = SpriteInstance::create(5., Vector2::zeros(), super::RenderLayer::Debug, Player::South);

        let floor_atlas = ren.asset_manager.create_asset("assets/floor.png", load_texture(ren.gc.clone(), "FLOOR".into()))?;
        let floor_json = ren.asset_manager.create_json_asset("assets/floor.json")?;
        let floor_sprite = Sprite::create(floor_atlas, floor_json);

        let dot = ren.asset_manager.create_asset("assets/dot.png", load_texture(ren.gc.clone(), "DOT".into()))?;
        Ok(
            GameRenderer {
                ps,
                pi,
                floor_sprite,
                dot
            }
        )
    }
    pub fn frame(&mut self, ren: &mut Renderer, world: &World, drag: Option<(Vector2<i32>, Vector2<i32>)>){
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

        self.draw_dot(ren);
        self.draw_floors(ren, world);
        self.draw_drag_preview(ren, drag);
        self.pi.render(&self.ps, ren);
    }
    fn draw_dot(&self, ren: &mut Renderer){
        let mouse = ren.input_manager.cursor();
        let half_size = Vector2::new(16., 16.);
        ren.atlas_renderer.draw_atlas(self.dot, UvBox::FULL, (mouse - half_size, mouse + half_size, RenderLayer::Debug));
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
    fn draw_drag_preview(&self, ren: &mut Renderer, drag: Option<(Vector2<i32>, Vector2<i32>)>){
        let Some((start,end)) = drag else{
            return;
        };
        let left = start.x.min(end.x);
        let right = start.x.max(end.x);
        let top = start.y.min(end.y);
        let bottom = start.y.max(end.y);

        for y in top..=bottom{
            for x in left..=right{
                let pos = TileGrid::tile_to_world(Vector2::new(x,y));
                self.floor_sprite.render(ren, &Floor::Carpet, (pos, pos + Vector2::repeat(TILE_SIZE), RenderLayer::FloorPreview));
            }
        }
    } 
}
