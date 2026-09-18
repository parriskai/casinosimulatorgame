use crate::{graphics::window::Window, prelude::*};

/// Core game class, holds all the good stuff
pub struct Game{
    window: Window
} impl Game{
    /// Create the game instance
    pub fn create() -> GResult<Game>{
        Ok(Game {
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