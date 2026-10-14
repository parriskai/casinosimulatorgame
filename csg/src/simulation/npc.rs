use nalgebra::Vector2;

pub type Money = i64;

pub enum NpcState{
    WalkingTo{machine: usize},
    Playing{machine: usize, timer: f32},
    Leaving,
    Gone,
}

pub struct Npc{
    pub pos: Vector2<f32>,
    pub wallet: Money,
    pub state: NpcState,
    pub machine_prefs: Vec<usize>,
}

