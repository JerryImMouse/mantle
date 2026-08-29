mod error;
pub use error::WebError;

mod middleware;

mod routes;
pub use routes::*;

mod dto;

#[cfg(feature = "openapi")]
pub mod openapi;
