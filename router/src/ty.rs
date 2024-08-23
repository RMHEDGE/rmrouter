use std::future::Future;

use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Serialize};

#[doc(hidden)]
#[allow(non_snake_case)]
pub async fn NOAUTH() -> bool {
    true
}

pub enum HttpVerb {
    POST,
    PUT,
}

pub trait Endpoint {
    type Data: DeserializeOwned;
    type Returns: Serialize;

    fn verb() -> HttpVerb;
    #[allow(unused_variables)]
    fn auth(auth_header: Option<String>) -> impl Future<Output = bool> {
        NOAUTH()
    }

    fn handler() -> fn(Self::Data) -> Result<Self::Returns>;
}

pub struct EndpointAdd;
impl Endpoint for EndpointAdd {
    type Data = (i8, i8);
    type Returns = i8;

    fn verb() -> HttpVerb {
        HttpVerb::POST
    }

    fn handler() -> fn(Self::Data) -> Result<Self::Returns> {
        add
    }
}

pub fn add(data: (i8, i8)) -> Result<i8> {
    data.0
        .checked_add(data.1)
        .ok_or(anyhow!("Failed to add {} & {}", data.0, data.1))
}

pub enum Router {
    EndpointAdd
}