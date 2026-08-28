//! Trogdor is a library for handling errors inside a programming language
//! compiler or interpreter. It manages storing source code, saving multiple
//! errors, and displaying the errors along with the source code that caused
//! them.
//!
//! # Example
//!
//! ```
//! use trogdor::{ErrorHandler, Span};
//!
//! // Create an error handler to store errors.
//! let mut eh = ErrorHandler::new();
//! let source = eh.add_source("<test>", "1 + 2 *");
//! // Do some kind of parsing...
//! eh.err("unexpected multiplication sign", Span::new(6..7, source))
//!     .help_generic("try adding a number after it")
//!     .help("the file ends here", Span::new(7..8, source));
//! eh.warn_generic("no configuration specified");
//! // Print all errors & warnings to stderr
//! eprintln!("{eh}");
//! ```

pub mod diagnostic;
mod error_handler;
pub(crate) mod source;
pub mod span;

pub use error_handler::{ErrorHandler, SourceId};
pub use span::Span;
