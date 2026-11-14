use nalgebra::Vector2;
use super::world::Money;

const CHEAT_BONUS: f64 = 0.15;
const CATCH_CHANCE: f64 = 0.0;

pub struct CasinoGame{
    pub pos: Vector2<f32>,
    pub kind: GameType,
    pub win_chance: f64,
    pub occupied: bool,
}

pub enum GameType{
    Slots(SlotMachine),
    Roulette(RouletteTable),
}impl GameType{
    fn win_chance(&self) -> f64{
        match self{
            GameType::Slots(_) => 0.45,
            GameType::Roulette(_) => 18./37.,
        }
    }
    fn payout(&self, bet: i64) -> i64{
        match self{
            GameType::Slots(_) => bet * 2,
            GameType::Roulette(_) => bet * 2,
        }
    }
}

pub enum Outcome{
    Win(Money),
    Lose,
    Caught,
}

pub struct SlotMachine{
}
pub struct RouletteTable{
}

impl CasinoGame{
    pub fn play(&mut self, bet: Money, cheating: bool) -> Outcome{
        let mut chance= self.win_chance;
        if cheating {
            chance += CHEAT_BONUS;
        }
        if rand::random_bool(CATCH_CHANCE){
            return Outcome::Caught;
        }

        if rand::random_bool(chance){
            Outcome::Win(self.kind.payout(bet))
        }else{
            Outcome::Lose
        }

    }
    pub fn new(pos: Vector2<f32>, kind: GameType) -> Self{
        let win_chance = kind.win_chance();
        Self{
            pos: pos,
            kind: kind,
            win_chance: win_chance,
            occupied: false,
        }
    }
}
impl SlotMachine{
}
impl RouletteTable{
}

