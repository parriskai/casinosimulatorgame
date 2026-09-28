use std::{collections::HashMap, time::Instant};

use glfw::{Action, Key, WindowEvent};
use nalgebra::Vector2;

pub struct InputManager{
    keys: HashMap<Key, Option<Instant>>,
    cursor: Vector2<f32>
} impl InputManager {
    pub fn create() -> InputManager{
        InputManager {
            cursor: Vector2::zeros(),
            keys: HashMap::new()
        }
    }

    pub fn handle_event(&mut self, e: &WindowEvent) -> bool{
        match e {
            WindowEvent::CursorPos(x, y) => {
                self.cursor.x = *x as f32;
                self.cursor.y = *y as f32;

                true
            }

            WindowEvent::Key(k, _sc, a, m) => {
                let k = self.keys.entry(*k).or_insert(None);
                
                match a{
                    Action::Press | Action::Repeat => {
                        *k = Some(Instant::now());
                    }
                    Action::Release => {
                        *k = None;
                    }
                }
                true
            },

            _ => {
                false
            }
        }
    }

    pub fn tick(&mut self){
        let cur = Instant::now();
        for (k,v) in self.keys.iter_mut(){
            match v{
                Some(t) => {
                    if  (cur - *t).as_secs() > 1{
                        *v = None;
                        tracing::info!("Key ({k:?}) released due to timeout");
                    }
                }
                None => {}
            }
        }
    }

    pub fn key_down(&self, k: &Key) -> bool{
        self.keys.get(k).unwrap_or(&None).is_some()
    }
}