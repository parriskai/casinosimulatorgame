use csg::{prelude::*, game::Game, logging::{init_tracing, log_result}};

fn main() -> GResult<()>{
    init_tracing();
    
    log_result(||{
        let mut game = Game::create()?;
        game.log_info();
    
        game.run()},

        "Game execution"
    )
}
