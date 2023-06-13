extern crate proc_macro;

use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, LitStr};

#[derive(Debug, Default)]
struct TokenOptions {
    lexeme: String,
}

fn has_prefix_lookup_derive_inner(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;

    let variants = match &ast.data {
        Data::Enum(v) => &v.variants,
        _ => {
            return Err(syn::Error::new(
                Span::call_site(),
                "derive(HasPrefixLookup) only supports enums",
            ))
        }
    };

    // maps from identifiers (e.g. "Plus") to token options (e.g. lexeme: "+")
    let mut member_table = HashMap::<String, TokenOptions>::new();

    for variant in variants {
        for attr in &variant.attrs {
            if attr.path().is_ident("token") {
                let mut token_options = TokenOptions::default();
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("lexeme") {
                        let value = meta.value()?;
                        let str_value: LitStr = value.parse()?;
                        token_options.lexeme = str_value.value();
                        return Ok(());
                    }

                    let arg_name = meta
                        .path
                        .get_ident()
                        .expect("meta.path.ident unpexpectedly None")
                        .to_string();

                    Err(meta.error(format!(
                        "Unexpected expr in token() macro helper: {}",
                        arg_name
                    )))
                })?;

                member_table.insert(variant.ident.to_string(), token_options);
            }
        }
    }

    let mut prefixes_to_vecs: HashMap<&str, Vec<&str>> = HashMap::new();

    // For each character in each lexeme, iterate over all lexemes in the enum,
    // and generate a list of other lexemes that could start with that prefix.
    for value in member_table.values() {
        let lexeme = &value.lexeme;

        for i in 0..lexeme.len() {
            let mut lexemes_with_prefix: Vec<&str> = Vec::new();
            let prefix = &lexeme[0..i + 1];
            for other_value in member_table.values() {
                if other_value.lexeme.starts_with(prefix) {
                    lexemes_with_prefix.push(&other_value.lexeme);
                }
            }

            if !lexemes_with_prefix.is_empty() {
                prefixes_to_vecs.insert(prefix, lexemes_with_prefix);
            }
        }
    }

    let mut phf_map_arms: Vec<TokenStream> = Vec::new();

    for (key, value) in prefixes_to_vecs.iter() {
        let arm = quote! {
            #key => &[#(#value),*]
        };
        phf_map_arms.push(arm);
    }

    Ok(quote! {

        impl HasPrefixLookup for #name {
            fn fields_starting_with(ident: &str) -> usize {
                use phf::phf_map;
                static PHF: phf::Map<&'static str, &[&str]> = phf_map! {
                    #(#phf_map_arms),*
                };

                if let Some(matches) = PHF.get(ident) {
                    return matches.len();
                }

                0
            }
        }
    })
}

#[proc_macro_derive(HasPrefixLookup, attributes(token))]
pub fn has_prefix_lookup_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    let tokens = has_prefix_lookup_derive_inner(&ast).unwrap_or_else(|e| e.to_compile_error());
    tokens.into()
}
