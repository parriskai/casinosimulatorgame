pub mod graphicscontrol;
pub mod atlasrender;
pub mod asset_mgr;
pub mod renderer;
pub mod textren;
pub mod window;

use bytemuck::{Pod, Zeroable};
use serde::Deserialize;
use strum::EnumCount;

#[repr(C)]
#[derive(Clone, Copy, Deserialize, Pod, Zeroable)]
pub struct UvBox{
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
} impl UvBox{
    const ZERO: Self = UvBox{u0: 0., v0: 0., u1: 0., v1: 0.};
    const FULL: Self = UvBox{u0: 0., v0: 0., u1: 1., v1: 1.};
} impl core::fmt::Debug for UvBox{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("UvBox[({}, {}) -> ({}, {})]", self.u0, self.v0, self.u1, self.v1))
    }
}

#[repr(u8)]
#[derive(EnumCount, Clone, Copy)]
pub enum RenderLayer{
    CLEAR,
    TOP,
}