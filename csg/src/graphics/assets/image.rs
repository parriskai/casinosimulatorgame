use crate::{
    graphics::assets::{
        AssetDefault, 
        DefaultAsset, 
        ReloadableAsset
    },
    prelude::*,
};
use image::DynamicImage;
use std::io::Cursor;

pub fn load_image(data: &[u8]) -> GResult<DynamicImage>{
    Ok(
        image::ImageReader::new(Cursor::new(data))
            .with_guessed_format().g_err()?
            .decode().g_err()?
    )
} impl ReloadableAsset for DynamicImage{
    fn reload(&mut self, data: &[u8]) -> crate::prelude::GResult<()> {
        // Just overwrite it
        drop(std::mem::replace(self, load_image(data)?));
        Ok(())
    }
}

// hardcoded
const MISSING_IMAGE_BYTES: &[u8; 120] = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x10\x00\x00\x00\x10\x08\x06\x00\x00\x00\x1f\xf3\xffa\x00\x00\x00\x01sRGB\x00\xae\xce\x1c\xe9\x00\x00\x002IDAT8\x8dc\xfc\xcf\xf0\xff?\x03\x1e\xc0\xc8\xc0\x88O\x9a\x81\t\xaf,\x11`\xd4\x80\xc1`\x00#\x03\x03\x03\xdet\xf0\x1f\xbf\xf4 \xf0\xc2\xa8\x01T0\x00\x001\xfa\x06\x1b\xa4}\x155\x00\x00\x00\x00IEND\xaeB`\x82";
impl DefaultAsset for DynamicImage{
    fn default() -> Self {
        load_image(MISSING_IMAGE_BYTES).unwrap()
    }
}
inventory::submit!(AssetDefault::create::<DynamicImage>());