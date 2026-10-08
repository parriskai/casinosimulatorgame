//! `csg::game`
//! Core game class
use crate::{
    graphics::window::Window,
    vfs::packed::create_packed_vfs,
    prelude::*,
    vfs::modularfs::ModuleFS,
    simulation::{
        world::World
    }
};

use std::sync::Arc;

/// Core game class, holds all the good stuff
pub struct Game{
    pub vfs: Arc<dyn CasinoFS>,
    window: Window,
    world: World,
} impl Game{ 
    /// Create the game instance
    pub fn create() -> GResult<Game>{
        let mut vfs = ModuleFS::create();

        vfs.mount("assets".into(), Box::new(create_packed_vfs())).unwrap();

        let avfs = Arc::new(vfs);
        let mut window = Window::create(avfs.clone())?;
        let mut world = World::create(&mut window)?;
        world.grid.create_chunk((0,0));

        Ok(Game {
            window: window,
            vfs: avfs,
            world,
        })
    }

    pub fn log_info(&self){
        self.window.renderer.gc.log_info();
    }

    /// Lets do this thing
    pub fn run(&mut self) -> GResult<()>{
        while !self.window.should_close(){
            self.window.update();

            self.world.update(&self.window.renderer.input_manager);

            self.window.render(&mut self.world);

            self.window.renderer.input_manager.should_reload_fb = false;            
        }
        Ok(())
    }
}
