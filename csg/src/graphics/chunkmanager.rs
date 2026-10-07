use crate::graphics::assets::{
    gputexture::GpuTexture,
    manager::AssetKey
};
use ahash::AHashMap;

pub enum Tile{
    None,
    Floor,
}
pub const CHUNK_SIZE: usize = 16;

pub struct Chunk{
   grid: [[Tile; CHUNK_SIZE]; CHUNK_SIZE]
} impl Chunk{
    pub fn get(&self, x: usize, y: usize) -> &Tile{
        &self.grid[y][x]
    }
}

pub struct  ChunkManager {
    _atlas: AssetKey<GpuTexture>,
    _table: AHashMap<(i32, i32), Chunk>
}