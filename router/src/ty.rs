use std::future::Future;

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

#[doc(hidden)]
#[allow(non_snake_case)]
pub async fn NOAUTH() -> bool {
    true
}

pub trait Endpoint {
    type Data: DeserializeOwned;
    type Returns: Serialize;

    fn is_idempotent() -> bool { false }
    fn path() -> String { String::new() }    

    #[allow(unused_variables)]
    fn auth(auth_header: Option<String>) -> impl Future<Output = bool> {
        NOAUTH()
    }

    fn handler() -> fn(Self::Data) -> Result<Self::Returns>;
}
