use crate::graphics::{RenderLayer, asset_mgr::TextureKey, atlasrender::AtlasRenderer};
use nalgebra::Vector2;

#[macro_export]
macro_rules! build_sprite_state_enum {
    (
        $p:vis enum $name:ident {
            $(
                $v:ident
            ),+ $(,)?
        } -> $json:ident
    ) => {
        $p enum $name {
            $($v),+
        }

        ::paste::paste! {
            #[derive(Debug, ::serde::Deserialize)]
            #[allow(nonstandard_style)]
            struct [<_ $json Inner>] {
                $(
                    $v: Option<$crate::graphics::UvBox>
                ),+
            }

            #[derive(Debug, ::serde::Deserialize)]
            $p struct $json {
                size: [f32; 2],
                sprites: [<_ $json Inner>],
            } impl $json {
                $p fn from_include(s: &str) -> $json{
                    ::serde_json::from_str(s).unwrap()
                }
            }

            impl $crate::graphics::sprite::SpriteJSON for $json {
                type ENUM = $name;

                fn size(&self) -> ::nalgebra::Vector2<f32> {
                    ::nalgebra::Vector2::new(
                        self.size[0],
                        self.size[1],
                    )
                }

                fn render(
                    &self,
                    ar: &mut $crate::graphics::atlasrender::AtlasRenderer,
                    tk: $crate::graphics::asset_mgr::TextureKey,
                    state: &Self::ENUM,
                    pos: (
                        ::nalgebra::Vector2<f32>,
                        ::nalgebra::Vector2<f32>,
                        $crate::graphics::RenderLayer,
                    ),
                ) {
                    match state {
                        $(
                            $name::$v => {
                                match self.sprites.$v {
                                    Some(uv) => {
                                        ar.draw_atlas(
                                            tk,
                                            uv,
                                            pos,
                                        );
                                    }
                                    None => {
                                        ar.draw_atlas(
                                            tk,
                                            $crate::graphics::UvBox::FULL,
                                            pos,
                                        );
                                    }
                                }
                            }
                        ),+
                    }
                }
            }
        }
    };
}
pub trait SpriteJSON {
    type ENUM;

    fn size(&self) -> Vector2<f32>;
    fn render(&self, ar: &mut AtlasRenderer, tk: TextureKey, state: &Self::ENUM, pos: (Vector2<f32>, Vector2<f32>, RenderLayer));
}

pub struct Sprite<S: SpriteJSON>{
    pub tk: TextureKey,
    pub json: S
} impl<S: SpriteJSON> Sprite<S>{
    pub fn create(tk: TextureKey, json: S) -> Sprite<S>{
        Sprite {
            tk,
            json
        }
    }

    pub fn render(&self, ar: &mut AtlasRenderer, state: &S::ENUM, pos: (Vector2<f32>, Vector2<f32>, RenderLayer)){
        self.json.render(ar, self.tk, state, pos);
    }
}

pub struct SpriteInstance<S: SpriteJSON>{
    pub scale: f32,
    size: Vector2<f32>,
    pub pos: Vector2<f32>,
    pub layer: RenderLayer,
    pub state: S::ENUM
} impl<S: SpriteJSON> SpriteInstance<S>  {
    pub fn create(sprite: &Sprite<S>, scale: f32, pos: Vector2<f32>, layer: RenderLayer, state: S::ENUM) -> SpriteInstance<S>{
        SpriteInstance {
            scale,
            size: sprite.json.size(),
            pos,
            layer,
            state
        }
    }

    pub fn render(&self, sp: &Sprite<S>, ar: &mut AtlasRenderer){
        sp.render(ar, &self.state, (self.pos, self.pos + self.size * self.scale, self.layer));
    }
}