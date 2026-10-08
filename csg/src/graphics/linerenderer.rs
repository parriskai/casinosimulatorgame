use crate::{
    graphics::{
        RenderLayer,
        graphicscontrol::GraphicsControl,
    },
    utils::IntoGpuMatrix,
};

use ahash::AHashMap;
use bytemuck::{Pod, Zeroable};
use nalgebra::{Matrix4, Vector2, Vector3, Vector4};
use wgpu::{
    Buffer,
    PipelineLayout,
    RenderPipeline,
    ShaderModule,
    TextureFormat,
    util::DeviceExt,
};


#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct LineVertex {
    // 0.0 = first point
    // 1.0 = second point
    t: f32,
}

impl LineVertex {
    fn vertex_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: &[wgpu::VertexAttribute] =
            &wgpu::vertex_attr_array![
                0 => Float32,
            ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRIBUTES,
        }
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct LineInstance {
    // Index into the point buffer.
    start: u32,

    // Index into the point buffer.
    end: u32,

    start_color: [f32; 4],
    end_color: [f32; 4],
}

impl LineInstance {
    fn vertex_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: &[wgpu::VertexAttribute] =
            &wgpu::vertex_attr_array![
                1 => Uint32,
                2 => Uint32,
                3 => Float32x4,
                4 => Float32x4,
            ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRIBUTES,
        }
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct GpuPoint {
    position: [f32; 4],
}


struct LineBucket {
    layer: RenderLayer,

    // CPU-side line instances.
    instances: Vec<LineInstance>,

    // GPU-side line instances.
    instance_buffer: Option<Buffer>,
}


pub struct LineRenderer {
    gc: GraphicsControl,

    pipeline: RenderPipeline,

    // The two shared line vertices.
    vertex_buffer: Buffer,

    // All unique points used by this frame.
    point_buffer: Option<Buffer>,

    // CPU point table.
    points: Vec<GpuPoint>,

    // Used to deduplicate points.
    point_lookup: AHashMap<(u32, u32, u32, bool), u32>,

    buckets: Vec<LineBucket>,

    world_to_cvv: Matrix4<f32>,
}


impl LineRenderer {
    pub fn create(
        gc: GraphicsControl,
        surface_format: TextureFormat,
    ) -> Self {
        let vertex_buffer = Self::vertex_buffer(&gc);

        let shader = Self::shader_module(&gc);

        let bind_group_layout =
            Self::bind_group_layout(&gc);

        let pipeline_layout =
            Self::pipeline_layout(
                &gc,
                &bind_group_layout,
            );

        let pipeline =
            Self::pipeline(
                &gc,
                &pipeline_layout,
                &shader,
                surface_format,
            );

        Self {
            world_to_cvv: gc.get_world_to_cvv(),

            gc,

            pipeline,

            vertex_buffer,

            point_buffer: None,

            points: Vec::new(),

            point_lookup: AHashMap::new(),

            buckets: Vec::new(),
        }
    }


    const LINE_VERTICES: &[LineVertex] = &[
        LineVertex { t: 0.0 },
        LineVertex { t: 1.0 },
    ];


    fn vertex_buffer(gc: &GraphicsControl) -> Buffer {
        gc.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(
                    "LineRenderer Vertex Buffer"
                ),

                contents: bytemuck::cast_slice(
                    Self::LINE_VERTICES
                ),

                usage:
                    wgpu::BufferUsages::VERTEX,
            }
        )
    }


    fn bind_group_layout(gc: &GraphicsControl) -> wgpu::BindGroupLayout {
        gc.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some(
                    "LineRenderer Bind Group Layout"
                ),

                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,

                        visibility:
                            wgpu::ShaderStages::VERTEX,

                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage {
                                read_only: true,
                            },

                            has_dynamic_offset: false,

                            min_binding_size: None,
                        },

