use std::collections::HashMap;

use quote::format_ident;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Block, Data, DataEnum, DeriveInput, Generics, Ident, Token, Type, Visibility, WhereClause,
};

pub struct Meta(
    pub Visibility,
    pub Ident,
    pub Generics,
    pub Type,
    pub Ident,
    pub Type,
    pub Option<WhereClause>,
    pub Block,
);

impl Parse for Meta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        /*
        (vis) fn (name)<(generics)>(arg: Type) -> Type {
             (block)
        }
        */

        let vis = input.parse::<Visibility>()?;
        let _ = input.parse::<Token![async]>()?;
        let _ = input.parse::<Token![fn]>()?;
        let name = input.parse::<Ident>()?;
        let generics = input.parse::<Generics>()?;

        let content;
        let _ = syn::parenthesized!(content in input);
        let arg_name = content.parse::<Ident>()?;
        let _ = content.parse::<Token![:]>()?;
        let arg = content.parse::<Type>()?;

        let _ = input.parse::<Token![->]>()?;
        let ret = input.parse::<Type>()?;
        let clause = input.parse::<WhereClause>().ok();

        let block = input.parse::<Block>()?;

        Ok(Meta(vis, name, generics, arg, arg_name, ret, clause, block))
    }
}

pub fn preamble(input: DeriveInput) -> (DeriveInput, Ident, DataEnum) {
    let name = input.clone().ident;
    let data = match input.clone().data {
        Data::Enum(data) => data,
        _ => panic!("Router can only be implemented for enums"),
    };

    (input, name, data)
}

pub fn get_inner_type(t: Type) -> Type {
    match t {
        Type::Path(p) => {
            let ty = match p.path.segments.first().unwrap().arguments.clone() {
                syn::PathArguments::AngleBracketed(t) => t,
                _ => panic!("Unexpected path arguments"),
            }
            .args;

            let ty = match ty.first().unwrap() {
                syn::GenericArgument::Type(t) => t,
                _ => panic!("Unexpected non-type generic argument"),
            };

            ty.clone()
        }
        _ => panic!("Cant get inner type of non-path"),
    }
}

#[allow(unused_variables)]
#[derive(Debug)]
pub struct RouteInfo {
    pub is_idempotent: bool,
    pub auth: Ident,
}

impl RouteInfo {
    pub fn parse(tokens: proc_macro2::TokenStream) -> syn::Result<Self> {
        let mut map = HashMap::<String, String>::new();
        let mut buf = Vec::<String>::new();

        for token in tokens.clone().into_iter() {
            match token.clone() {
                proc_macro2::TokenTree::Ident(i) => buf.push(i.to_string()),
                proc_macro2::TokenTree::Literal(l) => buf.push(
                    l.to_string()
                        .strip_prefix("\"")
                        .map(|s| s.strip_suffix("\""))
                        .flatten()
                        .unwrap_or(&l.to_string())
                        .to_string(),
                ),
                proc_macro2::TokenTree::Punct(p) => match p.as_char() {
                    '=' => {}
                    ',' => {
                        match buf.len() {
                            1 => map.insert(buf.first().unwrap().to_string(), true.to_string()),
                            2 => map.insert(buf.first().unwrap().to_string(), buf.last().unwrap().to_string()),
                            _ => return Err(syn::Error::new(token.span(), "Incorrect groupings of attributes. Should either be one (value[= true]) or three (key = value)"))
                        };

                        buf.clear();
                    }
                    el => {
                        return Err(syn::Error::new(
                            token.span(),
                            &format!("Unexpected punctuation mark: {el}"),
                        ))
                    }
                },
                _ => {
                    return Err(syn::Error::new(
                        token.span(),
                        "I have no idea what this guy is doing here",
                    ))
                }
            }
        }

        match buf.len() {
            0 => {},
            1 => { map.insert(buf.first().unwrap().to_string(), true.to_string()); },
            2 => { map.insert(buf.first().unwrap().to_string(), buf.last().unwrap().to_string()); },
            _ => return Err(syn::Error::new(tokens.clone().into_iter().last().unwrap().span(), "Incorrect groupings of attributes. Should either be one (value[= true]) or three (key = value)"))
        };

        Ok(RouteInfo {
            is_idempotent: map
                .get("idempotent")
                .cloned()
                .unwrap_or(false.to_string())
                .parse::<bool>()
                .or(Err(syn::Error::new(
                    tokens.span(),
                    "Value 'idempotent' must be a boolean",
                )))?,
            auth: map
                .get("auth")
                .map(|a| format_ident!("{}", a))
                .expect("No auth handler provided"),
        })
    }
}
