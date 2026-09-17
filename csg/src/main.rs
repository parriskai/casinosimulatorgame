use csg::{prelude::*, game::Game};

fn main() -> GResult<()>{
    let mut game = Game::create()?;
    game.run()
}
