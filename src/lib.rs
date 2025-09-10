pub mod diagnostic;
mod error_handler;
pub mod span;

pub use error_handler::{Config, ErrorHandler};

pub type Result<T, E = ()> = std::result::Result<T, E>;
