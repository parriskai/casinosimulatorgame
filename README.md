# WCU 26' Casino Simulator Game
## Summary

**TBD**

## Known bugs

- Performance
  - [x] Window lags when resizing (FIXED: Relpaced GraphicsControl::dim with an Arc<(AtomicU32, AtomicU32)> which should make reads faster)
- Rendering
  - Atlas renderer
    - [x] Atlas rendering doesnt include depth so randomly the two images swap. Run it again until it works (FIXED: Added depth test)
    - [x] Atlas rendering doesnt sort by layer causing alpha mixing problems (FIXED: Sorted by layer)
  - Font renderer
    - [x] Letters wrong size (FIXED: change variable, add correct scalling)
   
## Graphics API
**Loading a texture:** \
`ren.asset_manager.create_texture_from_butes(include_bytes!(..), "name".into()) -> TextureKey`

**Look up a texture by its name:** \
`ren.asset_manager.texture_by_name("name")` -> `TextureOrMissing`

**Load a font:** \
`ren.asset_manager.create_font(FontTextureAtlas::from_included(png, json), "name".into()) -> FontKey`

**Get font handle:** \
`ren.asset_manager.font_by_id(key)inner() -> Arc<Font>`

**Draw a quad:** \
`ren.atlas_renderer.draw_atlas(texKey, uv, (top_l, bottom_r, layer))`

**animate:** \
`Flipbook::create(tkey, FlipbookJSON::from_include(..))` and `.resume(t)` `.update(t)`, `.draw(pos, scale, layer, &mut ar)` 

**text:** \
`font.write_text("text", pos, scale, layer, &mut ar)`

## Authors

- Kai Parris (Programmer)
- Charles Rothbaum (Programmer)
- Evan Wright (Composer)
- Adrian (Artist)
