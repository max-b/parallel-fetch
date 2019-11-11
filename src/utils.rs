use std::error::Error;
use std::path::{Path, PathBuf};
use std::{fs, io};

use hex;
use md5::{Digest, Md5};
use reqwest::Url;

use crate::errors::FetchError;
use crate::fetch::Range;

/// Check a ETag in the form of a md5 hash hex string
/// against a file at path location
pub fn check_etag(etag: &str, path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Md5::new();
    let _n = io::copy(&mut file, &mut hasher)?;
    let hash = hasher.result();
    if &hex::decode(&etag)?[..] == &hash[..] {
        Ok(())
    } else {
        Err(Box::new(FetchError::ValidationError(
            "ETag does not match".to_owned(),
        )))
    }
}

/// Takes an optional output and a url to download from
/// and returns an output path to write to
pub fn parse_path(output_option: &Option<String>, url: &str) -> Result<PathBuf, Box<dyn Error>> {
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
pub fn create_ranges(content_length: u64, num_fetches: u64) -> Result<Vec<Range>, Box<dyn Error>> {
    if num_fetches == 0 {
        return Err(Box::new(FetchError::InvalidArgumentsError(
            "Number of fetches must be greater than zero".to_owned(),
        )));
    }
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

    Ok(ranges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_with_0_chunks() {
        let ranges = create_ranges(100, 0);
        let error = ranges.expect_err("testing");
        assert_eq!(
            error.description(),
            "Number of fetches must be greater than zero".to_owned(),
        );
    }

    #[test]
    fn range_with_2_chunks() {
        let ranges = create_ranges(100, 2).unwrap();

        assert_eq!(
            ranges,
            vec![Range { start: 0, end: 49 }, Range { start: 50, end: 99 }]
        );
    }

    #[test]
    fn range_with_uneven_chunks() {
        let ranges = create_ranges(10, 3).unwrap();

        assert_eq!(
            ranges,
            vec![
                Range { start: 0, end: 2 },
                Range { start: 3, end: 5 },
                Range { start: 6, end: 9 }
            ]
        );
    }

    #[test]
    fn parse_path_with_none_output_option() {
        let url = "https://test.com/big-image.jpg";
        let path = parse_path(&None, url).unwrap();

        assert_eq!(path, PathBuf::from("./big-image.jpg"));
    }

    #[test]
    fn parse_path_with_none_output_option_and_no_url_filename() {
        let url = "https://test.com/";
        let path = parse_path(&None, url).unwrap();

        assert_eq!(path, PathBuf::from("./index.html"));
    }

    #[test]
    fn parse_path_with_non_existent_output_option_dir() {
        let url = "https://test.com/";
        // I posit this will never exist on a test environment
        let output_option = Some("/tmp/fake/fake/fake/fake".to_owned());
        let path = parse_path(&output_option, url);

        let error = path.expect_err("testing");
        assert_eq!(error.description(), "Output argument invalid".to_owned(),);
    }

    #[test]
    fn parse_path_with_output_option_dir() {
        let url = "https://test.com/big-image.jpg";
        let output_option = Some("/tmp".to_owned());
        let path = parse_path(&output_option, url).unwrap();

        assert_eq!(path, PathBuf::from("/tmp/big-image.jpg"));
    }

    #[test]
    fn parse_path_with_output_option_file() {
        let url = "https://test.com/big-image.jpg";
        let output_option = Some("/tmp/my-big-image.jpg".to_owned());
        let path = parse_path(&output_option, url).unwrap();

        assert_eq!(path, PathBuf::from("/tmp/my-big-image.jpg"));
    }
}
