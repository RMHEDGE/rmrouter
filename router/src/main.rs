use anyhow::{anyhow, Result};
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::{TokioIo, TokioTimer};
use router::*;
use tokio::net::TcpListener;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();

    let addr: std::net::SocketAddr = ([127, 0, 0, 1], 3000).into();
    let listener = TcpListener::bind(addr).await?;

    println!("Started");
    loop {
        let (tcp, _) = listener.accept().await?;
        let io = TokioIo::new(tcp);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .timer(TokioTimer::new())
                .serve_connection(io, service_fn(Router::route))
                .await
            {
                eprintln!("Failed to serve conn: {err:?}")
            };
        });
    }
}



#[endpoint(path = "/sum")]
pub fn add(data: (i8, i8)) -> Result<i8> {
    data.0
        .checked_add(data.1)
        .ok_or(anyhow!("Failed to add {} & {}", data.0, data.1))
}

#[derive(Router)]
pub enum Router {
    Add(EndpointAdd),
}
