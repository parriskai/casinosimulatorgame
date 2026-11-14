use nalgebra::Vector2;
use std::cmp;
use super::world::Money;


pub enum DirectionFacing{
    North,
    South,
    East,
    West,
}
impl DirectionFacing{
    pub fn from_diff(v: Vector2<f32>)->DirectionFacing{
        if v.x <= 0.{
            if v.y <= 0.{
                return DirectionFacing::South;
            }else{
                return DirectionFacing::North;
            }
            return DirectionFacing::West;
        }
        DirectionFacing::East
    }
}

pub enum NpcState{
    WalkingTo{machine: usize},
    Playing{machine: usize, timer: f32},
    Leaving,
    Gone,
} 

pub struct Npc{
    pub pos: Vector2<f32>,
    pub facing: DirectionFacing,
    pub wallet: Money,
    pub state: NpcState,
    pub game_prefs: Vec<usize>,
    pub cheater: bool,
}

