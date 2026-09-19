use crate::{graphics::{RenderLayer, UvBox, asset_mgr::TextureKey, atlasrender::AtlasRenderer}};
use std::{collections::HashMap, io::Cursor, sync::Arc};
use serde::Deserialize;
use nalgebra::Vector2;

#[derive(Debug, Clone, Deserialize)]
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
    pub uvbox: UvBox,

    pub kerning: HashMap<char, f32>
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
    missing: AtlasGlyph,
    size: f32
} impl Font{
    pub fn create(json: AtlasJSON, atlas: TextureKey) -> Font{
        let mut mapping = HashMap::with_capacity(json.glyphs.len());
        
        for glyph in json.glyphs{
            mapping.insert(glyph.char, glyph);
        }

        let missing = if let Some(question_mark) = mapping.get(&'?'){
            question_mark.clone()
        } else if !mapping.is_empty(){
            // SAFTEY: We just checked
            unsafe{mapping.iter().next().unwrap_unchecked()}.1.clone()
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
                uvbox: UvBox { u0: 0., v0: 0., u1: size as f32, v1: size as f32 },
                kerning: HashMap::new()
            }
        };

        Font{
            atlas,
            mapping,
            missing,
            size: json.font_size as f32
        }
    }

    pub fn write_text(&self, text: &str, mut pos: Vector2<f32>, scale: f32, layer: RenderLayer, ar: &mut AtlasRenderer){
        let mut preveious: Option<&AtlasGlyph> = None;
        for chr in text.chars(){
            let glyph = self.mapping.get(&chr).unwrap_or(&self.missing);
            let kerning = if let Some(prev) = preveious{prev.kerning.get(&chr).cloned().unwrap_or(0.0) * 1.5} else {0.0};

            let topleft = pos + Vector2::new(glyph.offset_x as f32 + kerning, glyph.offset_y as f32) * scale;
            
            ar.draw_atlas(self.atlas, glyph.uvbox, (topleft, topleft + Vector2::new(glyph.width as f32, glyph.height as f32) * scale, layer));
            pos.x += (glyph.advance * 1.15 + kerning) * scale;
            preveious = Some(glyph);
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