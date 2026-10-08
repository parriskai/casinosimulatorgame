use std::{time::Instant};
use glfw::{Action, Key, Modifiers, MouseButton, PWindow, WindowEvent};
use nalgebra::Vector2;
use ahash::AHashMap;

pub struct InputManager{
    keys: AHashMap<Key, Option<(Instant, u32)>>,
    mouse: AHashMap<MouseButton, bool>,
    modifiers: Modifiers,
    cursor: Vector2<f32>,
    frame_time: Instant,
    delta_time: f64,
    scale: (f32, f32),
    pub should_reload_fb: bool
} impl InputManager {
    pub fn create() -> InputManager{
        InputManager {
            cursor: Vector2::zeros(),
            keys: AHashMap::new(),
            mouse: AHashMap::new(),
            modifiers: Modifiers::empty(),
            frame_time: Instant::now(),
            delta_time: 0.,
            scale: (1., 1.),
            should_reload_fb: true
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

            WindowEvent::FramebufferSize(_, _) | WindowEvent::Size(_, _)  | WindowEvent::ContentScale(_, _ ) => {
                self.should_reload_fb = true;
                true
            }

            WindowEvent::Key(k, _sc, a, _) => {
                let k = self.keys.entry(*k).or_insert(None);
                
                match a{
                    Action::Press | Action::Repeat => {
                        *k = Some((Instant::now(), 0));
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
        for (k,v) in self.keys.iter_mut(){
            match v{
                Some((t,ct)) => {
                    if (cur - *t).as_secs() > 1{
                        *v = None;
                        tracing::info!("Key ({k:?}) released due to timeout");
                    } else {
                        *ct += 1;
                    }
                }
                None => {}
            }
        }
    }

    pub fn key_down(&self, k: &Key) -> bool{
        self.keys.get(&k).unwrap_or(&None).is_some()
    }
    
    pub fn key_down_for(&self, k: &Key) -> Option<u32>{
        self.keys.get(&k).unwrap_or(&None).map(|x| x.1.clone())
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
