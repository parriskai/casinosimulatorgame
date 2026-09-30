//! `csg::game`
//! Core game class

use crate::{graphics::window::Window, prelude::*, vfs::modularfs::ModuleFS};

/// Core game class, holds all the good stuff
pub struct Game{
    vfs: ModuleFS,
    window: Window
} impl Game{
    /// Create the game instance
    pub fn create() -> GResult<Game>{
        let mut vfs = ModuleFS::create();


        Ok(Game {
            vfs,
            window: Window::create()?
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