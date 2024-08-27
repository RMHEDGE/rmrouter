use futures::future::BoxFuture;
use http_body_util::Full;
use hyper_util::rt::TokioIo;
use serde::de::DeserializeOwned;
use serde::Serialize;

use hyper::body::{Bytes, Frame};
use hyper::HeaderMap;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;

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

pub struct Body {
    _marker: PhantomData<*const ()>,
    data: Option<Bytes>,
}

impl Default for Body {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
            data: Default::default(),
        }
    }
}

impl From<String> for Body {
    fn from(value: String) -> Self {
        Body {
            _marker: PhantomData,
            data: Some(value.into()),
        }
    }
}

impl From<&'static [u8]> for Body {
    fn from(value: &'static [u8]) -> Self {
        Body {
            _marker: PhantomData,
            data: Some(value.into())
        }
    }
}

impl Body {
    pub fn full(self) -> Full<Bytes> {
        Full::new(Bytes::from(self.data.unwrap_or_default()))
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
