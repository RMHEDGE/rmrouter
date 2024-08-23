use proc_macro::TokenStream;
use syn::{
    parse::{Parse, ParseStream}, parse_macro_input, Block, Generics, Ident, Token, Type, Visibility, WhereClause
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