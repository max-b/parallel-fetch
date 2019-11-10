use std::error::Error;
use std::path::PathBuf;

use async_std::fs::OpenOptions;
use async_std::io::prelude::*;
use async_std::io::{BufWriter, SeekFrom};
use futures_util::future::try_join_all;
use reqwest::header::{HeaderMap, ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, RANGE};
use reqwest::StatusCode;
use slog::{self, info, Logger};

use crate::errors::FetchError;
use crate::utils::{create_ranges, parse_path};

#[derive(Debug, PartialEq)]
/// A range of bytes to fetch
pub struct Range {
    /// The start of the range
    pub start: u64,
    /// The end of the range
    pub end: u64,
}

impl slog::Value for Range {
    fn serialize(
        &self,
        _rec: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, &format!("{:?}", self))
    }
}

/// Options for fetching
pub struct FetchOptions {
    /// The url to fetch from
    pub url: String,
    /// An optional output location
    pub output_option: Option<String>,
    /// The number of parallel fetches to execute
    pub num_fetches: u64,
    /// A logger
    pub logger: Logger,
}

/// Fetch a url which accepts range requests w/ parallel requests
pub async fn fetch(options: FetchOptions) -> Result<(), Box<dyn Error>> {
    let path = parse_path(options.output_option, &options.url)?;

    info!(options.logger, "fetching"; "path" => format!("{:?}", &path));

    let client = reqwest::Client::new();
    let head = client.head(&options.url).send().await?.error_for_status()?;

    let headers = head.headers();

    let accept_ranges =
        headers
            .get(ACCEPT_RANGES)
            .ok_or(Box::new(FetchError::ServerSupportError(
                "Server does not include Accept-Ranges header".to_owned(),
            )))?;

    if accept_ranges == "none" {
        return Err(Box::new(FetchError::ServerSupportError(
            "Server's Accept-Ranges header set to none".to_owned(),
        )));
    }

    let content_length = headers
        .get(CONTENT_LENGTH)
        .ok_or(Box::new(FetchError::ServerSupportError(
            "Server does not include Content-Length header".to_owned(),
        )))?
        .to_str()?
        .parse::<u64>()?;

    let mut fetches = Vec::new();

    let ranges = create_ranges(content_length, options.num_fetches)?;
    for range in ranges {
        fetches.push(fetch_range(
            &client,
            &options.url,
            range,
            &path,
            content_length,
            &options.logger,
        ));
    }

    try_join_all(fetches).await?;

    Ok(())
}

async fn fetch_range(
    client: &reqwest::Client,
    url: &str,
    range: Range,
    path: &PathBuf,
    total_length: u64,
    logger: &Logger,
) -> Result<(), Box<dyn Error>> {
    let out_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .await?;

    let mut writer = BufWriter::new(out_file);
    writer.seek(SeekFrom::Start(range.start)).await?;

    info!(logger, "fetching"; "range" => &range);

    let mut headers = HeaderMap::new();
    headers.insert(
        RANGE,
        format!("bytes={}-{}", range.start, range.end).parse()?,
    );

    let mut res = client
        .get(url)
        .headers(headers)
        .send()
        .await?
        .error_for_status()?;

    let res_headers = res.headers();

    let status = res.status();

    if status != StatusCode::PARTIAL_CONTENT {
        return Err(Box::new(FetchError::ServerSupportError(
            "Range response status code was not a 206".to_owned(),
        )));
    }

    let content_range = res_headers
        .get(CONTENT_RANGE)
        .ok_or(Box::new(FetchError::ServerSupportError(
            "Range response did not include Content-Range header".to_owned(),
        )))?
        .to_str()?;

    if content_range != format!("bytes {}-{}/{}", range.start, range.end, total_length) {
        return Err(Box::new(FetchError::ServerSupportError(
            "Range response Content-Range headers did not match expected".to_owned(),
        )));
    }

    let content_length = res_headers
        .get(CONTENT_LENGTH)
        .ok_or(Box::new(FetchError::ServerSupportError(
            "Range response did not include Content-Length header".to_owned(),
        )))?
        .to_str()?
        .parse::<u64>()?;

    if content_length - 1 != range.end - range.start {
        return Err(Box::new(FetchError::ServerSupportError(
            "Range response Content-Length was incorrect".to_owned(),
        )));
    }

    info!(logger, "received"; "range" => &range, "content_range" => &content_range, "content_length" => content_length, "status" => format!("{}", res.status()));

    while let Some(chunk) = res.chunk().await? {
        writer.write(&chunk).await?;
    }

    writer.flush().await?;

    info!(logger, "written"; "range" => &range, "path" => format!("{:?}", &path));

    Ok(())
}
