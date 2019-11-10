#![deny(missing_docs)]

//! Parallel Fetch !

mod errors;
mod fetch;
mod utils;

pub use fetch::{fetch, FetchOptions};
