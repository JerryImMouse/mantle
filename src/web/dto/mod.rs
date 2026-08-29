pub mod auth;
pub mod health;

// Cross-module DTO, this one doesn't have specific module to live in
// They are being re-exported using `pub use` to make `use` like `use dto::SomeSharedDto`
// `auth` and `health` are module-specific, they should never be re-exported using `pub use`!!
mod shared;
pub use shared::*;
