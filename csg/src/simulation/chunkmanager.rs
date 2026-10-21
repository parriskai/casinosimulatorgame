use std::{cell::{Cell, RefCell}, rc::Rc};

use crate::{
    graphics::{
        RenderLayer, UvBox, assets::{
            gputexture::{GpuTexture, load_texture}, manager::{AssetKey, AssetManager}
        }, graphicscontrol::GraphicsControl
    }, math::{Adjacent, Rect, SetBit}, prelude::*, simulation::{chunkmanager::Tile::Carpet, world::View}, utils::{IntoGpuMatrix, coordinate_transform, vec2_to_vec3}
};
use ahash::AHashMap;
use bytemuck::{Pod, Zeroable};
use nalgebra::{Matrix4, Vector2, Vector3};
use serde::Deserialize;
use strum::EnumCount;
use wgpu::{BindGroup, BindGroupLayout, Buffer, PipelineLayout, RenderPass, RenderPipeline, ShaderModule, TextureFormat, naga::compact::KeepUnused::No, util::DeviceExt};

#[derive(Debug, Deserialize)]
pub struct TileSetJson{
    size: [f32; 2],
    tiles: AHashMap<String, Vec<UvBox>>
} impl TileSetJson{
    fn create_uv_list(&self) -> Vec<[f32; 4]>{
        let mut listing = Vec::new();
        for (tile_name, tile_uv_list) in &self.tiles{
            for (frame_index, tile_uv) in tile_uv_list.into_iter().enumerate(){
                match Tile::get_index_for_name(tile_name, frame_index){
                    Some(idx) => {
                        if idx >= listing.len(){
                            let old_len = listing.len();

                            listing.reserve(idx + 1);
                            // SAFTEY: We just reserved it and we are about to fill it
                            unsafe{listing.set_len(idx + 1)};
                            listing[old_len..idx + 1].fill([0., 0., 1., 1.]);
                        }
                        // SAFTEY: We already ensured that it is in bounds
                        *unsafe{listing.get_unchecked_mut(idx)} = [tile_uv.u0, tile_uv.v0, tile_uv.u1, tile_uv.v1];
                    }
                    None => tracing::warn!("Tile {tile_name} has no index!")
                }
            }
        }
        listing
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Tile{
    #[default]
    None,
    Floor,
    Carpet(usize),
} impl Tile{
    fn index(&self) -> u32{
        match self{
            Tile::None => 0,
            Tile::Floor => 1,
            Tile::Carpet(frame) => 2 + *frame as u32
        }
    }
    
    fn get_index_for_name(s: &str, frame: usize) -> Option<usize>{
        match (s, frame){
            // 0
            ("none", 0) => Some(0),
            // 1
            ("floor", 0) => Some(1),
            // 2..257
            ("carpet", 0..256) => Some(frame + 2),
            _ => None
        }
    }

    fn is_carpet(&self) -> bool{
        match self{
            Tile::Carpet(_) => true,
            _ => false
        }
    }

    fn update(&mut self, adjacent: &Adjacent<Option<Tile>>){
        match self{
            Tile::None => {}
            Tile::Floor => {}
            Tile::Carpet(index) => {
                for (bit, adj) in [adjacent.n, adjacent.ne, adjacent.e, adjacent.se, adjacent.s, adjacent.sw, adjacent.w, adjacent.nw].iter().enumerate(){
                    index.set_bit(
                        bit,
                        match adj {
                            None => false,
                            Some(t) => t.is_carpet()
                        }
                    );
                }
            }
        }
    }
}

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_SIZE_SIGNED: i32 = CHUNK_SIZE as i32;
pub const CHUNK_SIZE_NEG: i32 = -CHUNK_SIZE_SIGNED;
pub const CHUNK_SIZE_DOUBLE: i32 = 2 * CHUNK_SIZE_SIGNED;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuTileData {
    pub position: [i32; 2],
    pub uv_index: u32,
} impl GpuTileData {
    pub fn vertex_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Sint32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

pub struct Chunk{
    coords: (i32, i32),
    grid: [[Tile; CHUNK_SIZE]; CHUNK_SIZE],
    buffer: Buffer,
    dirty: bool,
} impl Chunk{
    fn create(coords:(i32, i32), gc: &GraphicsControl) -> Self{
        let mut grid: [[Tile; CHUNK_SIZE]; CHUNK_SIZE] = Default::default();

        for y in 0..CHUNK_SIZE{
            for x in 0..CHUNK_SIZE{
                grid[y][x] = match rand::random_bool(0.4){
                    true => {
                        print!("o");
                        Tile::Carpet(0)
                    },
                    false => {
                        print!("x");
                        Tile::Floor
                    },
                }
            }
            println!()
        }
        
        Chunk {
            coords,
            grid,
            buffer: Self::buffer(gc),
            dirty: true,
        }
    }

    fn buffer(gc: &GraphicsControl) -> Buffer{
        gc.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Atlas Instance Buffer"),
                size: (std::mem::size_of::<GpuTileData>() * CHUNK_SIZE * CHUNK_SIZE) as u64,
                usage:
                    wgpu::BufferUsages::VERTEX |
                    wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        )
    }   

    fn to_abs_coords(&self, x: usize, y: usize) -> (i32, i32){
        (x as i32 + self.coords.0 * (CHUNK_SIZE_SIGNED), y as i32 + self.coords.1 * (CHUNK_SIZE_SIGNED))
    }

    fn full_reload(&self, gc: &GraphicsControl){
        let new_buffer: [[GpuTileData; CHUNK_SIZE]; CHUNK_SIZE] = std::array::from_fn(|y| {
            std::array::from_fn(|x| {
                GpuTileData{
                    position: self.to_abs_coords(x, y).into(),
                    // SAFTEY: from_fn gurentees 
                    uv_index: unsafe{self.get(x, y).unwrap_unchecked()}.index()
                }
            })
        });
        gc.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&new_buffer));
    }

    fn get(&self, x: usize, y: usize) -> Option<&Tile>{
        println!("{x}x{y}");
        self.grid.get(y)?.get(x)
    }

    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile>{
        println!("mut {x}x{y}");
        self.grid.get_mut(y)?.get_mut(x)
    }


    fn get_or_adj(&self, x: i32, y: i32, adj: &Adjacent<Option<&Chunk>>) -> Option<Tile> {
        println!("GoA {x}x{y}");
        match (x, y) {
            // NW
            (CHUNK_SIZE_NEG..0, CHUNK_SIZE_NEG..0) =>
                adj.nw.as_deref().and_then(|c|
                    c.get(
                        (x + CHUNK_SIZE_SIGNED) as usize,
                        (y + CHUNK_SIZE_SIGNED) as usize,
                    ).cloned()
                ),

            // North
            (0..CHUNK_SIZE_SIGNED, CHUNK_SIZE_NEG..0) =>
                adj.n.as_deref().and_then(|c|
                    c.get(
                        x as usize,
                        (y + CHUNK_SIZE_SIGNED) as usize,
                    ).cloned()
                ),

            // NE
            (CHUNK_SIZE_SIGNED..CHUNK_SIZE_DOUBLE, CHUNK_SIZE_NEG..0) =>
                adj.ne.as_deref().and_then(|c|
                    c.get(
                        (x - CHUNK_SIZE_SIGNED) as usize,
                        (y + CHUNK_SIZE_SIGNED) as usize,
                    ).cloned()
                ),

            // West
            (CHUNK_SIZE_NEG..0, 0..CHUNK_SIZE_SIGNED) =>
                adj.w.as_deref().and_then(|c|
                    c.get(
                        (x + CHUNK_SIZE_SIGNED) as usize,
                        y as usize,
                    ).cloned()
                ),

            // Center
            (0..CHUNK_SIZE_SIGNED, 0..CHUNK_SIZE_SIGNED) =>
                self.get(x as usize, y as usize).cloned(),

            // East
            (CHUNK_SIZE_SIGNED..CHUNK_SIZE_DOUBLE, 0..CHUNK_SIZE_SIGNED) =>
                adj.e.as_deref().and_then(|c|
                    c.get(
                        (x - CHUNK_SIZE_SIGNED) as usize,
                        y as usize,
                    ).cloned()
                ),

            // SW
            (CHUNK_SIZE_NEG..0, CHUNK_SIZE_SIGNED..CHUNK_SIZE_DOUBLE) =>
                adj.sw.as_deref().and_then(|c|
                    c.get(
                        (x + CHUNK_SIZE_SIGNED) as usize,
                        (y - CHUNK_SIZE_SIGNED) as usize,
                    ).cloned()
                ),

            // South
            (0..CHUNK_SIZE_SIGNED, CHUNK_SIZE_SIGNED..CHUNK_SIZE_DOUBLE) =>
                adj.s.as_deref().and_then(|c|
                    c.get(
                        x as usize,
                        (y - CHUNK_SIZE_SIGNED) as usize,
                    ).cloned()
                ),

            // SE
            (CHUNK_SIZE_SIGNED..CHUNK_SIZE_DOUBLE, CHUNK_SIZE_SIGNED..CHUNK_SIZE_DOUBLE) =>
                adj.se.as_deref().and_then(|c|
                    c.get(
                        (x - CHUNK_SIZE_SIGNED) as usize,
                        (y - CHUNK_SIZE_SIGNED) as usize,
                    ).cloned()
                ),

            _ => None,
        }
    }

    fn update_tiles(&mut self, rect: Rect<usize>, adj: &Adjacent<Option<&Chunk>>){
        for y in rect.y_iter(){
            println!("Construct ADJ {y}");
            let mut t_adj = Adjacent::construct_from_fn_offset(|dx, dy| {
                self.get_or_adj(rect.x as i32 + dx, y as i32 + dy, adj)
            });
            for x in rect.x_iter(){
                println!("Get center {x}x{y}");
                let c = self.get_mut(x, y).unwrap();
                c.update(&t_adj);

                println!("Get move");
                t_adj.move_east(
                    Some(c.clone()),
                    self.get_or_adj(x as i32 + 2, y as i32 - 1, adj),
                    self.get_or_adj(x as i32 + 2, y as i32, adj),
                    self.get_or_adj(x as i32 + 2, y as i32 + 1, adj),
                );
            }
        }
    }

    fn render(&mut self, gc: &GraphicsControl, render_pass: &mut RenderPass){
        if self.dirty{
            self.full_reload(gc);
            self.dirty = false;
        }
        render_pass.set_vertex_buffer(1, self.buffer.slice(..));

        render_pass.draw_indexed(
            0..6,                  // six indices for the quad
            0,                     // index buffer offset
            0..(CHUNK_SIZE * CHUNK_SIZE) as u32, // instances
        );
    }
}

