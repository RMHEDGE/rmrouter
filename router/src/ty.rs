use futures_signals::signal::Mutable;
#[cfg(not(target_arch = "x86_64"))]
use wasm_utils::{futures_signals::signal::Mutable, utilities::ModelExt};
use {
    futures::future::BoxFuture,
    http_body_util::Full,
    hyper::{
        body::{Bytes, Frame},
        HeaderMap,
    },
    hyper_util::rt::TokioIo,
    serde::{de::DeserializeOwned, Serialize},
    std::{
        future::Future,
        marker::PhantomData,
        pin::Pin,
        sync::Arc,
        task::{Context, Poll},
    },
    tokio::net::TcpStream,
};

#[doc(hidden)]
#[allow(non_snake_case)]
pub async fn NOAUTH(_: HeaderMap) -> bool {
    true
}

type AsyncHandler<I, O> = Box<dyn Fn(I) -> BoxFuture<'static, O> + Send + 'static>;

pub trait Endpoint {
    type Data: DeserializeOwned;
    type Returns: Serialize;

    fn is_idempotent() -> bool;
    fn auth() -> AsyncHandler<HeaderMap, bool>;
    fn handler() -> AsyncHandler<Self::Data, anyhow::Result<Self::Returns>>;
}

pub mod wasm_utils {
   pub mod utilities {
    use std::future::Future;

    pub trait ModelExt: Clone + 'static {
        fn get_token(&self) -> impl Future<Output = String> + Send;
    }
   }
}

use wasm_utils::utilities::*;

pub trait Fetch: Endpoint {
    // #[cfg(target_arch = "x86_64")]
    // fn fetch(data: Self::Data) -> impl Future<Output = anyhow::Result<Self::Returns>>;
    // #[cfg(not(target_arch = "x86_64"))]
    fn fetch_wasm(data: Self::Data, model: Arc<impl ModelExt>)
        -> Mutable<FetchRequest<Self::Returns>>;
}

pub struct IOTypeNotSend {
    _marker: PhantomData<*const ()>,
    stream: TokioIo<TcpStream>,
}

impl IOTypeNotSend {
    pub fn new(stream: TokioIo<TcpStream>) -> Self {
        Self {
            _marker: PhantomData,
            stream,
        }
    }
}

impl hyper::rt::Write for IOTypeNotSend {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

impl hyper::rt::Read for IOTypeNotSend {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

#[derive(Default)]
pub struct Body {
    _marker: PhantomData<*const ()>,
    data: Option<Bytes>,
}

impl From<String> for Body {
    fn from(value: String) -> Self {
        Body {
            _marker: PhantomData,
            data: Some(value.into()),
        }
    }
}

impl<'a> From<&'a [u8]> for Body {
    fn from(value: &'a [u8]) -> Self {
        Body {
            _marker: PhantomData,
            data: Some(Bytes::from_iter(value.iter().cloned())),
        }
    }
}

impl Body {
    pub fn full(self) -> Full<Bytes> {
        Full::new(self.data.unwrap_or_default())
    }
}

impl hyper::body::Body for Body {
    type Data = Bytes;
    type Error = hyper::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Poll::Ready(self.get_mut().data.take().map(|d| Ok(Frame::data(d))))
    }
}

#[derive(Default, Debug, Clone)]
pub enum FetchRequest<T> {
    /// Failed
    Failed(Arc<anyhow::Error>),
    /// Ready
    Ready(T),
    /// Loading
    Pending,
    /// Not yet requested
    #[default]
    Waiting,
}

impl<T: Clone> FetchRequest<T> {
    pub fn get(&self) -> Option<Result<T, Arc<anyhow::Error>>> {
        match &self {
            FetchRequest::Failed(arc) => Some(Err(arc.clone())),
            FetchRequest::Ready(t) => Some(Ok(t.clone())),
            FetchRequest::Pending => None,
            FetchRequest::Waiting => None,
        }
    }
}

impl<T: Clone> Future for FetchRequest<T> {
    type Output = Result<T, Arc<anyhow::Error>>;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        match self.get() {
            Some(v) => Poll::Ready(v),
            None => Poll::Pending,
        }
    }
}
