use crate::{
    graphics::{
        RenderLayer,
        UvBox,
        assets::{
            AssetDefault,
            DefaultAsset,
            JsonLoadable,
            ReloadableAsset,
            gputexture::GpuTexture,
            manager::AssetKey
        },
        renderer::Renderer
    },
    prelude::*
};
use std::time::Instant;
use nalgebra::Vector2;
use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct FlipbookJSON{
    pub size: [f32; 2],
    pub fps: f64,
    pub frames: Vec<UvBox>
} impl ReloadableAsset for FlipbookJSON{
    fn reload(&mut self, data: &[u8]) -> GResult<()> {
        match Self::load(data){
            Ok(data) => {
                drop(std::mem::replace(self, data));
                Ok(())
            }
            Err(e) => {
                drop(std::mem::replace(self, <FlipbookJSON as DefaultAsset>::default()));
                Err(e)
            }
        }
    }
} impl DefaultAsset for FlipbookJSON{
    fn default() -> Self {
        FlipbookJSON{size: [16.,16.], fps: 1., frames: vec![UvBox::FULL]}
    }
}
inventory::submit!(AssetDefault::create::<FlipbookJSON>());

pub struct Flipbook{
    atlas: AssetKey<GpuTexture>,
    json: AssetKey<FlipbookJSON>,
    index: usize,
    speed: f64,
    lastft: Option<Instant>
} impl Flipbook{
    pub fn create(atlas: AssetKey<GpuTexture>, json: AssetKey<FlipbookJSON>) -> Flipbook{
        Flipbook{
            atlas,
            json,
            index: 0,
            speed: 1.,
            lastft: None
        }
    }

    pub fn pause(&mut self){
        self.lastft = None;
    }

    pub fn resume(&mut self, time: Instant){
        self.lastft = Some(time);
    }

    pub fn set_speed(&mut self, speed: f64){
        if speed == 0.{
            self.pause();
        }
        else{
            self.speed = speed;
        }
    }

    pub fn draw(&mut self, pos: Vector2<f32>, scale: f32, layer: RenderLayer, ren: &mut Renderer){
        let json = ren.asset_manager.get_asset_or_default(self.json).unwrap();

        let time = ren.input_manager.get_time();
        match &mut self.lastft{
            None => {/* Do nothing */},
            Some(last) => {
                let dt = (time - *last).as_secs_f64();
                if dt >= 1. / (json.fps * self.speed){
                    self.index = (self.index + (dt / json.fps * self.speed).floor() as usize) % json.frames.len();
                    *last = time;
                }
            }
        }

        ren.atlas_renderer.draw_atlas(self.atlas, json.frames[self.index], (pos, pos + Vector2::new(json.size[0], json.size[1]) * scale, layer));
    }
}