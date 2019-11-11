use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use futures;
use mockito;
use slog::debug;
use sloggers::null::NullLoggerBuilder;
use sloggers::terminal::TerminalLoggerBuilder;
use sloggers::types::Severity;
use sloggers::Build;
use tempfile::TempDir;
use tokio;
use tokio::prelude::*;

use parallel_fetch::{fetch, FetchOptions};

#[tokio::test]
async fn accept_ranges_none() {
    let url = &mockito::server_url();

    let logger = NullLoggerBuilder.build().unwrap();

    let _head_mock = mockito::mock("HEAD", "/")
        .with_status(200)
        .with_header("accept-ranges", "none")
        .create();

    let options = FetchOptions {
        url: url.to_owned(),
        output_option: None,
        num_fetches: 1,
        logger: logger.clone(),
    };

    let result = fetch(options).await;
    debug!(logger, "fetch finished"; "result" => format!("{:?}", &result));

    let error = result.expect_err("testing");
    // Kind of silly error checking - would be nice to actually leverage
    // the type system, but difficult with Box<dyn Error>
    assert_eq!(
        "Server's Accept-Ranges header set to none",
        error.description(),
    );
}

#[tokio::test]
async fn accept_ranges_missing() {
    let url = &mockito::server_url();

    let logger = NullLoggerBuilder.build().unwrap();

    let _head_mock = mockito::mock("HEAD", "/").with_status(200).create();

    let options = FetchOptions {
        url: url.to_owned(),
        output_option: None,
        num_fetches: 1,
        logger: logger.clone(),
    };

    let result = fetch(options).await;
    debug!(logger, "fetch finished"; "result" => format!("{:?}", &result));

    let error = result.expect_err("testing");
    // Kind of silly error checking - would be nice to actually leverage
    // the type system, but difficult with Box<dyn Error>
    assert_eq!(
        "Server does not include Accept-Ranges header",
        error.description(),
    );
}

#[tokio::test]
async fn content_length_missing() {
    let url = &mockito::server_url();

    let logger = NullLoggerBuilder.build().unwrap();

    let _head_mock = mockito::mock("HEAD", "/")
        .with_status(200)
        .with_header("accept-ranges", "bytes")
        .create();

    let options = FetchOptions {
        url: url.to_owned(),
        output_option: None,
        num_fetches: 1,
        logger: logger.clone(),
    };

    let result = fetch(options).await;
    debug!(logger, "fetch finished"; "result" => format!("{:?}", &result));

    let error = result.expect_err("testing");
    // Kind of silly error checking - would be nice to actually leverage
    // the type system, but difficult with Box<dyn Error>
    assert_eq!(
        "Server does not include Content-Length header",
        error.description(),
    );
}

#[tokio::test]
async fn single_fetch() {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut temp_file_path = PathBuf::from(temp_dir.path());
    temp_file_path.push("out.tmp");

    let url = &mockito::server_url();

    let logger = NullLoggerBuilder.build().unwrap();

    let _head_mock = mockito::mock("HEAD", "/")
        .with_status(200)
        .with_header("accept-ranges", "bytes")
        .with_header("content-length", "10")
        .create();

    let _body_mock = mockito::mock("GET", "/")
        .with_status(206)
        .with_header("content-length", "10")
        .with_header("content-range", "bytes 0-9/10")
        .with_body(&b"HelloWorld")
        .create();

    let options = FetchOptions {
        url: url.to_owned(),
        output_option: Some(temp_file_path.to_str().unwrap().to_owned()),
        num_fetches: 1,
        logger: logger.clone(),
    };

    let result = fetch(options).await;
    debug!(logger, "fetch finished"; "result" => format!("{:?}", &result));

    assert!(result.is_ok());

    let mut file = File::open(temp_file_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents, "HelloWorld");
}

#[tokio::test]
async fn second_fetch_fails() {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut temp_file_path = PathBuf::from(temp_dir.path());
    temp_file_path.push("out.tmp");

    let url = &mockito::server_url();

    let logger = NullLoggerBuilder.build().unwrap();

    let _head_mock = mockito::mock("HEAD", "/")
        .with_status(200)
        .with_header("accept-ranges", "bytes")
        .with_header("content-length", "10")
        .create();

    let _body_mock1 = mockito::mock("GET", "/")
        .with_status(206)
        .with_header("content-length", "10")
        .with_header("content-range", "bytes 0-9/10")
        .with_body(&b"HelloWorld")
        .create();

    let _body_mock2 = mockito::mock("GET", "/").with_status(500).create();

    let options = FetchOptions {
        url: url.to_owned(),
        output_option: Some(temp_file_path.to_str().unwrap().to_owned()),
        num_fetches: 2,
        logger: logger.clone(),
    };

    let result = fetch(options).await;
    debug!(logger, "fetch finished"; "result" => format!("{:?}", &result));

    let error = result.expect_err("testing");
    // Kind of silly error checking - would be nice to actually leverage
    // the type system, but difficult with Box<dyn Error>
    assert!(format!("{}", error).contains("500 Internal Server Error"),);
}

#[tokio::test]
async fn two_fetches() {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut temp_file_path = PathBuf::from(temp_dir.path());
    temp_file_path.push("out.tmp");

    let url = &mockito::server_url();

    let logger = NullLoggerBuilder.build().unwrap();

    let _head_mock = mockito::mock("HEAD", "/")
        .with_status(200)
        .with_header("accept-ranges", "bytes")
        .with_header("content-length", "10")
        .create();

    let _body_mock = mockito::mock("GET", "/")
        .with_status(206)
        .match_header("range", "bytes=0-4")
        .with_header("content-length", "5")
        .with_header("content-range", "bytes 0-4/10")
        .with_body(&b"Hello")
        .create();

    let _body_mock2 = mockito::mock("GET", "/")
        .with_status(206)
        .match_header("range", "bytes=5-9")
        .with_header("content-length", "5")
        .with_header("content-range", "bytes 5-9/10")
        .with_body(&b"World")
        .create();

    let options = FetchOptions {
        url: url.to_owned(),
        output_option: Some(temp_file_path.to_str().unwrap().to_owned()),
        num_fetches: 2,
        logger: logger.clone(),
    };

    let result = fetch(options).await;
    debug!(logger, "fetch finished"; "result" => format!("{:?}", &result));

    assert!(result.is_ok());

    let mut file = File::open(temp_file_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents, "HelloWorld");
}
