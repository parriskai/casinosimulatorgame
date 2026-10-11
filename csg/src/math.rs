use std::fmt::Debug;

pub struct RectSize<T>{
    pub width: T,
    pub height: T
} impl<T: Debug> Debug for RectSize<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}x{:?}", self.width, self.height))
    }
} impl<T: Copy> Copy for RectSize<T>{
} impl<T: Clone> Clone for RectSize<T>{
    fn clone(&self) -> Self {
        RectSize {
            width: self.width.clone(),
            height: self.height.clone()
        }
    }
}

pub fn rect_size<T>(width: T, height: T) -> RectSize<T>{
    RectSize { width, height }
}

pub type WindowSize = RectSize<u32>;
pub type TextureSize = RectSize<u32>;