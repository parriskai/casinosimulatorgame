use nalgebra::Vector2;
use super::npc::{Npc, NpcState};
use super::tiles::TileGrid;

pub struct SlotMachines{
    pub pos: Vector2<f32>
}

pub struct World{
    pub npcs: Vec<Npc>,
    pub machines: Vec<SlotMachines>,
    pub house_money: i64,
    pub entrance: Vector2<f32>,
    pub grid: TileGrid,
}

const HOUSE_START_MONEY: i64 = 3000;
const WALLET_MIN: i64 = 500; 
const WALLET_MAX: i64 = 500_000;
const SPIN_TIME: f32 = 10.0; //time it takes to play slots
const WALK_SPEED: f32 = 10.0; 
const WIN_CHANCE: f32 = 5.0;
const BET: i64 = 500;

impl World{
    pub fn new() -> World{
        World{
            npcs: Vec::new(),
            machines: vec![SlotMachines {pos: Vector2::new(300.0, 200.0)}],
            house_money: HOUSE_START_MONEY,
            entrance: Vector2::new(0.0, 0.0),
            grid: TileGrid::new(64, 64)
        }
    } 

    pub fn spawn_npc(&mut self){

        let new_npc = Npc{
            pos: self.entrance,
            wallet: rand::random_range(WALLET_MIN..WALLET_MAX),
            state: NpcState::WalkingTo{machine: 0},
        };

        self.npcs.push(new_npc);
    }
    pub fn update(&mut self, dt: f32){
        for npc in &mut self.npcs{
            match npc.state{
                NpcState::WalkingTo{machine} => {
                    let target = self.machines[machine].pos;

                    if move_toward(&mut npc.pos, target, dt * WALK_SPEED){
                        npc.state = NpcState::Playing{machine, timer: 0.0};
                    }
                }
                NpcState::Playing{machine, timer} => {
                    let timer = timer + dt;
                    if timer < SPIN_TIME {
                        npc.state = NpcState::Playing{machine, timer};
                        continue;
                    }
                    if npc.wallet < BET{
                        npc.state = NpcState::Leaving;
                        continue;
                    }
                    npc.wallet -= BET;
                    self.house_money += BET as i64;
                    if (rand::random_range(0..100) as f32) < WIN_CHANCE{
                        npc.wallet += BET * 2;
                        self.house_money -= (BET * 2) as i64;
                   }
                    npc.state = NpcState::Playing{machine, timer: 0.0}
                }
                NpcState::Leaving => {
                    let target = self.entrance;
                    if move_toward(&mut npc.pos, target, dt * WALK_SPEED){
                        npc.state = NpcState::Gone;
                    }
                }
                NpcState::Gone => {}
            }
        }
    }
}
fn move_toward(npc_pos: &mut Vector2<f32>, target: Vector2<f32>, step: f32) -> bool{
    let diff = target - *npc_pos;
    let dist = diff.norm();
    if dist < step{
        *npc_pos = target;
        true
    }else{
        *npc_pos += diff / dist * step;
        false
    }
}
