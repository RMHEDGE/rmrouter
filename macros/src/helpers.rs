use std::{collections::HashMap, str::FromStr};

use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Block, Data, DataEnum, DeriveInput, Generics, Ident, Token, Type, Visibility, WhereClause,
};

pub fn unit() -> Type {
    syn::parse_str("()").unwrap()
}

pub struct Meta(
    pub Visibility,
    pub Ident,
    pub Generics,
    pub Option<Type>,
    pub Option<Ident>,
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
        let arg_name = content.parse::<Ident>().ok();
        let _ = content.parse::<Token![:]>();
        let arg = arg_name
            .is_some()
            .then(|| content.parse::<Type>().ok())
            .flatten();

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
    pub auth: proc_macro2::TokenStream,
}

impl RouteInfo {
    fn parse_groups(
        map: &mut HashMap<String, (String, proc_macro2::TokenStream)>,
        buf: &mut Vec<String>,
        tbuf: &mut proc_macro2::TokenStream,
    ) -> Result<(), syn::Error> {
        match buf.clone().len() {
            0 => {}
            1 => { map.insert(buf.first().unwrap().to_string(), (true.to_string(), tbuf.clone())); },
            2 => { map.insert(buf.first().unwrap().to_string(), (buf.last().unwrap().to_string(), tbuf.clone())); },
            _ => return Err(syn::Error::new_spanned(
                tbuf.clone(),
                "sAttributes should either have a single value (idempotent = true), or be present to indicate a value of 'true'."
            ))
        };

        buf.clear();
        *tbuf = proc_macro2::TokenStream::new();

        Ok(())
    }

    fn push_or_append(s: &str, buf: &mut Vec<String>) {
        match buf
            .last()
            .cloned()
            .unwrap_or_default()
            .ends_with(&['(', ')', ':'])
            || s.starts_with(&['(', ')', ':'])
        {
            true => buf.last_mut().unwrap().push_str(s),
            false => buf.push(s.to_string()),
        }
    }

    pub fn parse(tokens: proc_macro2::TokenStream) -> Result<Self, syn::Error> {
        let mut map = HashMap::<String, (String, proc_macro2::TokenStream)>::new();
        let mut buf = Vec::<String>::new();
        let mut tbuf = proc_macro2::TokenStream::new();

        for token in tokens.clone().into_iter() {
            tbuf.extend(token.to_token_stream());
            match token.clone() {
                proc_macro2::TokenTree::Ident(i) => {
                    RouteInfo::push_or_append(&i.to_string(), &mut buf)
                }
                proc_macro2::TokenTree::Literal(l) => RouteInfo::push_or_append(
                    l.to_string()
                        .strip_prefix("\"")
                        .map(|s| s.strip_suffix("\""))
                        .flatten()
                        .unwrap_or(&l.to_string()),
                    &mut buf,
                ),
                proc_macro2::TokenTree::Punct(p) => match p.as_char() {
                    '=' => {}
                    '(' | ')' | ':' => RouteInfo::push_or_append(&p.to_string(), &mut buf),
                    ',' => RouteInfo::parse_groups(&mut map, &mut buf, &mut tbuf)?,

                    el => {
                        return Err(syn::Error::new_spanned(
                            token,
                            &format!("Unexpected punctuation mark: {el}"),
                        ))
                    }
                },
                _ => {
                    return Err(syn::Error::new_spanned(
                        token,
                        "I have no idea what this guy is doing here",
                    ))
                }
            }
        }

        RouteInfo::parse_groups(&mut map, &mut buf, &mut tbuf)?;

        Ok(RouteInfo {
            is_idempotent: {
                let (v, t) = map
                    .get("idempotent")
                    .cloned()
                    .unwrap_or((false.to_string(), Default::default()));

                v.parse::<bool>().map_err(|_| {
                    syn::Error::new_spanned(t, "Attribute 'idempotent' must be a valid boolean")
                })?
            },

            auth: {
                map.get("auth")
                    .cloned()
                    .map(|a| proc_macro2::TokenStream::from_str(&a.0).unwrap())
                    .ok_or(syn::Error::new_spanned(tokens, "No auth handler provided"))?
            },
        })
    }
}
