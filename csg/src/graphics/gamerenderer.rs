use glfw::Key;
use nalgebra::Vector2;
use std::time::Instant;
use slotmap::Key as _;

use crate::graphics::{input_mgr::InputManager, renderer::Renderer, sprite::{Sprite, SpriteInstance}};
use crate::graphics::{RenderLayer, UvBox, asset_mgr::TextureKey};
use crate::simulation::world::World;


pub struct GameRenderer{
    world: World,
    last_frame: Instant,
    spawn_timer: f32,
} 
impl GameRenderer {
    pub fn create(ren: &mut Renderer) -> GameRenderer{
        GameRenderer {
            world: World::new(),
            last_frame: Instant::now(),
            spawn_timer: 0.0,
        }
    }

    pub fn frame(&mut self, ren: &mut Renderer, im: &InputManager){
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.spawn_timer += dt;
        if self.spawn_timer > 3.0 {
            self.world.spawn_npc();
            self.spawn_timer = 0.0;
        }

        self.world.update(dt);

        // Draw machines (big boxes)
        for m in &self.world.machines {
            let size = Vector2::new(48.0, 64.0);
            ren.atlas_renderer.draw_atlas(TextureKey::null(), UvBox::FULL, (m.pos, m.pos + size, RenderLayer::Clear));
        }

        // Draw NPCs (small boxes, on top)
        for npc in &self.world.npcs {
            let size = Vector2::new(24.0, 32.0);
            ren.atlas_renderer.draw_atlas(TextureKey::null(), UvBox::FULL, (npc.pos, npc.pos + size, RenderLayer::Debug));
        }

    }
}
