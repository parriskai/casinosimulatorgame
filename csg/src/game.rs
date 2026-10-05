//! `csg::game`
//! Core game class
use std::sync::Arc;
use glfw::MouseButton;
use nalgebra::Vector2;

use crate::{graphics::window::Window, vfs::packed::create_packed_vfs, prelude::*, vfs::modularfs::ModuleFS, simulation::{tiles::{FloorType, TileGrid}, world::World}};

/// Core game class, holds all the good stuff
pub struct Game{
    vfs: Arc<dyn CasinoFS>,
    window: Window,
    world: World,
    drag: Option<(Vector2<i32>, Vector2<i32>)>,
    was_mouse_down: bool, // gonna have to keep track if the mouse has been held down to show
                          // preview or draw actual tiles. 
} impl Game{ 
    /// Create the game instance
    pub fn create() -> GResult<Game>{
        let mut vfs = ModuleFS::create();

        vfs.mount("assets".into(), Box::new(create_packed_vfs())).unwrap();

        let avfs = Arc::new(vfs);
        Ok(Game {
            window: Window::create(avfs.clone())?,
            vfs: avfs,
            world: World::new(),
            drag: None,
            was_mouse_down: false,
        })
    }

    pub fn log_info(&self){
        self.window.renderer.gc.log_info();
    }

    /// Lets do this thing
    pub fn run(&mut self) -> GResult<()>{
        while !self.window.should_close(){
            self.window.frame(&self.world, self.drag);

            let input = &self.window.renderer.input_manager;
            let mouse_down = input.mouse_down(MouseButton::Button1);
            let hovered = TileGrid::world_to_tile(input.cursor());

            // mouse just pressed: start a rectange at the hovered tile
            if mouse_down && !self.was_mouse_down{
                self.drag = Some((hovered, hovered));
            }

            // draggin: move the end corer to the hovered tile
            if let Some((start, _)) = self.drag{
                self.drag = Some((start, hovered));
            }

            //release: fill rectage w real tiles
            if !mouse_down && self.was_mouse_down{
                match self.drag{
                    Some((start, end)) => {
                        let left = start.x.min(end.x);
                        let right = start.x.max(end.x);
                        let top = start.y.min(end.y);
                        let bottom = start.y.max(end.y);

                        for y in top..=bottom{
                            for x in left..=right{
                                self.world.grid.set_floor(x,y,FloorType::Carpet);
                            }
                        }
                    }

                    None => {}
                }
                self.drag = None; // drag is OVer
            }
            self.was_mouse_down = mouse_down;
            self.world.update(input.get_delta_time() as f32);
        }
        Ok(())
    }
}
