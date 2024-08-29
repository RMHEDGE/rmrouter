use std::env;

use heck::AsSnekCase;
use helpers::{get_inner_type, preamble, RouteInfo};
use proc_macro::TokenStream;
use quote::format_ident;
use syn::{parse_macro_input, DeriveInput};

mod helpers;

#[proc_macro_attribute]
pub fn endpoint(annot: TokenStream, item: TokenStream) -> TokenStream {
    let it = item.clone();
    let meta = parse_macro_input!(it as helpers::Meta);
    
    let (_, name, _, arg, _, ret, _, _) =
    (meta.0, meta.1, meta.2, meta.3, meta.4, meta.5, meta.6, meta.7);
    
    let info = RouteInfo::parse(annot.into()).unwrap();
    let (idempotent, auth) = (info.is_idempotent, info.auth);

    let method = match idempotent {
        true => "PUT",
        false => "POST"
    };

    let ret = get_inner_type(ret);
    let struct_name = quote::format_ident!("Endpoint{}{}", name.to_string().split_at(1).0.to_uppercase(), name.to_string().split_at(1).1);

    let base: proc_macro2::TokenStream = item.into();
    quote::quote! {
        #[doc = concat!("Endpoint Struct for [", stringify!(#name) ,"]\n@ ", stringify!(#method), " -> ", stringify!(#struct_name), "::Data ([", stringify!(#ret), "])")]
        pub struct #struct_name;

        impl Endpoint for #struct_name {
            type Data = #arg;
            type Returns = #ret;

            fn is_idempotent() -> bool { #idempotent }

            fn auth() -> Box<dyn Fn(hyper::HeaderMap) -> futures::future::BoxFuture<'static, bool> + 'static + Send> {
                Box::new(move |i: hyper::HeaderMap| Box::pin(#auth(i)))
            }
            
            fn handler() -> Box<dyn Fn(Self::Data) -> futures::future::BoxFuture<'static, anyhow::Result<Self::Returns>> + 'static + Send> {
                Box::new(move |i: Self::Data| Box::pin(#name(i)))
            }
        }

        #[doc(r"Endpoint Handler for [#name]\n@ #method -> #struct_name::Data ([#arg])")]
        #base
        
    }
    .into()

}


#[proc_macro_derive(Router, attributes(path, assets))]
pub fn router(item: TokenStream) -> TokenStream {
    let (input, name, data) = preamble(parse_macro_input!(item as DeriveInput));
    
    let assets = input.attrs.iter().find(|a| a.path().is_ident("assets")).map(|v| v.parse_args::<syn::LitStr>().expect("Assets folder should be a literal string")).map(|v| format!("{}/{}", env::current_dir().map(|d| d.display().to_string()).unwrap_or_default(), v.value())).expect("No assets folder provided");
    let paths: Vec<proc_macro2::TokenStream> = data.variants.iter().map(|variant| {

        let path = format_ident!("{}", AsSnekCase(variant.ident.to_string()).to_string());
        let inner = &variant.fields.iter().next().expect(&format!("No endpoint specified for {}", variant.ident)).ty;
        let inner_name = &variant.ident;
        
        quote::quote! {
            (#path, i) if i == #inner::is_idempotent() => ({
                if !(#inner::auth())(headers).await {
                    log::debug!(concat!("[-] 401 Unauthorized /", stringify!(#path)));
                    return hyper::Response::builder()
                        .status(401)
                        .body(Body::from(format!("You aren't authorized to access this endpoint. If you believe this is a mistake, talk to your RMHedge Contact")).full())
                        .unwrap()
                }

                let bytes = req.collect().await.expect(&format!("Failed to read incoming bytes for {}", stringify!(#inner_name))).to_bytes();
                let body: <#inner as Endpoint>::Data = serde_json::from_str(&String::from_utf8_lossy(&bytes[..]).to_string()).expect(&format!("Failed to deserialize body for {}", stringify!(#inner_name)));
                match (#inner::handler())(body).await {
                    Ok(response) => {
                        let bytes = serde_json::to_string(&response).expect(&format!("Failed to serialize response for {}", stringify!(#inner_name)));
                        
                        log::debug!(concat!("[+] 200 Ok /", stringify!(#path)));
                        return hyper::Response::builder()
                        .status(200)
                        .body(Body::from(bytes).full())
                        .unwrap()
                    },
                    Err(e) => {
                        log::debug!(concat!("[-] 400 Bad Request /", stringify!(#path)));
                        return hyper::Response::builder()
                            .status(400)
                            .body(Body::from(e.to_string()).full())
                            .unwrap()
                    }
                };
            }),
        }

    }).collect::<Vec<_>>();

    TokenStream::from(quote::quote! {

        static __ASSETS: std::sync::LazyLock<std::collections::BTreeMap::<String, (String, &'static [u8])>> = std::sync::LazyLock::new(|| {
            use std::io::Read;
            let folder = std::path::PathBuf::from(#assets);
            let mut assets = std::collections::BTreeMap::<String, (String, &'static [u8])>::new();
            log::info!("[#] Collating local assets");

            if !folder.exists() {
                panic!(concat!("Invalid asset folder: ", #assets));
            }

            walkdir::WalkDir::new(folder.clone())
                .into_iter()
                .filter_map(|e| match e {
                    Err(_) => None,
                    Ok(f) => f.metadata().unwrap().is_file().then_some(f),
                })
                .for_each(|entry| {
                    let route = entry
                        .path()
                        .display()
                        .to_string()
                        .strip_prefix(&format!("{}/", folder.display().to_string()))
                        .unwrap()
                        .to_string();

                    let mut byt = Vec::new();
                    std::fs::File::open(entry.path())
                        .unwrap()
                        .read_to_end(&mut byt)
                        .unwrap();

                    let byt = Box::leak(Box::new(byt));
                    log::debug!("[#] Serving asset: {}", route);
                    assets.insert(
                        route.clone(),
                        (
                            mime_guess::from_path(route)
                                .first_or_text_plain()
                                .to_string(),
                            byt,
                        ),
                    );
                });

            assets
        });

        impl #name {
            pub async fn route(req: hyper::Request<hyper::body::Incoming>) -> std::result::Result<hyper::Response<http_body_util::Full<bytes::Bytes>>, std::convert::Infallible> {
                use http_body_util::BodyExt;
                use std::error::Error;

                let path = req.uri().path().to_string();
                let path = path.strip_prefix("/").map(|v| v.to_string()).unwrap_or(path);

                let headers = req.headers().clone();

                if let Some(file) = __ASSETS.get(&path) {
                    log::debug!("[#] 200 Ok (File) /{}", path);
                    return Ok(
                        hyper::Response::builder()
                            .status(200)
                            .header("Content-Type", file.0.to_string())
                            .body(match std::env::var("RM_LOCAL").is_ok() {
                                false => Body::from(file.1).full(),
                                true => {
                                    use std::io::Read;
                                    let mut byt = Vec::new();
                                    std::fs::File::open(std::path::PathBuf::from(#assets).join(path))
                                        .unwrap()
                                        .read_to_end(&mut byt)
                                        .unwrap();

                                    Body::from(byt.as_slice()).full() 
                                }
                            })
                            .unwrap()
                    )
                }

                Ok(match tokio::task::spawn(async move {
                    match (path, req.method().is_idempotent()) {
                        #(#paths)*
                        path => {
                            log::debug!("[?] 404 Not Found /{}", path.0);
                            return hyper::Response::builder()
                                .status(404)
                                .body(Body::default().full())
                                .unwrap()
                        }
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
                            .body(Body::from(format!("{:?}", err)).full())
                            .unwrap()
                    }
                })
            }
        }

    })
}