mod error;
pub use error::WebError;

mod middleware;

mod routes;
pub use routes::build_router;

