use nalgebra::Vector2;

pub const TILE_SIZE: f32 = 32.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FloorType{
    #[default]
    None,
    Carpet,
}
impl FloorType{
    pub fn cost(&self) -> i64 {match self {
        FloorType::None => 0,
        FloorType::Carpet => 50,
    }}
}

#[derive(Clone, Copy, Default)]
pub struct Tile{
    pub floor: FloorType,
}

pub struct TileGrid{
    pub width: usize,
    pub height: usize,
    tiles: Vec<Tile>,
    /// Set whenever a tile changes, consumers (renderer cache, pathfinding) can clear it
    pub dirty: bool,
}impl TileGrid{
    pub fn new(width: usize, height: usize) -> TileGrid{
        TileGrid{
            width,
            height,
            tiles: vec![Tile::default(); width*height],
            dirty: false,
        }
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool{
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&Tile>{
        self.in_bounds(x, y).then(|| &self.tiles[y as usize * self.width + x as usize])
    }

    pub fn set_floor(&mut self, x: i32, y: i32, floor: FloorType){
        if self.in_bounds(x, y){
            self.tiles[y as usize * self.width + x as usize].floor = floor;
            self.dirty = true;
        }
    }

    /// Clamp a tile rectangle (inclusive corners) to the grid, None if it is fully outside
    pub fn clamp_rect(&self, min: Vector2<i32>, max: Vector2<i32>) -> Option<(Vector2<i32>, Vector2<i32>)>{
        let gmax = Vector2::new(self.width as i32 - 1, self.height as i32 - 1);
        let min = min.sup(&Vector2::zeros());
        let max = max.inf(&gmax);
        (min.x <= max.x && min.y <= max.y).then_some((min, max))
    }

    pub fn world_to_tile(p: Vector2<f32>) -> Vector2<i32>{
        Vector2::new((p.x / TILE_SIZE).floor() as i32, (p.y / TILE_SIZE).floor() as i32)
    }

    pub fn tile_to_world(t: Vector2<i32>) -> Vector2<f32>{
        Vector2::new(t.x as f32 * TILE_SIZE, t.y as f32 * TILE_SIZE)
    }
}
