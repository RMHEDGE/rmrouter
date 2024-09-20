use anyhow::Result;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use log::{info, warn};
use reqwest;
use router::{endpoint, wasm_utils, Body, Endpoint, Fetch, IOTypeNotSend, Router, FetchRequest};
use std::{env, thread};
use tokio::net::TcpListener;

macro_rules! header {
    ($h:literal) => {
        __HEADERS
            .read()
            .unwrap()
            .get(&std::thread::current().id())
            .map(|l| {
                l.get($h)
                    .map(|h| h.to_str().map(|s| s.to_string()).ok())
                    .flatten()
            })
            .flatten()
    };
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
}

pub async fn server() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var(
        "RUST_LOG",
        env::var("RUST_LOG").unwrap_or("trace".to_string()),
    );
    pretty_env_logger::init();

    let addr: std::net::SocketAddr = ([127, 0, 0, 1], 3000).into();
    let listener = TcpListener::bind(addr).await?;

    info!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = IOTypeNotSend::new(TokioIo::new(stream));

        let service = service_fn(Router::route);
        tokio::task::spawn_local(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                warn!("Error serving connection: {:?}", err);
            }
        });
    }
}

#[endpoint(idempotent, auth = router::NOAUTH)]
pub async fn add(data: (i32, i32)) -> anyhow::Result<i32> {
    Ok(data.0 + data.1)
}
pub fn generate_html() -> String {
    r"<body>This is a valid* HTML file</body>".to_string()
}

#[derive(Router)]
#[assets("assets")]
#[html(generate_html)]
pub enum Router {
    Sum(EndpointAdd),
}
