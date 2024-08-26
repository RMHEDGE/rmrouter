use helpers::{get_inner_type, preamble, RouteInfo};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod helpers;

#[proc_macro_attribute]
pub fn endpoint(annot: TokenStream, item: TokenStream) -> TokenStream {
    let it = item.clone();
    let meta = parse_macro_input!(it as helpers::Meta);
    
    let (_, name, _, arg, _, ret, _, _) =
    (meta.0, meta.1, meta.2, meta.3, meta.4, meta.5, meta.6, meta.7);
    
    let info = RouteInfo::parse(annot.into(), name.to_string()).unwrap();
    let (path, idempotent) = (info.path, info.is_idempotent);

    let method = match idempotent {
        true => "PUT",
        false => "POST"
    };

    let ret = get_inner_type(ret);
    let struct_name = quote::format_ident!("Endpoint{}{}", name.to_string().split_at(1).0.to_uppercase(), name.to_string().split_at(1).1);

    let base: proc_macro2::TokenStream = item.into();
    quote::quote! {
        #[doc = concat!("Endpoint Struct for [", stringify!(#name) ,"]\n@ ", stringify!(#method), " /", stringify!(#path), " -> ", stringify!(#struct_name), "::Data ([", stringify!(#ret), "])")]
        pub struct #struct_name;

        impl Endpoint for #struct_name {
            type Data = #arg;
            type Returns = #ret;

            fn path() -> String { #path.to_string() }
            fn is_idempotent() -> bool { #idempotent }

            fn handler() -> AsyncPtr<Self::Data, anyhow::Result<Self::Returns>> {
                AsyncPtr::<Self::Data, anyhow::Result<Self::Returns>>::new(#name)
            }
        }

        #[doc(r"Endpoint Handler for [#name]\n@ #method /#name -> #struct_name::Data ([#arg])")]
        #base
        
    }
    .into()

}


#[proc_macro_derive(Router)]
pub fn router(item: TokenStream) -> TokenStream {
    let (_, name, data) = preamble(parse_macro_input!(item as DeriveInput));
    
    let paths: Vec<proc_macro2::TokenStream> = data.variants.iter().map(|variant| {
        let inner = &variant.fields.iter().next().expect(&format!("No endpoint specified for {}", variant.ident)).ty;
        let inner_name = &variant.ident;
        
        quote::quote! {
            (path, i) if i == #inner::is_idempotent() && path == #inner::path() => ({
                let bytes = req.collect().await.expect(&format!("Failed to read incoming bytes for {}", stringify!(#inner_name))).to_bytes();
                let body: <#inner as Endpoint>::Data = serde_json::from_str(&String::from_utf8_lossy(&bytes[..]).to_string()).expect(&format!("Failed to deserialize body for {}", stringify!(#inner_name)));
                match #inner::handler().run(body).await {
                    Ok(response) => {
                        let bytes = serde_json::to_string(&response).expect(&format!("Failed to serialize response for {}", stringify!(#inner_name)));
                        return hyper::Response::builder()
                            .status(200)
                            .body(http_body_util::Full::new(bytes::Bytes::from(bytes)))
                            .unwrap()
                    },
                    Err(e) => 
                        return hyper::Response::builder()
                            .status(400)
                            .body(http_body_util::Full::new(bytes::Bytes::from(e.to_string())))
                            .unwrap()
                };
            })
        }

    }).collect::<Vec<_>>();

    TokenStream::from(quote::quote! {

        impl #name {
            pub async fn route(req: hyper::Request<hyper::body::Incoming>) -> std::result::Result<hyper::Response<http_body_util::Full<bytes::Bytes>>, std::convert::Infallible> {
                use http_body_util::BodyExt;
                use std::error::Error;

                let path = req.uri().path().to_string();

                Ok(match tokio::task::spawn(async move {
                    match (path, req.method().is_idempotent()) {
                        #(#paths)*,
                        _ => return hyper::Response::builder()
                                .status(404)
                                .body(http_body_util::Full::default())
                                .unwrap()
                    }
                }).await {
                    Ok(inner) => inner,
                    Err(err) => {

                        let err = err.into_panic();

                        let value = err
                            .downcast_ref::<String>()
                            .cloned()
                            .or(err.downcast_ref::<&str>().map(|s| s.to_string()))
                            .unwrap_or("[Unexpected Error]".to_string());
                            
                        hyper::Response::builder()
                            .status(500)
                            .body(http_body_util::Full::new(bytes::Bytes::from(format!("{:?}", value))))
                            .unwrap()
                    }
                })
            }
        }

    })
}