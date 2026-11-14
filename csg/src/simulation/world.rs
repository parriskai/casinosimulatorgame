use nalgebra::Vector2;
use crate::graphics::input_mgr::InputManager;
use crate::graphics::{window::Window};
use crate::simulation::chunkmanager::ChunkManager;
use crate::prelude::*;
use super::npc::{Npc, NpcState, DirectionFacing};
use super::casino_games::{CasinoGame, GameType, SlotMachine, RouletteTable,  Outcome};
 
pub type Money = i64;

pub struct View{
    pub scale: f32,
    pub center: Vector2<f32>
}

pub struct World{
    pub npcs: Vec<Npc>,
    pub max_npcs: usize,
    pub spawn_timer: f32,
    pub games: Vec<CasinoGame>,
    pub house_money: Money,
    pub reputation: f32,
    pub entrance: Vector2<f32>,
    pub grid: ChunkManager,
    pub view: View,
}

const HOUSE_START_MONEY: Money = 3000;
const NPC_SPAWN_INTERVAL: f32 = 3.0;
const WALLET_MIN: Money = 500; 
const WALLET_MAX: Money = 5_000;
const SPIN_TIME: f32 = 4.0; //time it takes to play slots
const WALK_SPEED: f32 = 90.0; 
const MIN_BET: i64 = 5;
const CHEATER_RATE: f64 = 0.10;

impl World{
    pub fn create(win: &mut Window) -> GResult<World>{
        let grid = ChunkManager::create(win.renderer.gc.clone(), win.surface_format.clone(), &mut win.renderer.asset_manager)?;
        Ok(
            World{
                npcs: Vec::new(),
                max_npcs: 30,
                spawn_timer: 0.0,
                games: vec![
                    CasinoGame::new(Vector2::new(800.0, 200.0), GameType::Roulette(RouletteTable {})),
                    CasinoGame::new(Vector2::new(300.0, 200.0), GameType::Slots(SlotMachine {})),
                    CasinoGame::new(Vector2::new(200.0, 200.0), GameType::Slots(SlotMachine {})),
                ],
                house_money: HOUSE_START_MONEY,
                reputation: 0.0,
                entrance: Vector2::new(0.0, 0.0),
                view: View { scale: 0., center: Vector2::zeros()},
                grid
            }
        )
    } 

    pub fn spawn_npc(&mut self){
        let mut unoccupied_games = Vec::new();
        for (idx, game) in self.games.iter().enumerate(){
            if !game.occupied{
                unoccupied_games.push(idx) 
            }
        }
        if unoccupied_games.len() <= 0{
            return;
        }

        let mut game_idx = None;
        if !unoccupied_games.is_empty(){
            let pick= rand::random_range(0..unoccupied_games.len());
            game_idx = Some(unoccupied_games[pick]);
        }

        self.games[game_idx.unwrap()].occupied = true;
        let new_npc = Npc{
            pos: self.entrance,
            facing: DirectionFacing::East, 
            wallet: rand::random_range(WALLET_MIN..WALLET_MAX),
            state: NpcState::WalkingTo{machine: game_idx.unwrap()},
            game_prefs: vec![1; self.games.len()],
            cheater: rand::random_bool(CHEATER_RATE)
        };
        self.npcs.push(new_npc);
    }

    pub fn update(&mut self, im: &InputManager){
        if im.should_reload_fb{
            self.grid.reload_transform();
        }
        let dt = im.get_delta_time() as f32;
        self.spawn_timer += dt;
        if (self.spawn_timer > NPC_SPAWN_INTERVAL) && self.npcs.len() < self.max_npcs{ 
            self.spawn_timer = 0.0;
            self.spawn_npc();
        }
        
        for npc in &mut self.npcs{
            match npc.state{
                NpcState::WalkingTo{machine} => {
                    let target = self.games[machine].pos;
                    let diff = target - npc.pos;
                    npc.facing = DirectionFacing::from_diff(diff);

                    if move_toward(&mut npc.pos, target, dt * WALK_SPEED){
                        npc.state = NpcState::Playing{machine, timer: 0.0};
                    }
                }
                NpcState::Playing{machine, timer} => {
                    npc.facing = DirectionFacing::North;

                    let timer = timer + dt;
                    if timer < SPIN_TIME {
                        npc.state = NpcState::Playing{machine, timer};
                        continue;
                    }
                    if npc.wallet < MIN_BET{
                        self.games[machine].occupied = false;
                        npc.state = NpcState::Leaving;
                        continue;
                    }
                    let bet = rand::random_range(MIN_BET..=npc.wallet);

                    npc.wallet -= bet;
                    self.house_money += bet as i64;

                    match self.games[machine].play(bet, npc.cheater){
                        Outcome::Win(amount) => {
                            npc.wallet += amount;
                            self.house_money -= amount;
                        }
                        Outcome::Lose => {}
                        Outcome::Caught=>{
                            npc.state = NpcState::Leaving;
                            self.games[machine].occupied = false;
                            continue;
                        }
                    }
                    
                    npc.state = NpcState::Playing{machine, timer: 0.0}
                }
                NpcState::Leaving=> {
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
