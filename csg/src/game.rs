//! `csg::game`
//! Core game class
#[cfg(any(debug_assertions, feature = "debug_tools"))]
use crate::debug_features::DebugFeatures;
use crate::{commands::{GameCommand, GameCommandReciver, create_command_pair}, input_mgr::InputManager, prelude::*, renderer::Renderer, simulation::world::World, vfs::{modularfs::ModuleFS, packed::create_packed_vfs}, window::Window};
use std::{cell::Cell, rc::Rc};
/// Core game class, holds all the good stuff
pub struct Game{
    vfs: Rc<Cell<dyn CasinoFS>>,

    window: Window,
    
    renderer: Renderer,
    
    world: World,
    
    input_mgr: InputManager,
    
    #[cfg(any(debug_assertions, feature = "debug_tools"))]
    debug_features: Rc<Cell<DebugFeatures>>,

    game_command: GameCommandReciver
} impl Game{ 
    /// Create the game instance
    pub fn create() -> GResult<Game>{
        let (command_sender, command_recivers) = create_command_pair();

        let vfs = Rc::new(Cell::new(ModuleFS::create()));

        {
            let vfs = vfs.get_mut();
            vfs.mount("assets".into(), Box::new(create_packed_vfs())).unwrap();
        }
        
        let window = Window::create(command_sender.clone())?;
        
        let renderer = Renderer::create(&window, command_recivers.renderer_recv)?;

        let mut world = World::create(&mut window)?;
        world.grid.create_chunk((0,0));

        let input_mgr = InputManager::new();

        #[cfg(any(debug_assertions, feature = "debug_tools"))]
        let debug_features = Rc::new(Cell::new(DebugFeatures::default()));

        Ok(Game {
            vfs,
            window,
            renderer,
            world,
            input_mgr,
            #[cfg(any(debug_assertions, feature = "debug_tools"))]
            debug_features,
            game_command: command_recivers.game_recv
        })
    }

    /// Lets do this thing
    pub fn run(&mut self) -> GResult<()>{
        loop{
            for ge in self.game_command.iter(){
                match ge{
                    GameCommand::Quit => {
                        return Ok(())
                    }
                }
            }

            self.window.handle_events(&mut self.input_mgr);

            #[cfg(any(debug_assertions, feature = "debug_tools"))]
            {
                let debug = self.debug_features.get_mut();
                debug.debug_hotkeys(&self.input_mgr);
            }


            self.world.update(&self.input_mgr);
            self.renderer.update();

            self.renderer.render(&mut self.world);

            self.input_mgr.should_reload_fb = false;            
        }
    }
}
