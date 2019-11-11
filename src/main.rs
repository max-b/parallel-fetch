use std::error::Error;

use clap::{value_t, App, Arg};
use slog::{error, info};
use sloggers::terminal::TerminalLoggerBuilder;
use sloggers::types::Severity;
use sloggers::Build;

use parallel_fetch::{fetch, FetchOptions};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut builder = TerminalLoggerBuilder::new();
    builder.level(Severity::Info);

    let logger = builder.build().unwrap();
    info!(logger, "starting"; "version" => env!("CARGO_PKG_VERSION"));

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("url")
                .short("u")
                .long("url")
                .help("url to download")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .help("file output location")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("fetches")
                .short("n")
                .long("fetches")
                .help("the number of parallel fetches to execute, defaults to 10")
                .takes_value(true),
        )
        .get_matches();

    // unwrap is safe because url is required
    let url = matches.value_of("url").unwrap().to_owned();

    let output_option = matches.value_of("output").map(String::from);

    let num_fetches = value_t!(matches.value_of("fetches"), u64).unwrap_or(10);

    let options = FetchOptions {
        url,
        output_option,
        num_fetches,
        logger: logger.clone(),
    };

    match fetch(options).await {
        Ok(_) => {
            info!(logger, "Successfully downloaded");
            Ok(())
        }
        Err(err) => {
            error!(logger, "download failed"; "error" => err.description());
            Err(err)
        }
    }
}
