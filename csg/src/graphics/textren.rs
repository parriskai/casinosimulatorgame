use std::{collections::HashMap, io::Cursor, sync::Arc};

use serde::Deserialize;

use crate::graphics::{UvBox, asset_mgr::GPUTexture};

#[derive(Debug, Deserialize)]
struct AtlasGlyph{
    char: char,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    glyph_width: usize,
    glyph_height: usize,
    advance: f32,
    offset_x: usize,
    offset_y: usize,

    #[serde(flatten)]
    uvbox: UvBox
}

#[derive(Debug, Deserialize)]
struct AtlasJSON{
    font: String,
    font_size: u32,
    atlas_width: u32,
    atlas_height: u32,
    glyphs: Vec<AtlasGlyph>
}

pub struct FontTextureAtlas{
    json: AtlasJSON,
    image: image::GrayAlphaImage
} impl FontTextureAtlas{
    pub fn from_included(atlas: &'static [u8], json: &'static str) -> FontTextureAtlas{
        let json: AtlasJSON = serde_json::from_str(json).unwrap();
        let image: image::GrayAlphaImage = image::ImageReader::new(Cursor::new(atlas)).with_guessed_format().unwrap().decode().unwrap().into();

        assert!(image.width() == json.atlas_width);
        assert!(image.height() == json.atlas_height);
        FontTextureAtlas {
            json,
            image
        }
    }
}

pub struct Font{
    atlas: Arc<GPUTexture>,
    mapping: HashMap<char, AtlasGlyph>,
    missing: AtlasGlyph
} impl Font{
    fn create(json: AtlasJSON){
        
    }
}