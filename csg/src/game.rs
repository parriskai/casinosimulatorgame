//! `csg::game`
//! Core game class
use std::sync::Arc;
use glfw::MouseButton;

use crate::{graphics::window::Window, vfs::packed::create_packed_vfs, prelude::*, vfs::modularfs::ModuleFS, simulation::{tiles::{FloorType, TileGrid}, word::Word}};

/// Core game class, holds all the good stuff
pub struct Game{
    vfs: Arc<dyn CasinoFS>,
    window: Window,
    world: World,
} impl Game{
    /// Create the game instance
    pub fn create() -> GResult<Game>{
        let mut vfs = ModuleFS::create();

        vfs.mount("assets".into(), Box::new(create_packed_vfs())).unwrap();

        let avfs = Arc::new(vfs);
        Ok(Game {
            window: Window::create(avfs.clone())?,
            vfs: avfs,
            world: World::new()
        })
    }

    pub fn log_info(&self){
        self.window.renderer.gc.log_info();
    }

    /// Lets do this thing
    pub fn run(&mut self) -> GResult<()>{
        while !self.window.should_close(){
            self.window.frame(&self.world);
            let input = &self.window.renderer.input_manager;
            if input.mouse_down(MouseButton::Button1){
                let tile = TileGrid::world_to_tile(input.cursor());
                self.world.grid.set_floor(tile.x, tile.y, FloorType::Carpet);
            }
            self.world.update(input.get_delta_time() as f32);
        }

        Ok(())
    }
}