pub struct ChunkManager {
    atlas: AssetKey<GpuTexture>,
    tileset_json: TileSetJson,
    table: AHashMap<(i32, i32), Rc<RefCell<Chunk>>>,

    gc: GraphicsControl,

    pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    transform_buffer: Buffer,
    uv_buffer: Buffer,

    bind_group: BindGroup,

    center: Vector2<f32>,
    scale: f32
} impl ChunkManager{
    pub fn create(gc: GraphicsControl, sfmt: TextureFormat, asset_mgr: &mut AssetManager) -> GResult<Self>{
        let tileset_json: TileSetJson = serde_json::from_str(
            str::from_utf8(
                asset_mgr.vfs.read_all("assets/tileset.json").g_err()?
            ).g_err()?
        ).g_err()?;

        let (atlas, atlas_ref) = asset_mgr.get_or_create_asset("/assets/tileatlas.png", load_texture(gc.clone(), "TILE_ATLAS".into()))?;
        
        let vertex_buffer = Self::vertex_buffer(&gc);
        let index_buffer = Self::index_buffer(&gc);

        let scale = 1.5;
        let center = Vector2::new(CHUNK_SIZE as f32 / 2., CHUNK_SIZE as f32 / 2.);
        let transform = Self::calculate_world_to_cvv(&gc, tileset_json.size.clone(), scale.clone(), center.clone());
        let transform_buffer = Self::transform_buffer(&gc, &transform);

        let uv_list = tileset_json.create_uv_list();
        let uv_buffer = Self::uv_buffer(&gc, &uv_list);

        let bind_group_layout = Self::bind_group_layout(&gc);
        let bind_group = Self::bind_group(&gc, &bind_group_layout, atlas_ref, &transform_buffer, &uv_buffer);
        let shader = Self::shader_module(&gc);
        let pipeline_layout = Self::pipeline_layout(&gc, &bind_group_layout);
        let pipeline = Self::pipline(&gc, &pipeline_layout, &shader, sfmt);

        Ok(
            ChunkManager{
                atlas,
                tileset_json,
                table: AHashMap::new(),

                gc,

                pipeline,
                bind_group_layout,
                vertex_buffer,
                index_buffer,
                transform_buffer,
                uv_buffer,

                bind_group,
                center,
                scale
            }
        )
    }

    const QUAD_VERTICES: &[f32] = &[
        0.0, 0.0,
        1.0, 0.0,
        1.0, 1.0,
        0.0, 1.0,
    ];

    const QUAD_INDICES: &[u16] = &[
        0, 1, 2,
        2, 3, 0,
    ];

    fn vertex_buffer(gc: &GraphicsControl) -> Buffer{
        gc.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("ChunkRederer Vertex Buffer"),
                contents: bytemuck::cast_slice(Self::QUAD_VERTICES),
                usage: wgpu::BufferUsages::VERTEX
            }
        )
    }

    fn index_buffer(gc: &GraphicsControl) -> Buffer{
        gc.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("ChunkRenderer Index Buffer"),
                contents: bytemuck::cast_slice(Self::QUAD_INDICES),
                usage: wgpu::BufferUsages::INDEX
            }
        )
    }

    fn transform_buffer(gc: &GraphicsControl, transform: &Matrix4<f32>) -> Buffer {
        gc.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("ChunkRenderer Transform Buffer"),
                contents: bytemuck::cast_slice(transform.as_slice()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        )
    }

    fn uv_buffer(gc: &GraphicsControl, uvs: &[[f32; 4]]) -> Buffer {
        gc.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("ChunkRenderer UV Buffer"),
                contents: bytemuck::cast_slice(uvs),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            },
        )
    }

    fn bind_group_layout(gc: &GraphicsControl) -> BindGroupLayout {
        gc.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("ChunkRenderer Texture Layout"),
                entries: &[
                    // Atlas texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                        },
                        count: None,
                    },

                    // Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },

                    // World -> CVV transform
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },

                    // UV table
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage {
                                read_only: true,
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            },
        )
    }
    
    fn bind_group(gc: &GraphicsControl, layout: &BindGroupLayout, atlas: &GpuTexture, transform_buffer: &Buffer, uv_buffer: &Buffer) -> BindGroup {
        gc.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("ChunkRenderer Bind Group"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &atlas.view,
                        ),
                    },

                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            &atlas.sampler,
                        ),
                    },

                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: transform_buffer.as_entire_binding(),
                    },

                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: uv_buffer.as_entire_binding(),
                    },
                ],
            },
        )
    }

    fn shader_module(gc: &GraphicsControl) -> ShaderModule{
        gc.device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("ChunkRenderer Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../../assets/chunk.wgsl").into()
                ),
            },
        )

    }

    fn pipeline_layout(gc: &GraphicsControl, bgl: &BindGroupLayout) -> PipelineLayout{
        gc.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("ChunkRenderer Pipeline Layout"),
                bind_group_layouts: &[Some(&bgl)],
                immediate_size: 0
            },
        )
    }

    fn pipline(gc: &GraphicsControl, pl: &PipelineLayout, s: &ShaderModule, sfmt: TextureFormat) -> RenderPipeline{
        gc.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("ChunkRenderer Pipeline"),

                layout: Some(&pl),

                vertex: wgpu::VertexState {
                    module: &s,
                    entry_point: Some("vs_main"),
                    compilation_options:
                        wgpu::PipelineCompilationOptions::default(),

                    buffers: &[
                        // Quad vertex
                        Some(wgpu::VertexBufferLayout {
                            array_stride: 2 * 4,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    offset: 0,
                                    shader_location: 0,
                                    format: wgpu::VertexFormat::Float32x2,
                                },
                            ],
                        }),

                        // Instance
                        Some(GpuTileData::vertex_layout()),
                    ],
                },

                fragment: Some(wgpu::FragmentState {
                    module: &s,
                    entry_point: Some("fs_main"),
                    compilation_options:
                        wgpu::PipelineCompilationOptions::default(),

                    targets: &[Some(
                        wgpu::ColorTargetState {
                            format: sfmt,
                            blend: Some(
                                wgpu::BlendState::ALPHA_BLENDING
                            ),
                            write_mask:
                                wgpu::ColorWrites::ALL,
                        }
                    )],
                }),

                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },

                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),

                multisample: wgpu::MultisampleState::default(),

                cache: None,

                multiview_mask: None
            },
        )
    }

    fn calculate_world_to_cvv(gc: &GraphicsControl, size: [f32; 2], scale: f32, center: Vector2<f32>) -> Matrix4<f32> {
    let dim = gc.get_dim();

    let screen = Vector2::new(dim.0 as f32, dim.1 as f32);

    // Number of tile pixels visible in each direction.
    let visible_pixels = screen / scale;

    // Convert screen pixels to tile/world units.
    let visible_world = visible_pixels.component_div(&Vector2::from(size));

    let half_world = visible_world / 2.0;

    let min = center - half_world;
    let max = center + half_world;

    let tz = RenderLayer::Tile as u8 as f32;
    let rl_max = RenderLayer::COUNT as f32;

    coordinate_transform(
        vec2_to_vec3(min, 0.0),
        vec2_to_vec3(max, 1.0),
        Vector3::new(-1.0, 1.0, 1.0 - tz / rl_max),
        Vector3::new(1.0, -1.0, 1.0 - (tz + 1.0) / rl_max),
    )
}

    pub fn reload_transform(&mut self){
        let transform = Self::calculate_world_to_cvv(&self.gc, self.tileset_json.size.clone(), self.scale.clone(), self.center.clone());
        self.gc.queue.write_buffer(&self.transform_buffer, 0, bytemuck::cast_slice(&transform.into_gmat()));
    }

    pub fn create_chunk(&mut self, ccoords: (i32, i32)) -> &Rc<RefCell<Chunk>>{
        let mut chunk = Chunk::create(ccoords, &self.gc);
        chunk.update_tiles(Rect { x: 0, y: 0, w: CHUNK_SIZE, h: CHUNK_SIZE }, &Adjacent { nw: None, n: None, ne: None, w: None, e: None, sw: None, s: None, se: None });
        self.table.entry(ccoords).or_insert_with(|| Rc::new(RefCell::new(chunk)))
    }

    pub fn render(&mut self, render_pass: &mut RenderPass) {
        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.set_vertex_buffer(
            0,
            self.vertex_buffer.slice(..),
        );

        render_pass.set_index_buffer(
            self.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        ); 

        for (_coords, chunk) in &self.table{
            chunk.borrow_mut().render(&self.gc, render_pass);
        }
    }
}