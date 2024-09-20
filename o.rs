#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use anyhow::Result;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use log::{info, warn};
use reqwest;
use router::{
    endpoint, wasm_utils, Body, Endpoint, Fetch, IOTypeNotSend, Router, FetchRequest,
};
use std::{env, thread};
use tokio::net::TcpListener;
pub fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let body = async {
        let server = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to construct runtime");
            let local = tokio::task::LocalSet::new();
            local.block_on(&rt, server()).unwrap();
        });
        server.join().unwrap();
        Ok(())
    };
    #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
    {
        return tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
pub async fn server() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", env::var("RUST_LOG").unwrap_or("trace".to_string()));
    pretty_env_logger::init();
    let addr: std::net::SocketAddr = ([127, 0, 0, 1], 3000).into();
    let listener = TcpListener::bind(addr).await?;
    {
        let lvl = ::log::Level::Info;
        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
            ::log::__private_api::log(
                format_args!("Listening on http://{0}", addr),
                lvl,
                &("router", "router", ::log::__private_api::loc()),
                (),
            );
        }
    };
    loop {
        let (stream, _) = listener.accept().await?;
        let io = IOTypeNotSend::new(TokioIo::new(stream));
        let service = service_fn(Router::route);
        tokio::task::spawn_local(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                {
                    let lvl = ::log::Level::Warn;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!("Error serving connection: {0:?}", err),
                            lvl,
                            &("router", "router", ::log::__private_api::loc()),
                            (),
                        );
                    }
                };
            }
        });
    }
}
/**Endpoint Struct for [add]
@ "PUT" -> EndpointAdd::Data ([anyhow :: Result < i32 >])*/
pub struct EndpointAdd;
impl Endpoint for EndpointAdd {
    type Data = (i32, i32);
    type Returns = i32;
    fn is_idempotent() -> bool {
        true
    }
    fn auth() -> Box<
        dyn Fn(
            hyper::HeaderMap,
        ) -> futures::future::BoxFuture<'static, bool> + 'static + Send,
    > {
        Box::new(move |i: hyper::HeaderMap| Box::pin(router::NOAUTH(i)))
    }
    fn handler() -> Box<
        dyn Fn(
            Self::Data,
        ) -> futures::future::BoxFuture<
                'static,
                anyhow::Result<Self::Returns>,
            > + 'static + Send,
    > {
        Box::new(move |i: Self::Data| Box::pin(add(i)))
    }
}
#[doc("Endpoint Handler for [#name]\n@ #method -> #struct_name::Data ([#arg])")]
pub async fn add(data: (i32, i32)) -> anyhow::Result<i32> {
    Ok(data.0 + data.1)
}
pub fn generate_html() -> String {
    r"<body>This is a valid* HTML file</body>".to_string()
}
#[assets("assets")]
#[html(generate_html)]
pub enum Router {
    Sum(EndpointAdd),
}
impl Fetch for EndpointAdd {
    fn fetch_wasm(
        data: Self::Data,
        model: std::sync::Arc<impl wasm_utils::utilities::ModelExt>,
    ) -> futures_signals::signal::Mutable<FetchRequest<Self::Data>> {
        let signal = futures_signals::signal::Mutable::new(
            FetchRequest::<Self::Data>::Pending,
        );
        wasm_bindgen_futures::spawn_local({
            let signal = signal.clone();
            async move {
                let base_url = web_sys::window().unwrap().origin();
                match reqwest::Client::new()
                    .request(
                        match Self::is_idempotent() {
                            true => reqwest::Method::PUT,
                            false => reqwest::Method::POST,
                        },
                        {
                            let res = ::alloc::fmt::format(
                                format_args!("{1}/api/{0}", "EndpointAdd", base_url),
                            );
                            res
                        },
                    )
                    .header("Connection", "Keep-Alive")
                    .header("Keep-Alive", "timeout=600")
                    .header(
                        "Authorization",
                        {
                            let res = ::alloc::fmt::format(
                                format_args!("Bearer {0}", model.get_token().await),
                            );
                            res
                        },
                    )
                    .json(&data)
                    .send()
                    .await
                {
                    Ok(v) => {
                        match v.json::<Self::Returns>().await {
                            Ok(v) => signal.set(FetchRequest::<Self::Data>::Ready(v)),
                            Err(e) => signal.set(FetchRequest::<Self::Data>::Error(e)),
                        }
                    }
                    Err(e) => signal.set(FetchRequest::<Self::Data>::Error(e)),
                };
            }
        });
        signal
    }
}
static __ASSETS: std::sync::LazyLock<
    std::collections::BTreeMap<String, (String, &'static [u8])>,
> = std::sync::LazyLock::new(|| {
    use std::io::Read;
    let folder = std::path::PathBuf::from(
        "/home/flora/Documents/projects/rmrouter/assets",
    );
    let mut assets = std::collections::BTreeMap::<
        String,
        (String, &'static [u8]),
    >::new();
    {
        let lvl = ::log::Level::Info;
        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
            ::log::__private_api::log(
                format_args!("[#] Collating local assets"),
                lvl,
                &("router", "router", ::log::__private_api::loc()),
                (),
            );
        }
    };
    if !folder.exists() {
        {
            ::core::panicking::panic_fmt(
                format_args!(
                    "Invalid asset folder: /home/flora/Documents/projects/rmrouter/assets",
                ),
            );
        };
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
                .strip_prefix(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!("{0}/", folder.display().to_string()),
                        );
                        res
                    },
                )
                .unwrap()
                .to_string();
            let mut byt = Vec::new();
            std::fs::File::open(entry.path()).unwrap().read_to_end(&mut byt).unwrap();
            let byt = Box::leak(Box::new(byt));
            {
                let lvl = ::log::Level::Debug;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api::log(
                        format_args!("[#] Serving asset: {0}", route),
                        lvl,
                        &("router", "router", ::log::__private_api::loc()),
                        (),
                    );
                }
            };
            assets
                .insert(
                    route.clone(),
                    (mime_guess::from_path(route).first_or_text_plain().to_string(), byt),
                );
        });
    assets
});
static __HEADERS: std::sync::LazyLock<
    std::sync::RwLock<std::collections::HashMap<std::thread::ThreadId, hyper::HeaderMap>>,
