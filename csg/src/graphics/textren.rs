use std::{collections::HashMap, io::Cursor, sync::Arc};

use serde::Deserialize;

use crate::{graphics::{UvBox, asset_mgr::{GPUTexture, TextureKey}, atlasrender::AtlasRenderer}, utils::{IntoGPUMatrix, Transform}};

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct AtlasGlyph{
    pub char: char,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub glyph_width: usize,
    pub glyph_height: usize,
    pub advance: f32,
    pub offset_x: i32,
    pub offset_y: i32,

    #[serde(flatten)]
    pub uvbox: UvBox
}

#[derive(Debug, Deserialize)]
pub struct AtlasJSON{
    pub font: String,
    pub font_size: u32,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub glyphs: Vec<AtlasGlyph>
}

pub struct FontTextureAtlas{
    pub json: AtlasJSON,
    pub image: image::DynamicImage
} impl FontTextureAtlas{
    pub fn from_included(atlas: &'static [u8], json: &'static str) -> FontTextureAtlas{
        let json: AtlasJSON = serde_json::from_str(json).unwrap();
        let image = image::ImageReader::new(Cursor::new(atlas)).with_guessed_format().unwrap().decode().unwrap();

        assert!(image.width() == json.atlas_width);
        assert!(image.height() == json.atlas_height);
        FontTextureAtlas {
            json,
            image
        }
    }
}

pub struct Font{
    atlas: TextureKey,
    mapping: HashMap<char, AtlasGlyph>,
    missing: AtlasGlyph
} impl Font{
    pub fn create(json: AtlasJSON, atlas: TextureKey) -> Font{
        let mut mapping = HashMap::with_capacity(json.glyphs.len());
        
        for glyph in json.glyphs{
            mapping.insert(glyph.char, glyph);
        }

        let missing = if let Some(question_mark) = mapping.get(&'?'){
            *question_mark
        } else if !mapping.is_empty(){
            // SAFTEY: We just checked
            *unsafe{mapping.iter().next().unwrap_unchecked()}.1
        } else {
            let size = json.font_size as usize;
            AtlasGlyph{
                char: '\0',
                x: 0,
                y: 0,
                width: size,
                height: size,
                glyph_width: size,
                glyph_height: size,
                advance: size as f32,
                offset_x: 0,
                offset_y: 0,
                uvbox: UvBox { u0: 0., v0: 0., u1: size as f32, v1: size as f32 }}
        };

        Font{
            atlas,
            mapping,
            missing
        }
    }

    pub fn write_text(&self, text: &str, transform: Transform, ar: &mut AtlasRenderer){
        let mut offset = Transform::Identity;
        for chr in text.chars(){
            let glyph = self.mapping.get(&chr).unwrap_or(&self.missing);
            ar.draw_atlas(self.atlas, glyph.uvbox, offset.then(Transform::Translate(-glyph.offset_x as f32 / 24., -glyph.offset_y as f32 / 24., 0.)).then(transform));
            offset = offset.then(Transform::Translate(glyph.advance / 24., 0., 0.))
        }
    }
}

pub enum FontOrMissing {
    Font(Arc<Font>),
    Missing(Arc<Font>)
} impl FontOrMissing{
    pub fn inner(self) -> Arc<Font>{
        match self{
            FontOrMissing::Font(f) => f,
            FontOrMissing::Missing(m) => m
        }
    }
}