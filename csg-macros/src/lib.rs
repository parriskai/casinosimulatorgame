use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::ParseStream,
    parse_macro_input,
    Error, Ident, LitStr, Result, Token, Visibility,
};

struct SpriteStateInput {
    vis: Visibility,
    _enum_token: Token![enum],
    name: Ident,
    variants: Vec<SpriteVariant>,
    _arrow: Token![->],
    json_name: Ident,
}

struct SpriteVariant {
    name: Ident,
    json_name: Option<LitStr>,
}

impl syn::parse::Parse for SpriteStateInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let vis = input.parse()?;
        let enum_token = input.parse()?;
        let name = input.parse()?;

        let content;
        syn::braced!(content in input);

        let mut variants = Vec::new();

        while !content.is_empty() {
            let variant_name: Ident = content.parse()?;

            let json_name = if content.peek(Token![=]) {
                content.parse::<Token![=]>()?;
                Some(content.parse::<LitStr>()?)
            } else {
                None
            };

            variants.push(SpriteVariant {
                name: variant_name,
                json_name,
            });

            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            } else if !content.is_empty() {
                return Err(content.error("expected `,` between variants"));
            }
        }

        let arrow = input.parse()?;
        let json_name = input.parse()?;

        Ok(Self {
            vis,
            _enum_token: enum_token,
            name,
            variants,
            _arrow: arrow,
            json_name,
        })
    }
}

#[proc_macro]
pub fn build_sprite_state_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SpriteStateInput);

    match expand(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn expand(input: SpriteStateInput) -> Result<proc_macro2::TokenStream> {
    let SpriteStateInput {
        vis,
        _enum_token: _,
        name,
        variants,
        _arrow: _,
        json_name,
    } = input;

    if variants.is_empty() {
        return Err(Error::new(
            name.span(),
            "sprite state enum must contain at least one variant",
        ));
    }

    let inner_name = format_ident!("{}_inner", json_name);

    let variant_names: Vec<_> =
        variants.iter().map(|v| &v.name).collect();

    let fields = variants.iter().map(|variant| {
        let name = &variant.name;

        match &variant.json_name {
            Some(alias) => {
                quote! {
                    #[serde(rename = #alias)]
                    #name: Option<$crate::graphics::UvBox>
                }
            }
            None => {
                quote! {
                    #name: Option<$crate::graphics::UvBox>
                }
            }
        }
    });

    let render_arms = variants.iter().map(|variant| {
        let variant_name = &variant.name;

        quote! {
            #name::#variant_name => {
                match self.sprites.#variant_name {
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
        }
    });

    Ok(quote! {
        #vis enum #name {
            #(#variant_names),*
        }

        #[derive(Debug, ::serde::Deserialize)]
        struct #inner_name {
            #(#fields),*
        }

        #[derive(Debug, ::serde::Deserialize)]
        #vis struct #json_name {
            size: [f32; 2],
            sprites: #inner_name,
        }

        impl $crate::graphics::sprite::SpriteJSON for #json_name {
            type ENUM = #name;

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
                    #(#render_arms),*
                }
            }
        }
    })
}