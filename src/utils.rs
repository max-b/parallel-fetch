use std::error::Error;
use std::path::{Path, PathBuf};

use reqwest::Url;

use crate::errors::FetchError;
use crate::fetch::Range;

/// Takes an optional output and a url to download from
/// and returns an output path to write to
pub fn parse_path(output_option: Option<String>, url: &str) -> Result<PathBuf, Box<dyn Error>> {
    let parsed_url = Url::parse(url).unwrap();

    let segments = parsed_url.path_segments();

    let default_filename = "index.html";
    let mut url_filename = if let Some(mut segments) = segments {
        segments.next_back().unwrap_or(default_filename)
    } else {
        default_filename
    };

    if url_filename == "" {
        url_filename = default_filename;
    }

    let mut output_path = if let Some(o) = output_option {
        Path::new(&o).to_path_buf()
    } else {
        Path::new("./").to_path_buf()
    };

    // If the path is a directory, the filename
    // comes from the url
    if output_path.is_dir() {
        output_path.push(url_filename);
    } else {
        // If path is not a directory, ensure that
        // parent *is*
        match output_path.parent() {
            None => {
                return Err(Box::new(FetchError::InvalidArgumentsError(
                    "Output argument invalid".to_owned(),
                )));
            }
            Some(p) => {
                if !p.is_dir() {
                    return Err(Box::new(FetchError::InvalidArgumentsError(
                        "Output argument invalid".to_owned(),
                    )));
                }
            }
        }
    }

    // TODO: check if file already exists?
    Ok(output_path)
}

/// Takes a content_length and num_fetches
/// and returns a Vec<Range> which covers the content_length and where result.len() ==
/// num_fetches
/// TODO: "unit" test this
pub fn create_ranges(content_length: u64, num_fetches: u64) -> Vec<Range> {
    let mut cursor = 0;
    let mut ranges = Vec::new();

    // integer division returns a floor
    let step_size = content_length / num_fetches;

    for i in 0..num_fetches {
        let end = if i == num_fetches - 1 {
            content_length - 1
        } else {
            cursor + step_size - 1
        };

        let range = Range { start: cursor, end };

        cursor = end + 1;
        ranges.push(range);
    }

    ranges
}
