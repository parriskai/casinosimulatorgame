use std::{collections::HashMap, hash::RandomState};

use bytemuck::{Pod, Zeroable};
use nalgebra::Matrix4;
use wgpu::util::DeviceExt;

use crate::{graphics::{UvBox, asset_mgr::{AssetManager, TextureKey, TextureOrMissing}, graphicscontrol::GraphicsControl}, utils::IntoGPUMatrix};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct AtlasInstance{
    transform: [[f32; 4]; 4],
    uv: UvBox
} impl AtlasInstance {
    fn vertex_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
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

    buckets: HashMap<TextureKey, AtlasBucket>,

    bind_group_layout: wgpu::BindGroupLayout,
} impl AtlasRenderer{
    pub fn create(gc: GraphicsControl, surface_format: wgpu::TextureFormat) -> AtlasRenderer{
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

        let vertex_buffer = gc.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("AtlasRederer Vertex Buffer"),
                contents: bytemuck::cast_slice(QUAD_VERTICES),
                usage: wgpu::BufferUsages::VERTEX
            }
        );

        let index_buffer = gc.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("AtlasRenderer Index Buffer"),
                contents: bytemuck::cast_slice(QUAD_INDICES),
                usage: wgpu::BufferUsages::INDEX
            }
        );
        
        let bind_group_layout =
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
            );
        
        let shader = gc.device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("AtlasRenderer Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../../assets/atlas.wgsl").into()
                ),
            },
        );

        let pipeline_layout =
            gc.device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("AtlasRenderer Pipeline Layout"),
                    bind_group_layouts: &[Some(&bind_group_layout)],
                    immediate_size: 0
                },
            );

        let pipeline = gc.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("AtlasRenderer Pipeline"),

                layout: Some(&pipeline_layout),

                vertex: wgpu::VertexState {
                    module: &shader,
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
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options:
                        wgpu::PipelineCompilationOptions::default(),

                    targets: &[Some(
                        wgpu::ColorTargetState {
                            format: surface_format,
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

                depth_stencil: None,

                multisample: wgpu::MultisampleState::default(),

                cache: None,

                multiview_mask: None
            },
        );

        AtlasRenderer{
            gc,
            pipeline,
            vertex_buffer,
            index_buffer,
            buckets: HashMap::new(),
            bind_group_layout,
        }
    }

    pub fn draw_atlas<T: IntoGPUMatrix<RAW = [[f32; 4]; 4]>>(&mut self, tkey: TextureKey, uv: UvBox, transform: T) {
        let bucket = self
            .buckets
            .entry(tkey)
            .or_insert_with(|| AtlasBucket {
                instances: Vec::new(),
                instance_buffer: None,
            });

        bucket.instances.push(AtlasInstance {
            transform: transform.into_gmat(),
            uv
        });
    }

    pub fn render_all<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>, asset_mgr: &AssetManager) {
        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_vertex_buffer(
            0,
            self.vertex_buffer.slice(..),
        );

        render_pass.set_index_buffer(
            self.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );

        for (&texture_key, bucket) in self.buckets.iter_mut() {
            if bucket.instances.is_empty() {
                continue;
            }

            let texture = match asset_mgr.texture_by_id(texture_key){
                TextureOrMissing::Texture(t) => t,
                TextureOrMissing::Missing(t) => {
                    // Full Missing Texture
                    for v in bucket.instances.iter_mut(){
                        v.uv = UvBox::FULL
                    }
                    t
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

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}