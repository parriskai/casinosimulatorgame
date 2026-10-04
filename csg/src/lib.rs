//! Casino Simulator Game (name is still a WIP, codename `csg`)
//! 
//! # Running the game
//! 
//! - First you should call `csg::logging::init_tracing()`
//! - Then create the game instance `csg::game::Game::create()`
//! - Then run it `Game.run()`

pub mod simulation;
pub mod graphics;
pub mod logging;
pub mod prelude;
pub mod errors;
pub mod utils;
pub mod game;
pub mod vfs;
