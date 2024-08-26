use std::future::Future;

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use tokio;

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


#[cfg(test)]
mod test {
    use super::*;
    use macros::*;

    #[endpoint(idempotent, path = "/greet")]
    pub fn greet(name: String) -> Result<String> {
        Ok(format!("Hello, {}!", name))
    }

    #[allow(dead_code)]
    #[derive(Router)]
    pub enum Router {
        Greet(EndpointGreet)
    }
}