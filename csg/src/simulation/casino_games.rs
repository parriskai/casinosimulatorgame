use rand::RngExt;
use nalgebra::Vector2;
use super::npc::Money;

pub enum Outcome{
    Win(Money),
    Lose,
}

pub struct SlotMachine{
    pub odds: u32
}
pub struct RouletteTable{
    pub odds: u32
}
impl CasinoGame{
    pub fn play(&mut self, bet: Money) -> Outcome{
        match &mut self.kind{
            GameType::Slots(machine) => machine.play_slots(bet),
            GameType::Roulette(table)=> table.play_roulette(bet),
        }
    }
    pub fn slots(pos: Vector2<f32>, odds: u32) -> Self{
        Self{
            pos: pos,
            kind: GameType::Slots(SlotMachine{odds}),
        }
    }
    pub fn roulette(pos: Vector2<f32>, odds: u32) -> Self{
        Self{
            pos: pos, 
            kind: GameType::Roulette(RouletteTable{odds}),
        }
    }
}
impl SlotMachine{
    pub fn play_slots(&mut self, bet: Money) -> Outcome{
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
    pub fn play_roulette(&mut self, bet: Money) -> Outcome{
        let mut rng = rand::rng();
        let random_number: u32 = rng.random_range(0..36);

        if random_number < self.odds{
            Outcome::Win(bet * 2)
        }else{
            Outcome::Lose
        }
    }
}

pub struct CasinoGame{
    pub pos: Vector2<f32>,
    pub kind: GameType,
}

pub enum GameType{
    Slots(SlotMachine),
    Roulette(RouletteTable),
}
