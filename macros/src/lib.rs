use std::env;

use heck::{AsPascalCase, AsSnekCase};
use helpers::{get_inner_type, preamble, RouteInfo, unit};
use proc_macro::TokenStream;
use quote::{format_ident, ToTokens};
use syn::{parse_macro_input, DeriveInput};

mod helpers;

macro_rules! err {
    ($result:expr) => {
        match $result {
            Err(e) => return e.into_compile_error().into(),
            Ok(e) => e,
        }
    };
}

fn strip(a: &str) -> String {
    a.strip_prefix("\"").and_then(|b| b.strip_suffix("\"")).map(|v| v.to_string()).unwrap_or(a.to_string())
}

#[proc_macro_attribute]
pub fn endpoint(annot: TokenStream, item: TokenStream) -> TokenStream {
    let it = item.clone();
    let meta = parse_macro_input!(it as helpers::Meta);
    
    let (_, name, _, arg, arg_name, ret, _, block) =
    (meta.0, meta.1, meta.2, meta.3, meta.4, meta.5, meta.6, meta.7);

    let arg = arg.unwrap_or(unit());
    let arg_name = arg_name.unwrap_or(format_ident!("_"));

    let info = err!(RouteInfo::parse(annot.into()));
    let (idempotent, auth) = (info.is_idempotent, info.auth);

    let method = match idempotent {
        true => "PUT",
        false => "POST"
    };

    let inner_ret = get_inner_type(ret.clone());
    let struct_name = quote::format_ident!("Endpoint{}", AsPascalCase(name.to_string()).to_string());

    quote::quote! {
        #[doc = concat!("Endpoint Struct for [", stringify!(#name) ,"]\n@ ", stringify!(#method), " -> ", stringify!(#struct_name), "::Data ([", stringify!(#ret), "])")]
        pub struct #struct_name;

        impl Endpoint for #struct_name {
            type Data = #arg;
            type Returns = #inner_ret;

            fn is_idempotent() -> bool { #idempotent }

            fn auth() -> Box<dyn Fn(hyper::HeaderMap) -> futures::future::BoxFuture<'static, bool> + 'static + Send> {
                Box::new(move |i: hyper::HeaderMap| Box::pin(#auth(i)))
            }
            
            fn handler() -> Box<dyn Fn(Self::Data) -> futures::future::BoxFuture<'static, anyhow::Result<Self::Returns>> + 'static + Send> {
                Box::new(move |i: Self::Data| Box::pin(#name(i)))
            }
        }

        #[doc("Endpoint Handler for [#name]\n@ #method -> #struct_name::Data ([#arg])")]
        pub async fn #name(#arg_name: #arg) -> #ret #block
        
    }
    .into()

}


#[proc_macro_derive(Router, attributes(path, assets, html))]
pub fn router(item: TokenStream) -> TokenStream {
    let (input, name, data) = preamble(parse_macro_input!(item as DeriveInput));
    
    let assets = input.attrs.iter().find(|a| a.path().is_ident("assets"));
    let assets = assets.map(|a| {
        err!(a.parse_args::<syn::LitStr>()
            .map(|a| a.to_token_stream())
            .map_err(|_| syn::Error::new_spanned(a.into_token_stream(), "Assets attribute should be a literal string")))
    });

    let html = input.attrs.iter().find(|a| a.path().is_ident("html"));
    let html = html.map(|a| {
        err!(a.parse_args::<syn::Expr>()
            .map(|a| a.to_token_stream())
            .map_err(|_| syn::Error::new_spanned(a.into_token_stream(), "HTML attribute should point to a function")))
    });

    let assets = assets
        .map(|a| format!(
            "{}/{}",
                env::current_dir()
                    .map(|d| d.display().to_string())
                    .unwrap_or_default(),
                strip(&a.to_string())
            ));


    let assets = err!(assets.ok_or(
        syn::Error::new_spanned(name.to_token_stream(), "Missing #[assets()] attribute")
    ));

    let paths: Result<Vec<proc_macro2::TokenStream>, syn::Error> = data.variants.iter().map(|variant| {

        let path = format_ident!("{}", AsSnekCase(variant.ident.to_string()).to_string());
        let inner = variant.fields.iter()
            .next()
            .map(|ty| ty.ty.clone())
            .ok_or(syn::Error::new_spanned(
                variant.to_token_stream(), 
                format!("No endpoint specified for {}", variant.ident)
            ))?;
        
        let inner_name = &variant.ident;
        
        Ok(quote::quote! {
            (stringify!(#path), i) if i == #inner::is_idempotent() => ({
                if !(#inner::auth())(headers).await {
                    log::debug!(concat!("[-] 401 Unauthorized /", stringify!(#path)));
                    return hyper::Response::builder()
                        .status(401)
                        .body(Body::from(format!("You aren't authorized to access this endpoint. If you believe this is a mistake, talk to your RMHedge Contact")).full())
                        .unwrap()
                }

                let body: std::boxed::Box<dyn std::any::Any> = match std::any::type_name::<<#inner as Endpoint>::Data>() {
                    "()" => std::boxed::Box::new(()),
                    _ => {
                        let bytes = req.collect().await.expect(&format!("Failed to read incoming bytes for {}", stringify!(#inner_name))).to_bytes();
                        std::boxed::Box::new(serde_json::from_str::<<#inner as Endpoint>::Data>(&String::from_utf8_lossy(&bytes[..]).to_string()).expect(&format!("Failed to deserialize body for {}", stringify!(#inner_name))))
                    }
                };

                let body: <#inner as Endpoint>::Data = *body.downcast::<<#inner as Endpoint>::Data>().unwrap();
                
                match (#inner::handler())(body).await {
                    Ok(response) => {
                        __HEADERS.write().unwrap().remove(&std::thread::current().id());
                        let bytes = serde_json::to_string(&response).expect(&format!("Failed to serialize response for {}", stringify!(#inner_name)));
                        
                        log::debug!(concat!("[+] 200 Ok /", stringify!(#path)));
                        return hyper::Response::builder()
                            .status(200)
                            .body(Body::from(bytes).full())
                            .unwrap()
                    },
                    Err(e) => {
                        __HEADERS.write().unwrap().remove(&std::thread::current().id());
                        log::debug!(concat!("[-] 400 Bad Request /", stringify!(#path)));
                        return hyper::Response::builder()
                            .status(400)
                            .body(Body::from(e.to_string()).full())
                            .unwrap()
                    }
                };
            }),
        })

    }).collect();

    let paths: Vec<proc_macro2::TokenStream> = err!(paths);
    let route_callers = data.variants.iter().map(|variant| {
        let ident = variant.fields.iter().next().unwrap().ty.clone();
        quote::quote! {
            impl Fetch for #ident {

                // #[cfg(target_arch = "x86_64")]
                // async fn fetch(data: Self::Data) -> anyhow::Result<Self::Returns> {
                //     Ok(
                //         reqwest::Client::new().request(match Self::is_idempotent() {
                //             true => reqwest::Method::PUT,
                //             false => reqwest::Method::POST
                //         }, format!("https://.../{}", stringify!(#ident)))
                //             .header("Connection", "Keep-Alive")
                //             .header("Keep-Alive", "timeout=600")
                //             .json(&data)
                //             .send().await?
                //             .json::<Self::Returns>().await?
                //     )
                // }

                // #[cfg(not(target_arch = "x86_64"))]
                fn fetch_wasm(data: Self::Data, model: std::sync::Arc<impl wasm_utils::utilities::ModelExt>) -> futures_signals::signal::Mutable<FetchRequest<Self::Returns>> {
                    let signal = futures_signals::signal::Mutable::new(FetchRequest::<Self::Returns>::Pending);
                    wasm_bindgen_futures::spawn_local({
                        let signal = signal.clone();
                        async move {
                            let base_url = web_sys::window().unwrap().origin();
                                match reqwest::Client::new().request(match Self::is_idempotent() {
                                    true => reqwest::Method::PUT,
                                    false => reqwest::Method::POST
                                }, format!("{base_url}/api/{}", stringify!(#ident)))
                                    .header("Connection", "Keep-Alive")
                                    .header("Keep-Alive", "timeout=600")
                                    .header("Authorization", format!("Bearer {}", model.get_token().await))
                                    .json(&data)
                                    .send().await {
                                        Ok(v) => match v.json::<Self::Returns>().await {
                                            Ok(v) => signal.set(FetchRequest::<Self::Returns>::Ready(v)),
                                            Err(e) => signal.set(FetchRequest::<Self::Returns>::Failed(std::sync::Arc::new(anyhow::anyhow!("{}", e))))
                                        },
                                        Err(e) => signal.set(FetchRequest::<Self::Returns>::Failed(std::sync::Arc::new(anyhow::anyhow!("{}", e))))
                                    };
                        }
                    });

                    signal
                }
            }
        }
    }).collect::<Vec<_>>();

    TokenStream::from(quote::quote! {

        #(#route_callers)*

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

        static __HEADERS: std::sync::LazyLock<std::sync::RwLock<std::collections::HashMap::<std::thread::ThreadId, hyper::HeaderMap>>> = std::sync::LazyLock::new(|| {
            std::sync::RwLock::new(std::collections::HashMap::new())
        });

        impl #name {
            pub async fn route(req: hyper::Request<hyper::body::Incoming>) -> std::result::Result<hyper::Response<http_body_util::Full<bytes::Bytes>>, std::convert::Infallible> {
                use http_body_util::BodyExt;
                use std::error::Error;

                let path = req.uri().path().to_string();
                let path = path.strip_prefix("/").map(|v| v.to_string()).unwrap_or(path);
                let headers = req.headers().clone();

                __HEADERS.write().unwrap().insert(std::thread::current().id(), req.headers().clone());

                if req.method() == hyper::Method::GET {
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
                    } else {
                        return Ok(
                            hyper::Response::builder()
                                .status(200)
                                .header("Content-Type", "text/html")
                                .body(Body::from(#html()).full())
                                .unwrap()
                        )
                    }
                }

                Ok(match tokio::task::spawn(async move {
                    match (path.as_str(), req.method().is_idempotent()) {
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