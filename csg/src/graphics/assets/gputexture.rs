use std::sync::LazyLock;

use image::DynamicImage;
use wgpu::BindGroup;

use crate::{errors::{GResult, GeneralizeError}, graphics::{assets::{ReloadableAsset, image::load_image}, graphicscontrol::GraphicsControl}};

pub struct GpuTexture {
    gc: GraphicsControl,
    name: String,

    texture: wgpu::Texture,
    view: wgpu::TextureView,

    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
} impl GpuTexture{
    pub fn from_di(gc: GraphicsControl, image: DynamicImage, name: String) -> GpuTexture{
        let image = image.to_rgba8();
        let (width, height) = image.dimensions();

        let texture = gc.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&name),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        gc.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = gc.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(format!("{name} Sampler").as_str()),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = gc.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(format!("{name} Bind Group Layout").as_str()),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = gc.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(format!("{name} Bind Group").as_str()),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        GpuTexture {
            gc,
            name,

            texture,
            view,

            sampler,
            bind_group,
            bind_group_layout
        }
    }

    pub fn reload(&mut self, image: DynamicImage) {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();

        let current_size = self.texture.size();

        if current_size.width == width && current_size.height == height {
            // Same dimensions, so we can just replace the texture contents.
            self.gc.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );

            return;
        }

        // Size changed, so the GPU texture itself has to be recreated.
        let texture = self.gc.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&self.name),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.gc.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = self.gc.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(format!("{} Bind Group", self.name).as_str()),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        self.texture = texture;
        self.view = view;
        self.bind_group = bind_group;
    }

    pub fn bind_group(&self) -> &BindGroup{
        &self.bind_group
    }
}

pub fn load_texture(gc: GraphicsControl, name: String) -> impl FnOnce(&[u8]) -> GResult<GpuTexture>{
    |data| Ok(GpuTexture::from_di(gc, load_image(data)?, name))
}

impl ReloadableAsset for GpuTexture{
    fn reload(&mut self, data: &[u8]) -> crate::prelude::GResult<()> {
        self.reload(load_image(data)?);
        Ok(())
    }
}