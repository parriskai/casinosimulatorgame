use std::ops::{Add, Range};

use glfw::Key::S;
pub struct Rect<T>{
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T
} impl<T, O> Rect<T> where T: Add<T, Output = O> + Clone + Into<O>{
    pub fn x_iter(&self) -> Range<O>{
        self.x.clone().into()..self.x.clone() + self.w.clone()
    }
    pub fn y_iter(&self) -> Range<O>{
        self.y.clone().into()..self.y.clone() + self.w.clone()
    }
}

pub struct Square3x3<T>{
    pub nw: T, pub n:  T, pub ne: T,
    pub w:  T, pub c:  T, pub e:  T,
    pub sw: T, pub s:  T, pub se: T
} impl<T> Square3x3<T>{
    pub fn construct_from_fn_offset<F: Fn(i32, i32) -> T>(f: F) -> Square3x3<T>{
        Square3x3 {
            nw: f(-1, -1), n: f(0, -1), ne: f(1, -1),
            w:  f(-1,  0), c: f(0,  0), e:  f(1,  0),
            sw: f(-1,  1), s: f(0,  1), se: f(1,  1) }
    }
    pub fn move_north(&mut self, nw: T, n: T, ne: T){
        self.sw = std::mem::replace(&mut self.w, std::mem::replace(&mut self.nw, nw));
        self.s  = std::mem::replace(&mut self.c, std::mem::replace(&mut self.n,  n ));
        self.se = std::mem::replace(&mut self.e, std::mem::replace(&mut self.ne, ne));
    }

    pub fn move_south(&mut self, sw: T, s: T, se: T){
        self.nw = std::mem::replace(&mut self.w, std::mem::replace(&mut self.sw, sw));
        self.n  = std::mem::replace(&mut self.c, std::mem::replace(&mut self.s,  s ));
        self.ne = std::mem::replace(&mut self.e, std::mem::replace(&mut self.se, se));
    }
    
    pub fn move_east(&mut self, ne: T, e: T, se: T){
        self.nw = std::mem::replace(&mut self.n, std::mem::replace(&mut self.ne, ne));
        self.w =  std::mem::replace(&mut self.c, std::mem::replace(&mut self.e,  e ));
        self.sw = std::mem::replace(&mut self.s, std::mem::replace(&mut self.se, se));
    }

    pub fn move_west(&mut self, nw: T, w: T, sw: T){
        self.ne = std::mem::replace(&mut self.n, std::mem::replace(&mut self.nw, nw));
        self.e =  std::mem::replace(&mut self.c, std::mem::replace(&mut self.w,  w ));
        self.se = std::mem::replace(&mut self.s, std::mem::replace(&mut self.sw, sw));
    }

    pub fn get_adjacent_center(&mut self) -> (&mut T, Adjacent<&mut T>){
        (
            &mut self.c,
            Adjacent{
                nw: &mut self.nw, n: &mut self.n, ne: &mut self.ne,
                w:  &mut self.w,               e:  &mut self.e,
                sw: &mut self.sw, s: &mut self.s, se: &mut self.se
            }
        )
    }
}

pub struct Adjacent<T>{
    pub nw: T, pub n:  T, pub ne: T,
    pub w:  T,            pub e:  T,
    pub sw: T, pub s:  T, pub se: T
} impl<T> Adjacent<T>{
    pub fn construct_from_fn_offset<F: FnMut(i32, i32) -> T>(mut f: F) -> Adjacent<T>{
        Adjacent {
            nw: f(-1, -1), n: f(0, -1), ne: f(1, -1),
            w:  f(-1,  0),               e: f(1,  0),
            sw: f(-1,  1), s: f(0,  1), se: f(1,  1)
        }
    }
    pub fn move_north(&mut self, c: T, nw: T, n: T, ne: T) -> T{
        self.sw = std::mem::replace(&mut self.w, std::mem::replace(&mut self.nw, nw));
        self.s = c;
        self.se = std::mem::replace(&mut self.e, std::mem::replace(&mut self.ne, ne));
        std::mem::replace(&mut self.n,  n)
    }

    pub fn move_south(&mut self, c: T, sw: T, s: T, se: T) -> T{
        self.nw = std::mem::replace(&mut self.w, std::mem::replace(&mut self.sw, sw));
        self.n = c;
        self.ne = std::mem::replace(&mut self.e, std::mem::replace(&mut self.se, se));
        std::mem::replace(&mut self.s,  s)
    }
    
    pub fn move_east(&mut self, c: T, ne: T, e: T, se: T) -> T{
        self.nw = std::mem::replace(&mut self.n, std::mem::replace(&mut self.ne, ne));
        self.w = c;
        self.sw = std::mem::replace(&mut self.s, std::mem::replace(&mut self.se, se));
        std::mem::replace(&mut self.e,  e)
    }

    pub fn move_west(&mut self, c: T, nw: T, w: T, sw: T) -> T{
        self.ne = std::mem::replace(&mut self.n, std::mem::replace(&mut self.nw, nw));
        self.e = c;
        self.se = std::mem::replace(&mut self.s, std::mem::replace(&mut self.sw, sw));
        std::mem::replace(&mut self.w,  w)
    }

    pub fn fill_center(self, c: T) -> Square3x3<T>{
        Square3x3 {
            nw: self.nw, n: self.n, ne: self.ne,
            w:  self.w,  c: c,      e:  self.e,
            sw: self.sw, s: self.s, se: self.se 
        }
    }
}

pub trait SetBit {
    fn enable_bit(&mut self, bit: usize);
    fn clear_bit(&mut self, bit: usize);
    fn set_bit(&mut self, bit: usize, value: bool){
        if value{
            self.enable_bit(bit);
        } else {
            self.clear_bit(bit);
        }
    }
}

impl SetBit for usize{
    fn enable_bit(&mut self, bit: usize) {
        *self |= 1 << bit;
    }
    fn clear_bit(&mut self, bit: usize) {
        *self = *self & !(1 << bit);
    }
}