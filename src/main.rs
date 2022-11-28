#![allow(clippy::needless_return)]

mod args;
mod commands;
mod config;
mod fs;

use args::*;
use clap::Parser;
use commands::export::ExportItemsArgs;

use std::process::exit;

use quickbooks_ureq::{config::QuickbooksConfig, constants::base_url};

const BASE_CONFIG_PATH: &str = "qb-api-cfg";

#[cfg(feature = "production")]
const QB_BASE_URL: &str = base_url::PRODUCTION;
#[cfg(not(feature = "production"))]
const QB_BASE_URL: &str = base_url::SANDBOX;

fn main() {
    env_logger::init();
    let args = Args::parse();

    match args.command {
        #[cfg(feature = "export")]
        Command::Export { command } => match command {
            ExportCommand::Items {
                output_path,
                format,
            } => commands::export::items(&ExportItemsArgs {
                quiet: args.quiet,
                verbose: args.verbose,
                output_path,
                format,
            }),
        },
    }
}
