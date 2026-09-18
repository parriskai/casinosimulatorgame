use csg::{prelude::*, game::Game, logging::init_tracing};

fn main() -> GResult<()>{
    init_tracing();
    
    let mut game = Game::create()?;
    game.log_info();
    
    game.run()
}
