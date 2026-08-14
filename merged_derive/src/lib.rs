use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::{env, fs, path::PathBuf};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Data, DeriveInput, Fields, Ident, Token, Type,
};

fn registry_dir() -> PathBuf {
    let manifest = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let workspace_root = PathBuf::from(&manifest);
    let dir = workspace_root.join("target").join("__merged_registry");
    fs::create_dir_all(&dir).ok();
    dir
}

fn registry_path(struct_name: &str) -> PathBuf {
    println!(
        "{:?}",
        registry_dir().join(format!("{}.fields", struct_name))
    );
    registry_dir().join(format!("{}.fields", struct_name))
}

fn wrap_in_option(ty: &Type) -> proc_macro2::TokenStream {
    if let Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            if seg.ident == "Option" {
                return quote! { #ty };
            }
        }
    }
    quote! { Option<#ty> }
}

#[proc_macro_derive(MergedSource)]
pub fn merged_source_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.to_string();

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => f.named.clone(),
            _ => panic!("MergedSource only supports structs with named fields"),
        },
        _ => panic!("MergedSource only supports structs"),
    };

    let lines: Vec<String> = fields
        .iter()
        .map(|f| {
            let fname = f.ident.as_ref().unwrap().to_string();
            let ty = &f.ty;
            let ty_str = quote! { #ty }.to_string();
            format!("{}|{}", fname, ty_str)
        })
        .collect();

    fs::write(registry_path(&name), lines.join("\n")).expect("failed to write field registry");

    TokenStream::new()
}

struct MergedArgs {
    sources: Vec<Ident>,
}

impl Parse for MergedArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let sources = input
            .parse_terminated(Ident::parse, Token![,])?
            .into_iter()
            .collect();
        Ok(MergedArgs { sources })
    }
}

#[proc_macro_attribute]
pub fn merged(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as MergedArgs);
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let extra_fields: Vec<proc_macro2::TokenStream> = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => f
                .named
                .iter()
                .map(|field| {
                    let fname = &field.ident;
                    let fty = wrap_in_option(&field.ty);
                    let attrs = &field.attrs;
                    quote! { #(#attrs)* pub #fname: #fty, }
                })
                .collect(),
            Fields::Unit => vec![],
            _ => panic!("merged only supports structs with named fields"),
        },
        _ => panic!("merged only supports structs"),
    };

    let mut injected_fields: Vec<proc_macro2::TokenStream> = vec![];

    for src in &args.sources {
        let path = registry_path(&src.to_string());
        let content = fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!(
                "Fields of '{}' not found in registry. \
                 Make sure #[derive(MergedSource)] is present on the source struct.",
                src
            )
        });

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts = line.split_once('|').unwrap();
            let fname_str = parts.0;
            let ftype_str = parts.1;

            let field_ident = Ident::new(fname_str, Span::call_site());
            let ftype: Type = syn::parse_str(ftype_str)
                .unwrap_or_else(|_| panic!("unparseable type: '{}'", ftype_str));
            let option_ty = wrap_in_option(&ftype);

            injected_fields.push(quote! {
                pub #field_ident: #option_ty,
            });
        }
    }

    quote! {
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        pub struct #name {
            #(#injected_fields)*
            #(#extra_fields)*
        }
    }
    .into()
}
