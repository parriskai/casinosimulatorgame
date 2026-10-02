pub mod gputexture;
pub mod manager;
pub mod image;

use serde::Deserialize;
use unsafe_any::UnsafeAnyExt;
use crate::prelude::*;
use std::any::{Any, TypeId};


pub trait ReloadableAsset: Any{
    fn reload(&mut self, data: &[u8]) -> GResult<()>;
}
unsafe impl UnsafeAnyExt for dyn ReloadableAsset {}

impl<T: ReloadableAsset> ReloadableAsset for Box<T> where T: ?Sized{
    fn reload(&mut self, data: &[u8]) -> GResult<()> {
        self.as_mut().reload(data)
    }
}

pub trait JsonLoadable<'a>: Deserialize<'a> {
    fn from_include(s: &'a str) -> GResult<Self>{
        Ok(serde_json::from_str(s).g_err()?)
    }

    fn load(data: &'a [u8]) -> GResult<Self>{
        Self::from_include(str::from_utf8(data).g_err()?)
    }
}
impl<'a, T: Deserialize<'a>> JsonLoadable<'a> for T{}

pub trait DefaultAsset{
    fn default() -> Self;
}
pub struct AssetDefault {
    key: TypeId,
    value: fn() -> Box<dyn Any>,
} impl AssetDefault {
    pub const fn create<T: DefaultAsset + 'static>() -> Self {
        
        fn construct<T: DefaultAsset + 'static>() -> Box<dyn Any> {
            Box::new(T::default())
        }

        Self {
            key: TypeId::of::<T>(),
            value: construct::<T>,
        }
    }
}
// This is a lie, I dont care
unsafe impl Sync for AssetDefault{}

inventory::collect!(AssetDefault);