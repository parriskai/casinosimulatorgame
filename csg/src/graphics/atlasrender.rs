use crate::{
    graphics::{
        RenderLayer, UvBox, assets::{
            gputexture::GpuTexture,
            manager::{
                AssetKey,
                AssetManager
            }
        }, graphicscontrol::GraphicsControl, linerenderer::LineRenderer
    }, utils::{
        IntoGpuMatrix,
        coordinate_transform
    }
};
use wgpu::{
    BindGroupLayout,
    Buffer,
    PipelineLayout,
    RenderPipeline,
    ShaderModule,
    TextureFormat,
    util::DeviceExt
};
use nalgebra::{
    Matrix4,
    Vector2,
    Vector3
};
use bytemuck::{
    Pod,
    Zeroable
};
use ahash::AHashMap;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct AtlasInstance{
    pub transform: [[f32; 4]; 4],
    pub uv: UvBox
} impl AtlasInstance {
    pub fn vertex_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: &[wgpu::VertexAttribute] =
            &wgpu::vertex_attr_array![
                1 => Float32x4,
                2 => Float32x4,
                3 => Float32x4,
                4 => Float32x4,
                5 => Float32x2,
                6 => Float32x2,
            ];

        wgpu::VertexBufferLayout {
            array_stride:
                std::mem::size_of::<AtlasInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRIBUTES,
        }
    }
}

struct AtlasBucket{
    instances: Vec<AtlasInstance>,
    instance_buffer: Option<wgpu::Buffer>
}

pub struct AtlasRenderer{
    gc: GraphicsControl,

    pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    buckets: AHashMap<(RenderLayer, AssetKey<GpuTexture>), AtlasBucket>,

    world_to_cvv: Matrix4<f32>,

    bind_group_layout: wgpu::BindGroupLayout,