                        count: None,
                    }
                ],
            }
        )
    }


    fn shader_module(gc: &GraphicsControl) -> ShaderModule {
        gc.device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some(
                    "LineRenderer Shader"
                ),

                source: wgpu::ShaderSource::Wgsl(
                    include_str!(
                        "../../../assets/line.wgsl"
                    ).into()
                ),
            }
        )
    }


    fn pipeline_layout(gc: &GraphicsControl, bgl: &wgpu::BindGroupLayout) -> PipelineLayout {
        gc.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some(
                    "LineRenderer Pipeline Layout"
                ),

                bind_group_layouts: &[
                    Some(bgl)
                ],

                immediate_size: 0,
            }
        )
    }


    fn pipeline(gc: &GraphicsControl, pl: &PipelineLayout, shader: &ShaderModule, surface_format: TextureFormat) -> RenderPipeline {
        gc.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some(
                    "LineRenderer Pipeline"
                ),

                layout: Some(pl),

                vertex: wgpu::VertexState {
                    module: shader,

                    entry_point:
                        Some("vs_main"),

                    compilation_options:
                        wgpu::PipelineCompilationOptions::default(),

                    buffers: &[
                        Some(
                            LineVertex::vertex_layout()
                        ),

                        Some(
                            LineInstance::vertex_layout()
                        ),
                    ],
                },

                fragment: Some(
                    wgpu::FragmentState {
                        module: shader,

                        entry_point:
                            Some("fs_main"),

                        compilation_options:
                            wgpu::PipelineCompilationOptions::default(),

                        targets: &[
                            Some(
                                wgpu::ColorTargetState {
                                    format: surface_format,

                                    blend: Some(
                                        wgpu::BlendState::ALPHA_BLENDING
                                    ),

                                    write_mask:
                                        wgpu::ColorWrites::ALL,
                                }
                            )
                        ],
                    }
                ),

                primitive: wgpu::PrimitiveState {
                    topology:
                        wgpu::PrimitiveTopology::LineList,

                    ..Default::default()
                },

                depth_stencil: Some(
                    wgpu::DepthStencilState {
                        format:
                            wgpu::TextureFormat::Depth32Float,

                        depth_write_enabled: Some(true),

                        depth_compare: Some(wgpu::CompareFunction::LessEqual),

                        stencil:
                            Default::default(),

                        bias:
                            Default::default(),
                    }
                ),

                multisample:
                    wgpu::MultisampleState::default(),

                cache: None,

                multiview_mask: None,
            }
        )
    }


    pub fn update_world_to_cvv(&mut self) {
        self.world_to_cvv =
            self.gc.get_world_to_cvv();
    }


    /*
     * Add/retrieve a point.
     *
     * Points are specified in pixel/world space and
     * converted to CVV exactly once.
     */
    fn get_point(&mut self, position: Vector2<f32>, layer: RenderLayer) -> u32 {
        let key = (
            position.x.to_bits(),
            position.y.to_bits(),
            layer as u32,
            false,
        );

        if let Some(&index) =
            self.point_lookup.get(&key)
        {
            return index;
        }

        /*
         * Convert from pixel/world coordinates to
         * clip-space on the CPU.
         *
         * RenderLayer becomes the Z coordinate.
         */
        let position = self.world_to_cvv
            * Vector4::new(
                position.x,
                position.y,
                layer as u8 as f32,
                1.0,
            );

        let point = GpuPoint {
            position: position.into(),
        };

        let index =
            self.points.len() as u32;

        self.points.push(point);

        self.point_lookup.insert(
            key,
            index,
        );

        index
    }

    fn get_raw_point(&mut self, position: Vector3<f32>) -> u32 {
        let key = (
            position.x.to_bits(),
            position.y.to_bits(),
            position.z.to_bits(),
            true,
        );

        if let Some(&index) =
            self.point_lookup.get(&key)
        {
            return index;
        }

        /*
         * Convert from pixel/world coordinates to
         * clip-space on the CPU.
         *
         * RenderLayer becomes the Z coordinate.
         */
        let position = Vector4::new(
                position.x,
                position.y,
                position.z,
                1.0,
            );

        let point = GpuPoint {
            position: position.into(),
        };

        let index =
            self.points.len() as u32;

        self.points.push(point);

        self.point_lookup.insert(
            key,
            index,
        );

        index
    }


    pub fn draw_line(
        &mut self,

        start: Vector2<f32>,
        end: Vector2<f32>,

        start_color: [f32; 4],
        end_color: [f32; 4],

        layer: RenderLayer,
    ) {
        let start =
            self.get_point(start, layer);

        let end =
            self.get_point(end, layer);

        let bucket = self
            .buckets
            .iter_mut()
            .find(|bucket|
                bucket.layer == layer
            );

        let instance = LineInstance {
            start,
            end,
            start_color,
            end_color,
        };

        match bucket {
            Some(bucket) => {
                bucket.instances.push(instance);
            }

            None => {
                self.buckets.push(
                    LineBucket {
                        layer,

                        instances:
                            vec![instance],

                        instance_buffer:
                            None,
                    }
                );
            }
        }
    }

    pub fn draw_raw_line(
        &mut self,

        start: Vector3<f32>,
        end: Vector3<f32>,

        start_color: [f32; 4],
        end_color: [f32; 4],

        layer: RenderLayer
    ) {
        let start =
            self.get_raw_point(start);

        let end =
            self.get_raw_point(end);

        let bucket = self
            .buckets
            .iter_mut()
            .find(|bucket|
                bucket.layer == layer
            );

        let instance = LineInstance {
            start,
            end,
            start_color,
            end_color,
        };

        match bucket {
            Some(bucket) => {
                bucket.instances.push(instance);
            }

            None => {
                self.buckets.push(
                    LineBucket {
                        layer,

                        instances:
                            vec![instance],

                        instance_buffer:
                            None,
                    }
                );
            }
        }
    }


    pub fn draw_polyline(
        &mut self,

        points: &[Vector2<f32>],
        colors: &[[f32; 4]],

        layer: RenderLayer,
    ) {
        assert!(
            points.len() >= 2
        );

        assert_eq!(
            points.len(),
            colors.len()
        );

        for i in 0..points.len() - 1 {
            self.draw_line(
                points[i],
                points[i + 1],

                colors[i],
                colors[i + 1],

                layer,
            );
        }
    }

    pub fn draw_raw_polyline(
        &mut self,

        points: &[Vector3<f32>],
        colors: &[[f32; 4]],

        layer: RenderLayer,
    ) {
        assert!(
            points.len() >= 2
        );

        assert_eq!(
            points.len(),
            colors.len()
        );

        for i in 0..points.len() - 1 {
            self.draw_raw_line(
                points[i],
                points[i + 1],

                colors[i],
                colors[i + 1],

                layer,
            );
        }
    }


    pub fn render_all(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        if self.points.is_empty() {
            return;
        }

        /*
         * Upload the unique point table once.
         */
        let required_point_size =
            (self.points.len()
                * std::mem::size_of::<GpuPoint>())
                as u64;

        let point_buffer =
            match &self.point_buffer {
                Some(buffer)
                    if buffer.size()
                        >= required_point_size =>
                {
                    buffer
                }

                _ => {
                    let buffer =
                        self.gc.device.create_buffer(
                            &wgpu::BufferDescriptor {
                                label: Some(
                                    "Line Point Buffer"
                                ),

                                size:
                                    required_point_size,

                                usage:
                                    wgpu::BufferUsages::STORAGE |
                                    wgpu::BufferUsages::COPY_DST,

                                mapped_at_creation:
                                    false,
                            }
                        );

                    self.point_buffer =
                        Some(buffer);

                    self.point_buffer
                        .as_ref()
                        .unwrap()
                }
            };

        self.gc.queue.write_buffer(
            point_buffer,
            0,
            bytemuck::cast_slice(
                &self.points
            ),
        );

        let bind_group =
            self.gc.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some(
                        "LineRenderer Bind Group"
                    ),

                    layout:
                        &self.pipeline.get_bind_group_layout(0),

                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,

                            resource:
                                point_buffer.as_entire_binding(),
                        }
                    ],
                }
            );


        render_pass.set_pipeline(
            &self.pipeline
        );

        render_pass.set_bind_group(
            0,
            &bind_group,
            &[],
        );

        render_pass.set_vertex_buffer(
            0,
            self.vertex_buffer.slice(..),
        );


        /*
         * Render layers in order.
         */
        self.buckets.sort_unstable_by(
            |a, b| a.layer.cmp(&b.layer)
        );


        for bucket in self.buckets.iter_mut() {
            if bucket.instances.is_empty() {
                continue;
            }

            let required_size =
                (bucket.instances.len()
                    * std::mem::size_of::<LineInstance>())
                    as u64;

            let instance_buffer =
                match &bucket.instance_buffer {
                    Some(buffer)
                        if buffer.size()
                            >= required_size =>
                    {
                        buffer
                    }

                    _ => {
                        let buffer =
                            self.gc.device.create_buffer(
                                &wgpu::BufferDescriptor {
                                    label: Some(
                                        "Line Instance Buffer"
                                    ),

                                    size:
                                        required_size,

                                    usage:
                                        wgpu::BufferUsages::VERTEX |
                                        wgpu::BufferUsages::COPY_DST,

                                    mapped_at_creation:
                                        false,
                                }
                            );

                        bucket.instance_buffer =
                            Some(buffer);

                        bucket.instance_buffer
                            .as_ref()
                            .unwrap()
                    }
                };


            self.gc.queue.write_buffer(
                instance_buffer,
                0,
                bytemuck::cast_slice(
                    &bucket.instances
                ),
            );


            render_pass.set_vertex_buffer(
                1,
                instance_buffer.slice(..),
            );


            render_pass.draw(
                0..2,
                0..bucket.instances.len() as u32,
            );
        }

        self.clear();
    }


    pub fn clear(&mut self) {
        self.points.clear();
        self.point_lookup.clear();

        for bucket in self.buckets.iter_mut() {
            bucket.instances.clear();
        }
    }
}