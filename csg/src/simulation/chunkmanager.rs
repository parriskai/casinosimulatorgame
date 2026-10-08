use crate::{
    graphics::{
        RenderLayer, UvBox, assets::{
            gputexture::{GpuTexture, load_texture}, manager::{AssetKey, AssetManager}
        }, graphicscontrol::GraphicsControl
    }, prelude::*, utils::{IntoGpuMatrix, coordinate_transform, vec2_to_vec3}
};
use ahash::AHashMap;
use bytemuck::{Pod, Zeroable};
use nalgebra::{Matrix4, Vector2, Vector3};
use serde::Deserialize;
use strum::EnumCount;
use wgpu::{BindGroup, BindGroupLayout, Buffer, PipelineLayout, RenderPass, RenderPipeline, ShaderModule, TextureFormat, util::DeviceExt};

#[derive(Debug, Deserialize)]
pub struct TileSetJson{
    size: [f32; 2],
    tiles: AHashMap<String, UvBox>
} impl TileSetJson{
    fn create_uv_list(&self) -> Vec<[f32; 4]>{
        let mut listing = Vec::new();
        for (tile_name, tile_uv) in &self.tiles{
            match Tile::get_index_for_name(tile_name){
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
        listing
    }
}

#[derive(Debug, Default)]
pub enum Tile{
    None,
    #[default]
    Floor,
} impl Tile{
    fn index(&self) -> u32{
        match self{
            Tile::None => 0,
            Tile::Floor => 1
        }
    }
    fn get_index_for_name(s: &str) -> Option<usize>{
        match s{
            "none" => Some(0),
            "floor" => Some(1),

            _ => None
        }
    }
}

pub const CHUNK_SIZE: usize = 16;

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
    dirty: bool
} impl Chunk{
    fn create(coords:(i32, i32), gc: &GraphicsControl) -> Self{
        let mut grid: [[Tile; CHUNK_SIZE]; CHUNK_SIZE] = Default::default();

        Chunk {
            coords,
            grid,
            buffer: Self::buffer(gc),
            dirty: true
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

    fn full_reload(&self, gc: &GraphicsControl){
        let new_buffer: [[GpuTileData; CHUNK_SIZE]; CHUNK_SIZE] = std::array::from_fn(|y| {
            std::array::from_fn(|x| {
                GpuTileData{
                    position: [x as i32 + self.coords.0 * (CHUNK_SIZE as i32), y as i32 + self.coords.1 * (CHUNK_SIZE as i32)],
                    uv_index: self.get(x, y).index()
                }
            })
        });
        gc.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&new_buffer));
    }

    fn get(&self, x: usize, y: usize) -> &Tile{
        &self.grid[y][x]
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
    table: AHashMap<(i32, i32), Chunk>,

    gc: GraphicsControl,

    pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    transform_buffer: Buffer,
    uv_buffer: Buffer,

    bind_group: BindGroup,

    pub scale: f32,
    pub center: Vector2<f32>
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

        let scale = 2.;
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

    fn calculate_world_to_cvv(
    gc: &GraphicsControl,
    size: [f32; 2],
    scale: f32,
    center: Vector2<f32>,
) -> Matrix4<f32> {
    let dim = (2256, 1469); //gc.get_dim();

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

    pub fn create_chunk(&mut self, ccoords: (i32, i32)) -> &mut Chunk{
        self.table.entry(ccoords).or_insert_with(|| Chunk::create(ccoords, &self.gc))
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

        for (_coords, chunk) in &mut self.table{
            chunk.render(&self.gc, render_pass);
        }
    }
}