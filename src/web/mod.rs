mod error;
pub use error::WebError;

mod middleware;

mod routes;
pub use routes::*;

#[cfg(feature = "openapi")]
pub mod openapi;