> = std::sync::LazyLock::new(|| {
    std::sync::RwLock::new(std::collections::HashMap::new())
});
impl Router {
    pub async fn route(
        req: hyper::Request<hyper::body::Incoming>,
    ) -> std::result::Result<
        hyper::Response<http_body_util::Full<bytes::Bytes>>,
        std::convert::Infallible,
    > {
        use http_body_util::BodyExt;
        use std::error::Error;
        let path = req.uri().path().to_string();
        let path = path.strip_prefix("/").map(|v| v.to_string()).unwrap_or(path);
        let headers = req.headers().clone();
        __HEADERS
            .write()
            .unwrap()
            .insert(std::thread::current().id(), req.headers().clone());
        if req.method() == hyper::Method::GET {
            if let Some(file) = __ASSETS.get(&path) {
                {
                    let lvl = ::log::Level::Debug;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!("[#] 200 Ok (File) /{0}", path),
                            lvl,
                            &("router", "router", ::log::__private_api::loc()),
                            (),
                        );
                    }
                };
                return Ok(
                    hyper::Response::builder()
                        .status(200)
                        .header("Content-Type", file.0.to_string())
                        .body(
                            match std::env::var("RM_LOCAL").is_ok() {
                                false => Body::from(file.1).full(),
                                true => {
                                    use std::io::Read;
                                    let mut byt = Vec::new();
                                    std::fs::File::open(
                                            std::path::PathBuf::from(
                                                    "/home/flora/Documents/projects/rmrouter/assets",
                                                )
                                                .join(path),
                                        )
                                        .unwrap()
                                        .read_to_end(&mut byt)
                                        .unwrap();
                                    Body::from(byt.as_slice()).full()
                                }
                            },
                        )
                        .unwrap(),
                );
            } else {
                return Ok(
                    hyper::Response::builder()
                        .status(200)
                        .header("Content-Type", "text/html")
                        .body(Body::from(generate_html()).full())
                        .unwrap(),
                )
            }
        }
        Ok(
            match tokio::task::spawn(async move {
                    match (path.as_str(), req.method().is_idempotent()) {
                        ("sum", i) if i == EndpointAdd::is_idempotent() => {
                            ({
                                if !(EndpointAdd::auth())(headers).await {
                                    {
                                        let lvl = ::log::Level::Debug;
                                        if lvl <= ::log::STATIC_MAX_LEVEL
                                            && lvl <= ::log::max_level()
                                        {
                                            ::log::__private_api::log(
                                                format_args!("[-] 401 Unauthorized /sum"),
                                                lvl,
                                                &("router", "router", ::log::__private_api::loc()),
                                                (),
                                            );
                                        }
                                    };
                                    return hyper::Response::builder()
                                        .status(401)
                                        .body(
                                            Body::from({
                                                    let res = ::alloc::fmt::format(
                                                        format_args!(
                                                            "You aren\'t authorized to access this endpoint. If you believe this is a mistake, talk to your RMHedge Contact",
                                                        ),
                                                    );
                                                    res
                                                })
                                                .full(),
                                        )
                                        .unwrap();
                                }
                                let body: std::boxed::Box<dyn std::any::Any> = match std::any::type_name::<
                                    <EndpointAdd as Endpoint>::Data,
                                >() {
                                    "()" => std::boxed::Box::new(()),
                                    _ => {
                                        let bytes = req
                                            .collect()
                                            .await
                                            .expect(
                                                &{
                                                    let res = ::alloc::fmt::format(
                                                        format_args!("Failed to read incoming bytes for {0}", "Sum"),
                                                    );
                                                    res
                                                },
                                            )
                                            .to_bytes();
                                        std::boxed::Box::new(
                                            serde_json::from_str::<
                                                <EndpointAdd as Endpoint>::Data,
                                            >(&String::from_utf8_lossy(&bytes[..]).to_string())
                                                .expect(
                                                    &{
                                                        let res = ::alloc::fmt::format(
                                                            format_args!("Failed to deserialize body for {0}", "Sum"),
                                                        );
                                                        res
                                                    },
                                                ),
                                        )
                                    }
                                };
                                let body: <EndpointAdd as Endpoint>::Data = *body
                                    .downcast::<<EndpointAdd as Endpoint>::Data>()
                                    .unwrap();
                                match (EndpointAdd::handler())(body).await {
                                    Ok(response) => {
                                        __HEADERS
                                            .write()
                                            .unwrap()
                                            .remove(&std::thread::current().id());
                                        let bytes = serde_json::to_string(&response)
                                            .expect(
                                                &{
                                                    let res = ::alloc::fmt::format(
                                                        format_args!("Failed to serialize response for {0}", "Sum"),
                                                    );
                                                    res
                                                },
                                            );
                                        {
                                            let lvl = ::log::Level::Debug;
                                            if lvl <= ::log::STATIC_MAX_LEVEL
                                                && lvl <= ::log::max_level()
                                            {
                                                ::log::__private_api::log(
                                                    format_args!("[+] 200 Ok /sum"),
                                                    lvl,
                                                    &("router", "router", ::log::__private_api::loc()),
                                                    (),
                                                );
                                            }
                                        };
                                        return hyper::Response::builder()
                                            .status(200)
                                            .body(Body::from(bytes).full())
                                            .unwrap();
                                    }
                                    Err(e) => {
                                        __HEADERS
                                            .write()
                                            .unwrap()
                                            .remove(&std::thread::current().id());
                                        {
                                            let lvl = ::log::Level::Debug;
                                            if lvl <= ::log::STATIC_MAX_LEVEL
                                                && lvl <= ::log::max_level()
                                            {
                                                ::log::__private_api::log(
                                                    format_args!("[-] 400 Bad Request /sum"),
                                                    lvl,
                                                    &("router", "router", ::log::__private_api::loc()),
                                                    (),
                                                );
                                            }
                                        };
                                        return hyper::Response::builder()
                                            .status(400)
                                            .body(Body::from(e.to_string()).full())
                                            .unwrap();
                                    }
                                };
                            })
                        }
                        path => {
                            {
                                let lvl = ::log::Level::Debug;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        format_args!("[?] 404 Not Found /{0}", path.0),
                                        lvl,
                                        &("router", "router", ::log::__private_api::loc()),
                                        (),
                                    );
                                }
                            };
                            return hyper::Response::builder()
                                .status(404)
                                .body(Body::default().full())
                                .unwrap();
                        }
                    }
                })
                .await
            {
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
                        .body(
                            Body::from({
                                    let res = ::alloc::fmt::format(format_args!("{0:?}", err));
                                    res
                                })
                                .full(),
                        )
                        .unwrap()
                }
            },
        )
    }
}
async fn abc() {}
