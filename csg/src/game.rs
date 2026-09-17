use crate::{graphics::window::Window, prelude::*};


pub struct Game{
    window: Window
} impl Game{
    pub fn create() -> GResult<Game>{
        Ok(Game {
            window: Window::create()?
        })
    }

    pub fn run(&mut self) -> GResult<()>{
        loop{
            self.window.frame();
        }
    }
}