    #[cfg(any(debug_assertions, feature = "debug_tools"))]
    debug_draw_lines: Vec<(Vector2<f32>, Vector2<f32>)>
} impl AtlasRenderer{
    pub fn create(gc: GraphicsControl, surface_format: wgpu::TextureFormat) -> AtlasRenderer{
        let vertex_buffer = Self::vertex_buffer(&gc);

        let index_buffer = Self::index_buffer(&gc);
        
        let bind_group_layout = Self::bind_group_layout(&gc);
        
        let shader = Self::shader_module(&gc);

        let pipeline_layout = Self::pipeline_layout(&gc, &bind_group_layout);

        let pipeline = Self::pipline(&gc, &pipeline_layout, &shader, surface_format);

        let world_to_cvv = gc.get_world_to_cvv();

        AtlasRenderer{
            gc,
            pipeline,
            vertex_buffer,
            index_buffer,
            buckets: AHashMap::new(),
            world_to_cvv,
            bind_group_layout,

            #[cfg(any(debug_assertions, feature = "debug_tools"))]
            debug_draw_lines: Vec::new()
        }
    }

    const QUAD_VERTICES: &[f32] = &[
        -1.0, -1.0,
         1.0, -1.0,
         1.0,  1.0,
        -1.0,  1.0,
    ];

    const QUAD_INDICES: &[u16] = &[
        0, 1, 2,
        2, 3, 0,
    ];

    fn vertex_buffer(gc: &GraphicsControl) -> Buffer{
        gc.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("AtlasRederer Vertex Buffer"),
                contents: bytemuck::cast_slice(Self::QUAD_VERTICES),
                usage: wgpu::BufferUsages::VERTEX
            }
        )
    }

    fn index_buffer(gc: &GraphicsControl) -> Buffer{
        gc.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("AtlasRenderer Index Buffer"),
                contents: bytemuck::cast_slice(Self::QUAD_INDICES),
                usage: wgpu::BufferUsages::INDEX
            }
        )
    }

    fn bind_group_layout(gc: &GraphicsControl) -> BindGroupLayout{
        gc.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("AtlasRenderer Texture Layout"),
                entries: &[
                    // Texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension:
                                wgpu::TextureViewDimension::D2,
                            sample_type:
                                wgpu::TextureSampleType::Float {
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
                ],
            },
        )
    }

    pub fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    fn shader_module(gc: &GraphicsControl) -> ShaderModule{
        gc.device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("AtlasRenderer Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../../assets/atlas.wgsl").into()
                ),
            },
        )

    }

    fn pipeline_layout(gc: &GraphicsControl, bgl: &BindGroupLayout) -> PipelineLayout{
        gc.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("AtlasRenderer Pipeline Layout"),
                bind_group_layouts: &[Some(&bgl)],
                immediate_size: 0
            },
        )
    }

    fn pipline(gc: &GraphicsControl, pl: &PipelineLayout, s: &ShaderModule, sfmt: TextureFormat) -> RenderPipeline{
        gc.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("AtlasRenderer Pipeline"),

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
                        Some(AtlasInstance::vertex_layout()),
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

    pub fn draw_atlas(&mut self, tkey: AssetKey<GpuTexture>, uv: UvBox, position: (Vector2<f32>, Vector2<f32>, RenderLayer)) {
        #[cfg(any(debug_assertions, feature = "debug_tools"))]
        {
            self.debug_draw_lines.push((position.0.clone(), position.1.clone()));
        }

        let bucket = self
            .buckets
            .entry((position.2, tkey))
            .or_insert_with(|| AtlasBucket {
                instances: Vec::new(),
                instance_buffer: None,
            });
        
        let transform = self.world_to_cvv * coordinate_transform(
                Vector3::new(-1., -1., 0.),
                Vector3::new(1., 1., 1.),
                Vector3::new(position.0.x, position.0.y, position.2 as u8 as f32),
                Vector3::new(position.1.x, position.1.y, position.2 as u8 as f32));
        
        bucket.instances.push(AtlasInstance {
            transform: transform.into_gmat(),
            uv
        });
    }

    pub fn update_world_to_cvv(&mut self){
        self.world_to_cvv = self.gc.get_world_to_cvv();
    }

    #[cfg(any(debug_assertions, feature = "debug_tools"))]
    pub fn draw_debug_lines(&mut self, lr: &mut LineRenderer){
        for (nw, se) in self.debug_draw_lines.drain(..) {
            let ne = Vector2::new(se.x, nw.y);
            let sw = Vector2::new(nw.x, se.y);
            lr.draw_polyline(&[nw, ne, se, sw, nw], &[[1., 0., 0., 1.]; 5], RenderLayer::Debug);
        }
    }


    pub fn render_all(&mut self, render_pass: &mut wgpu::RenderPass<'_>, asset_mgr: &AssetManager) {
        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_vertex_buffer(
            0,
            self.vertex_buffer.slice(..),
        );

        render_pass.set_index_buffer(
            self.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );

        let mut buckets: Vec<_> =self.buckets.iter_mut().collect();
        buckets.sort_by(|((a, _), _), ((b, _), _)| a.cmp(b));

        for ((__, texture_key), bucket) in buckets{
            if bucket.instances.is_empty() {
                continue;
            }

            let texture = match asset_mgr.get_asset(*texture_key){
                Some(t) => t,
                None => {
                    // Full Missing Texture
                    for v in bucket.instances.iter_mut(){
                        v.uv = UvBox::FULL
                    }
                    asset_mgr.get_default().unwrap()
                }
            };

            let required_size =
                (bucket.instances.len()
                    * std::mem::size_of::<AtlasInstance>())
                    as u64;

            let buffer = match &bucket.instance_buffer {
                Some(buffer) if buffer.size() >= required_size => buffer,

                _ => {
                    let buffer = self.gc.device.create_buffer(
                        &wgpu::BufferDescriptor {
                            label: Some("Atlas Instance Buffer"),
                            size: required_size,
                            usage:
                                wgpu::BufferUsages::VERTEX |
                                wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        },
                    );

                    bucket.instance_buffer = Some(buffer);

                    bucket.instance_buffer.as_ref().unwrap()
                }
            };

            self.gc.queue.write_buffer(
                buffer,
                0,
                bytemuck::cast_slice(&bucket.instances),
            );

            render_pass.set_bind_group(
                0,
                texture.bind_group(),
                &[],
            );

            render_pass.set_vertex_buffer(
                1,
                buffer.slice(..),
            );

            render_pass.draw_indexed(
                0..6,
                0,
                0..bucket.instances.len() as u32,
            );
        }
        self.clear();
    }

    pub fn clear(&mut self) {
        for bucket in self.buckets.values_mut() {
            bucket.instances.clear();
        }
    }
}