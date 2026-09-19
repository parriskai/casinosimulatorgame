use std::{collections::HashMap, io::Cursor, sync::Arc};

use image::DynamicImage;
use slotmap::{Key, SlotMap};
use wgpu::BindGroup;

use crate::graphics::{graphicscontrol::GraphicsControl, textren::{AtlasJSON, Font, FontOrMissing, FontTextureAtlas}};

// hardcoded
const MISSING_TEXTURE: &[u8; 120] = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x10\x00\x00\x00\x10\x08\x06\x00\x00\x00\x1f\xf3\xffa\x00\x00\x00\x01sRGB\x00\xae\xce\x1c\xe9\x00\x00\x002IDAT8\x8dc\xfc\xcf\xf0\xff?\x03\x1e\xc0\xc8\xc0\x88O\x9a\x81\t\xaf,\x11`\xd4\x80\xc1`\x00#\x03\x03\x03\xdet\xf0\x1f\xbf\xf4 \xf0\xc2\xa8\x01T0\x00\x001\xfa\x06\x1b\xa4}\x155\x00\x00\x00\x00IEND\xaeB`\x82";

pub struct GPUTexture{
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: BindGroup
} impl GPUTexture{
    fn from_di(gc: &GraphicsControl, image: DynamicImage, name: &str) -> GPUTexture{
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();

        let texture = gc.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
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

        let sampler = gc.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(format!("{name} View").as_str()),
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

        GPUTexture {
            texture,
            view,
            bind_group
        }
    }

    fn from_bytes(gc: &GraphicsControl, image: &[u8], name: &str) -> GPUTexture{
        let di = image::ImageReader::new(Cursor::new(image)).with_guessed_format().unwrap().decode().unwrap();
        Self::from_di(gc, di, name)
    }

    pub fn bind_group(&self) -> &BindGroup{
        &self.bind_group
    }
}

pub enum TextureOrMissing{
    Texture(Arc<GPUTexture>),
    Missing(Arc<GPUTexture>)
} impl TextureOrMissing{
    pub fn inner(self) -> Arc<GPUTexture>{
        match self {
            TextureOrMissing::Texture(t) => t,
            TextureOrMissing::Missing(t) => t
        }
    }
}

slotmap::new_key_type!{
    pub struct TextureKey;
    pub struct FontKey;
    pub struct FlipbookKey;
}

pub struct AssetManager{
    gc: GraphicsControl,

    textures: SlotMap<TextureKey, Arc<GPUTexture>>,
    textures_by_name: HashMap<String, TextureKey>,
    missing_texture: Arc<GPUTexture>,

    fonts: SlotMap<FontKey, Arc<Font>>,
    fonts_by_name: HashMap<String, FontKey>,
    missing_font: Arc<Font>,
} impl AssetManager{
    pub fn create(gc: GraphicsControl) -> AssetManager{
        let missing_texture = Arc::new(GPUTexture::from_bytes(&gc, MISSING_TEXTURE, "MISSING_TEXTURE"));
        let missing_font = Arc::new(Font::create(AtlasJSON{font: "MISSING_FONT".into(), font_size: 32, atlas_width: 32, atlas_height: 32, glyphs: Vec::new()}, TextureKey::null()));
        
        AssetManager {
            gc,

            textures: SlotMap::with_key(),
            textures_by_name: HashMap::new(),
            missing_texture,

            fonts: SlotMap::with_key(),
            fonts_by_name: HashMap::new(),
            missing_font
        }
    }
    
    pub fn create_texture(&mut self, image: DynamicImage, name: String) -> TextureKey{
        let id = self.textures.insert(Arc::new(GPUTexture::from_di(&self.gc, image, &name)));
        self.textures_by_name.insert(name, id);
        id
    }

    pub fn create_texture_from_bytes(&mut self, image: &[u8], name: String) -> TextureKey{
        let id = self.textures.insert(Arc::new(GPUTexture::from_bytes(&self.gc, image, &name)));
        self.textures_by_name.insert(name, id);
        id
    }

    pub fn texture_or_create<F: FnOnce() -> DynamicImage>(&mut self, name: &str, create: Option<F>) -> Option<(TextureKey, Arc<GPUTexture>)>{
        if let Some(key) = self.textures_by_name.get(name){
            // SAFTEY: textures and textures_by_name should always match
            Some((key.clone(), unsafe{self.textures.get_unchecked(*key).to_owned()}))
        } else {
            let di = create?();
            let key = self.create_texture(di, name.into());
            Some((key.clone(), unsafe{self.textures.get_unchecked(key).to_owned()}))
        }
    }

    pub fn texture_by_name(&self, name: &str) -> TextureOrMissing{
        self.textures_by_name.get(name).map_or_else(
            || {TextureOrMissing::Missing(self.missing_texture.to_owned())},
            // SAFTEY: textures and textures_by_name should always match
            |v| {TextureOrMissing::Texture(unsafe{self.textures.get_unchecked(v.to_owned())}.to_owned())}
        )
    }

    pub fn texture_by_id(&self, key: TextureKey) -> TextureOrMissing{
        self.textures.get(key).map_or_else(
            || {TextureOrMissing::Missing(self.missing_texture.to_owned())},
            |v| {TextureOrMissing::Texture(v.to_owned())}
        )
    }

    pub fn create_font(&mut self, font: FontTextureAtlas, name: String) -> FontKey{
        let atlas = self.create_texture(font.image, format!("ATLAS[{name}]"));

        let id = self.fonts.insert(Arc::new(Font::create(font.json, atlas)));
        self.fonts_by_name.insert(name, id);
        id
    }

    pub fn font_or_create<F: FnOnce() -> FontTextureAtlas>(&mut self, name: &str, create: Option<F>) -> Option<(FontKey, Arc<Font>)>{
        if let Some(key) = self.fonts_by_name.get(name){
            // SAFTEY: fonts and fonts_by_name should always match
            Some((key.clone(), unsafe{self.fonts.get_unchecked(*key).to_owned()}))
        } else {
            let di = create?();
            let key = self.create_font(di, name.into());
            Some((key.clone(), unsafe{self.fonts.get_unchecked(key).to_owned()}))
        }
    }

    pub fn font_by_name(&self, name: &str) -> FontOrMissing{
        self.fonts_by_name.get(name).map_or_else(
            || {FontOrMissing::Missing(self.missing_font.to_owned())},
            // SAFTEY: fonts and fonts_by_name should always match
            |v| {FontOrMissing::Font(unsafe{self.fonts.get_unchecked(v.to_owned())}.to_owned())}
        )
    }

    pub fn font_by_id(&self, key: FontKey) -> FontOrMissing{
        self.fonts.get(key).map_or_else(
            || {FontOrMissing::Missing(self.missing_font.to_owned())},
            |v| {FontOrMissing::Font(v.to_owned())}
        )
    }
}