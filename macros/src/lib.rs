use proc_macro::TokenStream;
use syn::parse_macro_input;

mod meta;

#[proc_macro_attribute]
pub fn worked(attr: TokenStream, item: TokenStream) -> TokenStream {
    let meta = parse_macro_input!(item as meta::Meta);

    let attr = parse_macro_input!(attr as syn::LitStr);
    let path = attr.value();

    let (vis, name, generics, arg, arg_name, ret, clause, block) =
        (meta.0, meta.1, meta.2, meta.3, meta.4, meta.5, meta.6, meta.7);

    let struct_name = quote::format_ident!("Endpoint{}{}", name.to_string().split_at(1).0.to_uppercase(), name.to_string().split_at(1).1);
    let quiet_name_string = struct_name.to_string();

    quote::quote! {
        pub fn #struct_name #generics(i__: Vec<u8>) -> js_sys::Array #clause {
            let #arg_name: #arg = bincode::decode_from_slice(&i__, bincode::config::standard()).unwrap().0;
            let res = #block;
            
            let array = js_sys::Array::new();
            for item in bincode::encode_to_vec(&res, bincode::config::standard()).unwrap() {
                array.push(&JsValue::from(item));
            }

            array
        }

        #vis async fn #name #generics(i: #arg, c: impl Fn(#ret) + 'static) #clause  {
            let w = WrappedWorker::<#arg, #ret>::new(#path).await;
            w.run_task(#quiet_name_string, i, c);
        }
    }
    .into()
}