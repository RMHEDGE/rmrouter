use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::Result;
use bytes::Bytes;
use futures::future::BoxFuture;
use http_body_util::Full;
use hyper::{body::Frame, HeaderMap};
use serde::{de::DeserializeOwned, Serialize};
use tokio;

#[doc(hidden)]
#[allow(non_snake_case)]
pub async fn NOAUTH(_: HeaderMap) -> bool {
    true
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
            data: Some(value.into()),
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

type AsyncHandler<I, O> = Box<dyn Fn(I) -> BoxFuture<'static, O> + Send + 'static>;

pub trait Endpoint {
    type Data: DeserializeOwned;
    type Returns: Serialize;

    fn is_idempotent() -> bool;
    fn auth() -> AsyncHandler<HeaderMap, bool>;
    fn handler() -> AsyncHandler<Self::Data, anyhow::Result<Self::Returns>>;
}

#[cfg(test)]
mod test {
    use super::*;
    use macros::*;

    #[endpoint(idempotent, auth = NOAUTH)]
    pub async fn greet(name: String) -> Result<String> {
        Ok(format!("Hello, {}!", name))
    }

    #[derive(Router)]
    #[assets("./assets")]
    pub enum Router {
        Greet(EndpointGreet),
    }
}
