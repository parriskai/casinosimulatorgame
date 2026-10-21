use nalgebra::Vector2;
use wgpu::TextureFormat;
use crate::graphics::input_mgr::InputManager;
use crate::graphics::{assets::manager::AssetManager, window::Window};
use crate::graphics::graphicscontrol::GraphicsControl;
use crate::simulation::chunkmanager::ChunkManager;
use crate::prelude::*;
use super::npc::{Npc, NpcState, Money};

pub struct SlotMachines{
    pub pos: Vector2<f32>
}

pub struct View{
    pub scale: f32,
    pub center: Vector2<f32>
}

pub enum GamePhase{
    Editing,
}


pub struct World{
    pub npcs: Vec<Npc>,
    pub max_npcs: usize,
    pub spawn_timer: f32,
    pub machines: Vec<SlotMachines>,
    pub house_money: Money,
    pub entrance: Vector2<f32>,
    pub grid: ChunkManager,

    pub view: View,
    //pub mouse_tile: (i32, i32)
}

const HOUSE_START_MONEY: Money = 3000;
const NPC_SPAWN_INTERVAL: f32 = 3.0;
const WALLET_MIN: Money = 500; 
const WALLET_MAX: Money = 500_000;
const SPIN_TIME: f32 = 10.0; //time it takes to play slots
const WALK_SPEED: f32 = 10.0; 
const WIN_CHANCE: f32 = 5.0;
const BET: i64 = 500;

impl World{
    pub fn create(win: &mut Window) -> GResult<World>{
        let grid = ChunkManager::create(win.renderer.gc.clone(), win.surface_format.clone(), &mut win.renderer.asset_manager)?;
        Ok(
            World{
                npcs: Vec::new(),
                max_npcs: 3,
                spawn_timer: 0.0,
                machines: vec![SlotMachines {pos: Vector2::new(300.0, 200.0)}],
                house_money: HOUSE_START_MONEY,
                entrance: Vector2::new(0.0, 0.0),
                view: View { scale: 0., center: Vector2::zeros()},
                grid
            }
        )
    } 

    pub fn spawn_npc(&mut self){

        let new_npc = Npc{
            pos: self.entrance,
            wallet: rand::random_range(WALLET_MIN..WALLET_MAX),
            state: NpcState::WalkingTo{machine: 0},
            machine_prefs: vec![1; self.machines.len()]
        };

        self.npcs.push(new_npc);
    }

    pub fn update(&mut self, im: &InputManager){
        if im.should_reload_fb{
            self.grid.reload_transform();
        }
        let dt = im.get_delta_time() as f32;

        self.spawn_timer += dt;
        // if (self.spawn_timer > NPC_SPAWN_INTERVAL) && self.npcs.len() < self.npcs_population{ 
        //     self.spawn_timer = 0.0;
        //     self.spawn_npc();
        // }
        
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
        // get rid of the npcs that left the casino. 
        self.npcs.retain(|npc| {
            match npc.state{
                NpcState::Gone => false,
                _ => true,
            }
        });
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
