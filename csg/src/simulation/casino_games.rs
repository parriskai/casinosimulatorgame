use rand::RngExt;
use nalgebra::Vector2;
use super::world::Money;

const CHEAT_BONUS: f64 = 0.15;
const CATCH_CHANCE: f64 = 0.0;

pub struct CasinoGame{
    pub pos: Vector2<f32>,
    pub kind: GameType,
    pub occupied: bool,
}

pub enum GameType{
    Slots(SlotMachine),
    Roulette(RouletteTable),
}impl GameType{
    fn win_chance(&self) -> f64{
        match self {
            GameType::Slots(g) => g.odds as f64 / 100.,
            GameType::Roulette(g) => g.odds as f64 / 37.0,
        }
    }
}

pub enum Outcome{
    Win(Money),
    Lose,
    Caught,
}

pub struct SlotMachine{
    pub odds: u32
}
pub struct RouletteTable{
    pub odds: u32
}
impl CasinoGame{
    pub fn play(&mut self, bet: Money, cheating: bool) -> Outcome{
        let mut chance = self.kind.win_chance();
        if cheating {
            chance += CHEAT_BONUS;
        }
        if rand::random_bool(CATCH_CHANCE){
            return Outcome::Caught;
        }

        match &mut self.kind{
            GameType::Slots(machine) => machine.play_slots(bet, cheating),
            GameType::Roulette(table)=> table.play_roulette(bet, cheating),
        }
    }
    pub fn slots(pos: Vector2<f32>, odds: u32) -> Self{
        Self{
            pos: pos,
            kind: GameType::Slots(SlotMachine{odds}),
            occupied: false, 
        }
    }
    pub fn roulette(pos: Vector2<f32>, odds: u32) -> Self{
        Self{
            pos: pos, 
            kind: GameType::Roulette(RouletteTable{odds}),
            occupied: false,
        }
    }
}
impl SlotMachine{
    pub fn play_slots(&mut self, bet: Money, cheating: bool) -> Outcome{
        let mut rng = rand::rng();
        let random_number: u32 = rng.random_range(0..100);

        if random_number < self.odds{
            Outcome::Win(bet * 2)
        }else{
            Outcome::Lose
        }
    }
}
impl RouletteTable{
    pub fn play_roulette(&mut self, bet: Money, cheating: bool) -> Outcome{
        let mut rng = rand::rng();
        let random_number: u32 = rng.random_range(0..36);

        let mut odds = self.odds;
        if cheating{
            odds += 10;
        }

        if random_number < self.odds{
            Outcome::Win(bet * 2)
        }else{
            Outcome::Lose
        }
    }
}

