use std::{future::Future, sync::Arc};

use anyhow::Result;
use futures::future::BoxFuture;
use serde::{de::DeserializeOwned, Serialize};

#[doc(hidden)]
#[allow(non_snake_case)]
pub async fn NOAUTH() -> bool {
    true
}

pub trait Endpoint {
    type Data: DeserializeOwned;
    type Returns: Serialize;

    fn is_idempotent() -> bool;
    fn path() -> String;

    #[allow(unused_variables)]
    fn auth(auth_header: Option<String>) -> impl Future<Output = bool> {
        NOAUTH()
    }

    fn handler() -> AsyncPtr<Self::Data, Result<Self::Returns>>;
}

#[derive(Clone)]
pub struct AsyncPtr<I, R>(Arc<Box<dyn Fn(I) -> BoxFuture<'static, R> + Send + 'static>>);

unsafe impl<I, R> Send for AsyncPtr<I, R> {}
unsafe impl<I, R> Sync for AsyncPtr<I, R> {}

impl<I, R> AsyncPtr<I, R> {
    pub fn new<I2, R2>(f: fn(I2) -> R2) -> AsyncPtr<I2, R2::Output>
    where
        R2: Future<Output = R> + Send + 'static,
        I2: 'static,
    {
        AsyncPtr(Arc::new(Box::new(move |i: I2| Box::pin(f(i)))))
    }

    pub async fn run(&self, i: I) -> R {
        (self.0)(i).await
    }
}
