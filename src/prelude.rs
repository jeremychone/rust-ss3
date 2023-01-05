//! Crate prelude

// Re-export the crate Error.
pub use crate::error::Error;

// Alias Result to be the crate Result.
pub type Result<T> = core::result::Result<T, Error>;

// Personal preference.
pub use std::format as f;
