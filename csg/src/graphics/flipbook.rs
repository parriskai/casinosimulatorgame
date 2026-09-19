use std::sync::Arc;

use nalgebra::Vector2;
use serde::Deserialize;

use crate::graphics::{RenderLayer, UvBox, asset_mgr::TextureKey, atlasrender::AtlasRenderer};

#[derive(Debug, Deserialize)]
pub struct FlipbookJSON{
    pub size: [f32; 2],
    pub fps: f64,
    pub frames: Vec<UvBox>
} impl FlipbookJSON{
    pub fn from_include(s: &str) -> FlipbookJSON{
        serde_json::from_str(s).unwrap()
    }
}

pub struct Flipbook{
    atlas: TextureKey,
    json: FlipbookJSON,
    index: usize,
    interval: f64,
    lastft: Option<f64>
} impl Flipbook{
    pub fn create(atlas: TextureKey, json: FlipbookJSON) -> Flipbook{
        let interval = 1. / json.fps;

        Flipbook{
            atlas,
            json,
            index: 0,
            interval,
            lastft: None
        }
    }

    pub fn pause(&mut self){
        self.lastft = None
    }

    pub fn resume(&mut self, time: f64){
        self.lastft = Some(time)
    }

    pub fn set_speed(&mut self, speed: f64){
        if speed == 0.{self.pause();}
        else{self.interval = 1. / (self.json.fps * speed);}
    }
    
    pub fn update(&mut self, time: f64){
        match &mut self.lastft{
            None => {/* Do nothing */},
            Some(last) => {
                let dt = time - *last;
                if dt >= self.interval{
                    self.index = (self.index + (dt / self.interval).floor() as usize) % self.json.frames.len();
                    *last = time;
                }
            }
        }
    }

    pub fn draw(&mut self, pos: Vector2<f32>, scale: f32, layer: RenderLayer, ar: &mut AtlasRenderer){
        ar.draw_atlas(self.atlas, self.json.frames[self.index], (pos, pos + Vector2::new(self.json.size[0], self.json.size[1]) * scale, layer));
    }
}


pub enum FlipbookOrMissing {
    Flipbook(Arc<Flipbook>),
    Missing(Arc<Flipbook>)
} impl FlipbookOrMissing{
    pub fn inner(self) -> Arc<Flipbook>{
        match self{
            FlipbookOrMissing::Flipbook(f) => f,
            FlipbookOrMissing::Missing(m) => m
        }
    }
}