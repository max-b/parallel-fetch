use std::error::Error;
use std::fmt;
use std::io;
use std::result;

use reqwest;

#[derive(Debug)]
/// Errors during Fetch
pub enum FetchError {
    /// An Error indicating an issue with Server Support
    ServerSupportError(String),
    /// Invalid Arguments
    InvalidArgumentsError(String),
    /// Validation Failure
    ValidationError(String),
    /// Error originating in reqwest
    ReqwestError(reqwest::Error),
    /// Error originating from io
    IoError(io::Error),
    /// Error in creating header
    InvalidHeaderValueError(reqwest::header::InvalidHeaderValue),
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
            FetchError::ValidationError(string) => string,
            FetchError::ReqwestError(err) => err.description(),
            FetchError::IoError(err) => err.description(),
            FetchError::InvalidHeaderValueError(err) => err.description(),
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FetchError::ServerSupportError(_) => None,
            FetchError::InvalidArgumentsError(_) => None,
            FetchError::ValidationError(_) => None,
            FetchError::ReqwestError(err) => Some(err),
            FetchError::IoError(err) => Some(err),
            FetchError::InvalidHeaderValueError(err) => Some(err),
        }
    }
}

impl From<io::Error> for Box<FetchError> {
    fn from(err: io::Error) -> Box<FetchError> {
        Box::new(FetchError::IoError(err))
    }
}

impl From<reqwest::Error> for Box<FetchError> {
    fn from(err: reqwest::Error) -> Box<FetchError> {
        Box::new(FetchError::ReqwestError(err))
    }
}

impl From<reqwest::header::ToStrError> for Box<FetchError> {
    fn from(_err: reqwest::header::ToStrError) -> Box<FetchError> {
        Box::new(FetchError::ServerSupportError(
            "Could not parse header to string".to_owned(),
        ))
    }
}

impl From<reqwest::header::InvalidHeaderValue> for Box<FetchError> {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Box<FetchError> {
        Box::new(FetchError::InvalidHeaderValueError(err))
    }
}
/// A Result that wraps FetchError
pub type Result<T> = result::Result<T, Box<FetchError>>;
