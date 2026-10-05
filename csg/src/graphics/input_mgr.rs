use std::{collections::HashMap, time::Instant};

use glfw::{Action, Key, Modifiers, MouseButton, PWindow, WindowEvent};
use nalgebra::Vector2;

pub struct InputManager{
    keys: HashMap<Key, Option<Instant>>,
    mouse: HashMap<MouseButton, bool>,
    modifiers: Modifiers,
    cursor: Vector2<f32>,
    frame_time: Instant,
    total_time: f64,
    delta_time: f64,
    scale: (f32, f32),
} impl InputManager {
    pub fn create() -> InputManager{
        InputManager {
            cursor: Vector2::zeros(),
            keys: HashMap::new(),
            mouse: HashMap::new(),
            modifiers: Modifiers::empty(),
            frame_time: Instant::now(),
            total_time: 0.,
            delta_time: 0.,
            scale: (1., 1.)
        }
    }

    pub fn handle_event(&mut self, e: &WindowEvent) -> bool{
        match e {
            WindowEvent::CursorPos(x, y) => {
                self.cursor.x = *x as f32 * self.scale.0;
                self.cursor.y = *y as f32 * self.scale.1;

                true
            }

            WindowEvent::MouseButton(button, action, _) => {
                if *action == Action::Press{
                    self.mouse.insert(*button, true);
                }else{
                    self.mouse.insert(*button, false);
                }
                true
            }
            WindowEvent::Key(k, _sc, a, _) => {
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

    fn poll_mods(pw: &PWindow) -> Modifiers {
        let mut modfrs = Modifiers::empty();
        
        if pw.get_key(Key::LeftShift) == Action::Press || pw.get_key(Key::RightShift) == Action::Press{
            modfrs |= Modifiers::Shift;
        }
        
        if pw.get_key(Key::LeftControl) == Action::Press || pw.get_key(Key::RightControl) == Action::Press{
            modfrs |= Modifiers::Control;
        }

        if pw.get_key(Key::LeftAlt) == Action::Press || pw.get_key(Key::RightAlt) == Action::Press{
            modfrs |= Modifiers::Alt;
        }

        if pw.get_key(Key::LeftSuper) == Action::Press || pw.get_key(Key::RightSuper) == Action::Press{
            modfrs |= Modifiers::Super;
        }

        if pw.get_key(Key::CapsLock) == Action::Press{
            modfrs |= Modifiers::CapsLock;
        }
        
        if pw.get_key(Key::NumLock) == Action::Press{
            modfrs |= Modifiers::NumLock;
        }

        modfrs
    }

    pub fn tick(&mut self, pw: &PWindow){
        let cur = Instant::now();
        self.delta_time = (cur - self.frame_time).as_secs_f64();
        self.frame_time = cur;
        self.modifiers = Self::poll_mods(pw);
        self.scale = pw.get_content_scale();
        println!("{}x{}", self.scale.0, self.scale.1);

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
        self.keys.get(&k).unwrap_or(&None).is_some()
    }

    pub fn mouse_down(&self, button: MouseButton) -> bool{
        *self.mouse.get(&button).unwrap_or(&false)
    }

    pub fn cursor(&self) -> Vector2<f32>{
        self.cursor
    }

    pub fn get_time(&self) -> Instant{
        self.frame_time
    }

    pub fn get_delta_time(&self) -> f64{
        self.delta_time
    }
}
