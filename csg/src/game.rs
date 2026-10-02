//! `csg::game`
//! Core game class
use std::sync::Arc;

use crate::{graphics::window::Window, vfs::packed::create_packed_vfs, prelude::*, vfs::modularfs::ModuleFS};

/// Core game class, holds all the good stuff
pub struct Game{
    vfs: Arc<dyn CasinoFS>,
    window: Window
} impl Game{
    /// Create the game instance
    pub fn create() -> GResult<Game>{
        let mut vfs = ModuleFS::create();

        vfs.mount("assets".into(), Box::new(create_packed_vfs())).unwrap();

        let avfs = Arc::new(vfs);
        Ok(Game {
            window: Window::create(avfs.clone())?,
            vfs: avfs
        })
    }

    pub fn log_info(&self){
        self.window.renderer.gc.log_info();
    }

    /// Lets do this thing
    pub fn run(&mut self) -> GResult<()>{
        while !self.window.should_close(){
            self.window.frame();
        }

        Ok(())
    }
}