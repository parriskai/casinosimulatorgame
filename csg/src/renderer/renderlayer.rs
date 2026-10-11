use strum::EnumCount;

#[repr(u8)]
#[derive(EnumCount, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderLayer{
    /// Bottom most layer, used to draw the missing texture behind everything else
    Clear,
    Tile,
    /// Debug layer, rendered above everything else
    Debug,
}
