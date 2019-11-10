use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
/// Errors during Fetch
pub enum FetchError {
    /// An Error indicating an issue with Server Support
    ServerSupportError(String),
    /// Invalid Arguments
    InvalidArgumentsError(String),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for FetchError {
    fn description(&self) -> &str {
        match self {
            FetchError::ServerSupportError(string) => string,
            FetchError::InvalidArgumentsError(string) => string,
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FetchError::ServerSupportError(_) => None,
            FetchError::InvalidArgumentsError(_) => None,
        }
    }
}
