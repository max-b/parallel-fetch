#![deny(missing_docs)]

//! Parallel Fetch !

mod errors;
mod fetch;
mod utils;

pub use errors::{FetchError, Result};
pub use fetch::{fetch, FetchOptions};
