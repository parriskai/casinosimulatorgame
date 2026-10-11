//! Casino Simulator Game (name is still a WIP, codename `csg`)
//! 
//! # Running the game
//! 
//! - First you should call `csg::logging::init_tracing()`
//! - Then create the game instance `csg::game::Game::create()`
//! - Then run it `Game.run()`

#[cfg(any(debug_assertions, feature = "debug_tools"))]
pub mod debug_features;
pub mod simulation;
pub mod input_mgr;
pub mod commands;
pub mod renderer;
pub mod logging;
pub mod prelude;
pub mod window;
pub mod errors;
pub mod utils;
pub mod math;
pub mod game;
pub mod vfs;
