#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use anyhow::Result;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use log::{info, warn};
use router::*;
use std::{env, thread, time::Instant};
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
@ "POST" -> EndpointAdd::Data ([Result < i32 >])*/
pub struct EndpointAdd;
impl Endpoint for EndpointAdd {
    type Data = (i32, i32);
    type Returns = i32;
    fn is_idempotent() -> bool {
        false
    }
    fn auth() -> Box<
        dyn Fn(
            hyper::HeaderMap,
        ) -> futures::future::BoxFuture<'static, bool> + 'static + Send,
    > {
        Box::new(move |i: hyper::HeaderMap| Box::pin(NOAUTH(i)))
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
#[doc(r"Endpoint Handler for [#name]\n@ #method -> #struct_name::Data ([#arg])")]
pub async fn add(data: (i32, i32)) -> Result<i32> {
    { Ok(data.0 + data.1) }
}
/**Endpoint Struct for [now]
@ "POST" -> EndpointNow::Data ([Result < String >])*/
pub struct EndpointNow;
impl Endpoint for EndpointNow {
    type Data = ();
    type Returns = String;
    fn is_idempotent() -> bool {
        false
    }
    fn auth() -> Box<
        dyn Fn(
            hyper::HeaderMap,
        ) -> futures::future::BoxFuture<'static, bool> + 'static + Send,
    > {
        Box::new(move |i: hyper::HeaderMap| Box::pin(NOAUTH(i)))
    }
    fn handler() -> Box<
        dyn Fn(
            Self::Data,
        ) -> futures::future::BoxFuture<
                'static,
                anyhow::Result<Self::Returns>,
            > + 'static + Send,
    > {
        Box::new(move |i: Self::Data| Box::pin(now(i)))
    }
}
#[doc(r"Endpoint Handler for [#name]\n@ #method -> #struct_name::Data ([#arg])")]
pub async fn now(_: ()) -> Result<String> {
    {
        Ok({
            let res = ::alloc::fmt::format(format_args!("{0:?}", Instant::now()));
            res
        })
    }
}
#[assets("assets")]
pub enum Router {
    Sum(EndpointAdd),
    Now(EndpointNow),
}
static __ASSETS: std::sync::LazyLock<
    std::collections::BTreeMap<String, (String, &'static [u8])>,
> = std::sync::LazyLock::new(|| {
    use std::io::Read;
    let folder = std::path::PathBuf::from("/home/flora/rmrouter/assets");
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
                format_args!("Invalid asset folder: /home/flora/rmrouter/assets"),
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
        {
            let lvl = ::log::Level::Debug;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api::log(
                    format_args!("{0}", path),
                    lvl,
                    &("router", "router", ::log::__private_api::loc()),
                    (),
                );
            }
        };
        let headers = req.headers().clone();
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
                                        std::path::PathBuf::from("/home/flora/rmrouter/assets")
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
                        ("now", i) if i == EndpointNow::is_idempotent() => {
                            ({
                                if !(EndpointNow::auth())(headers).await {
                                    {
                                        let lvl = ::log::Level::Debug;
                                        if lvl <= ::log::STATIC_MAX_LEVEL
                                            && lvl <= ::log::max_level()
                                        {
                                            ::log::__private_api::log(
                                                format_args!("[-] 401 Unauthorized /now"),
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
                                    <EndpointNow as Endpoint>::Data,
                                >() {
                                    "()" => std::boxed::Box::new(()),
                                    _ => {
                                        let bytes = req
                                            .collect()
                                            .await
                                            .expect(
                                                &{
                                                    let res = ::alloc::fmt::format(
                                                        format_args!("Failed to read incoming bytes for {0}", "Now"),
                                                    );
                                                    res
                                                },
                                            )
                                            .to_bytes();
                                        std::boxed::Box::new(
                                            serde_json::from_str::<
                                                <EndpointNow as Endpoint>::Data,
                                            >(&String::from_utf8_lossy(&bytes[..]).to_string())
                                                .expect(
                                                    &{
                                                        let res = ::alloc::fmt::format(
                                                            format_args!("Failed to deserialize body for {0}", "Now"),
                                                        );
                                                        res
                                                    },
                                                ),
                                        )
                                    }
                                };
                                let body: <EndpointNow as Endpoint>::Data = *body
                                    .downcast::<<EndpointNow as Endpoint>::Data>()
                                    .unwrap();
                                match (EndpointNow::handler())(body).await {
                                    Ok(response) => {
                                        let bytes = serde_json::to_string(&response)
                                            .expect(
                                                &{
                                                    let res = ::alloc::fmt::format(
                                                        format_args!("Failed to serialize response for {0}", "Now"),
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
                                                    format_args!("[+] 200 Ok /now"),
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
                                        {
                                            let lvl = ::log::Level::Debug;
                                            if lvl <= ::log::STATIC_MAX_LEVEL
                                                && lvl <= ::log::max_level()
                                            {
                                                ::log::__private_api::log(
                                                    format_args!("[-] 400 Bad Request /now"),
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
