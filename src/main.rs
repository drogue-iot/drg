pub mod admin;
pub mod applications;
pub mod arguments;
pub mod command;
pub mod config;
pub mod devices;
pub mod lib;
pub mod openid;
pub mod stream;
pub mod util;

use crate::arguments::cli::Parameters;
use anyhow::Result;
use lib::interactive_mode;
use lib::process_arguments;
use std::process::exit;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = arguments::cli::app_arguments().get_matches();

    simple_logger::SimpleLogger::new()
        .with_level(util::log_level(&matches))
        .init()
        .unwrap();

    let code = if matches.is_present(Parameters::interactive.as_ref()) {
        interactive_mode();
        0
    } else {
        process_arguments(matches).await?
    };

    exit(code)
}
