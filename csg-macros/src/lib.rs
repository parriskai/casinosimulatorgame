use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
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

    match expand_sprite_state(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn expand_sprite_state(input: SpriteStateInput) -> Result<proc_macro2::TokenStream> {
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

    let crate_path = match crate_name("csg").unwrap() {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(::#ident)
        }
    };

    let inner_name = format_ident!("{}Inner", json_name);

    let variant_names: Vec<_> =
        variants.iter().map(|v| &v.name).collect();

    let fields = variants.iter().map(|variant| {
        let variant_name_string = &variant.name.to_string();
        let snake_variant_name = Ident::new(variant.name.to_string().as_str().to_snake_case().as_str(), variant.name.span());
        match &variant.json_name {
            Some(alias) => {
                quote! {
                    #[serde(rename = #alias)]
                    #snake_variant_name: Option<#crate_path::graphics::UvBox>
                }
            }
            None => {
                quote! {
                    #[serde(rename = #variant_name_string)]
                    #snake_variant_name: Option<#crate_path::graphics::UvBox>
                }
            }
        }
    });

    let none_checks = variants.iter().map(|variant| {
        // I dont like the clone but without a much larger solution its the only way
        let error = format!("Sprite variant {} is not defined in the json file but is defined in {} sprite!", variant.json_name.clone().map_or(variant.name.to_string(), |x| x.value()), name.to_string());
        let snake_variant_name = Ident::new(variant.name.to_string().as_str().to_snake_case().as_str(), variant.name.span());
        quote! {
            if inst.sprites.#snake_variant_name.is_none(){
                ::tracing::error!(#error);
            }
        }
    });

    let render_arms = variants.iter().map(|variant| {
        let variant_name = &variant.name;
        let snake_variant_name = Ident::new(variant.name.to_string().as_str().to_snake_case().as_str(), variant.name.span());

        quote! {
            #name::#variant_name => {
                match self.sprites.#snake_variant_name {
                    Some(uv) => {
                        ar.draw_atlas(
                            tk,
                            uv,
                            pos,
                        );
                    }
                    None => {
                        ar.draw_atlas(
                            <#crate_path::graphics::assets::manager::AssetKey<#crate_path::graphics::assets::gputexture::GpuTexture> as ::slotmap::Key>::null(),
                            #crate_path::graphics::UvBox::FULL,
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

        #[derive(Debug, Default, ::serde::Deserialize)]
        struct #inner_name {
            #(#fields),*
        }

        #[derive(Debug, Default, ::serde::Deserialize)]
        #vis struct #json_name {
            size: [f32; 2],
            sprites: #inner_name,
        } impl #json_name{
            #vis fn from_include(s: &str) -> Self {
                let inst: Self = ::serde_json::from_str(s).unwrap();
                
                #(#none_checks)*
                
                inst
            }
            #vis fn load(data: &[u8]) -> #crate_path::errors::GResult<Self>{
                Ok(
                    Self::from_include(
                        <Result<&str, std::str::Utf8Error> as #crate_path::errors::GeneralizeError<&str>>::g_err(str::from_utf8(data))?
                    )
                )
            }
        } impl #crate_path::graphics::assets::ReloadableAsset for #json_name{
            fn reload(&mut self, data: &[u8]) -> #crate_path::errors::GResult<()> {
                match #json_name::load(data){
                    Ok(data) => {
                        std::mem::replace(self, data);
                        Ok(())
                    }
                    Err(e) => {
                        std::mem::replace(self, <#json_name as #crate_path::graphics::assets::DefaultAsset>::default());
                        Err(e)
                    }
                }
            }
        }
        impl #crate_path::graphics::assets::DefaultAsset for #json_name{
            fn default() -> Self {
                #json_name{size: [16.,16.], .. Default::default()}
            }
        }
        inventory::submit!(#crate_path::graphics::assets::AssetDefault::create::<#json_name>());

        impl #crate_path::graphics::sprite::SpriteJSON for #json_name {
            type ENUM = #name;

            fn size(&self) -> ::nalgebra::Vector2<f32> {
                ::nalgebra::Vector2::new(
                    self.size[0],
                    self.size[1],
                )
            }

            fn render(
                &self,
                ar: &mut #crate_path::graphics::atlasrender::AtlasRenderer,
                tk: #crate_path::graphics::assets::manager::AssetKey<#crate_path::graphics::assets::gputexture::GpuTexture>,
                state: &Self::ENUM,
                pos: (
                    ::nalgebra::Vector2<f32>,
                    ::nalgebra::Vector2<f32>,
                    #crate_path::graphics::RenderLayer,
                ),
            ) {
                match state {
                    #(#render_arms),*
                }
            }
        }
    })
}