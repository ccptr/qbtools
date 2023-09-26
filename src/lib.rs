mod args;
mod commands;
mod config;
mod fs;

#[cfg(feature = "cmd-export")]
use commands::export::ExportArgs;
#[cfg(feature = "cmd-export")]
use std::ffi::OsString;

#[cfg(feature = "cmd-get")]
use commands::get::GetArgs;

mod wrapper {
    #![allow(unused_imports)]
    use super::*;

    pub use args::*;
    pub use clap::Parser;
    pub use die_exit::*;
}

#[cfg(any(feature = "cmdline", feature = "interactive"))]
use wrapper::*;

use quickbooks_ureq::{config::QuickbooksConfig, constants::base_url};

use std::process::exit;

const BASE_CONFIG_PATH: &str = "qb-api-cfg";

#[cfg(feature = "production")]
const QB_BASE_URL: &str = base_url::PRODUCTION;
#[cfg(not(feature = "production"))]
const QB_BASE_URL: &str = base_url::SANDBOX;

#[cfg(feature = "interactive")]
pub fn main_interactive(_args: impl Iterator<Item = OsString>) {
    println!(
        "qbtools version {}, running interactively",
        env!("CARGO_PKG_VERSION")
    );
}

#[cfg(feature = "cmdline")]
pub fn main_cmdline(args: impl Iterator<Item = OsString>) {
    let args = Args::parse_from(args);

    match args.command {
        #[cfg(feature = "cmd-export")]
        Command::Export {
            command,
            format,
            output_path,
            pretty,
        } => {
            let args = ExportArgs {
                format,
                output_path,
                pretty,
                quiet: args.quiet,
                verbose: args.verbose,
            };

            match command {
                ExportCommands::Customers(c_args) => commands::export::customers(&args, &c_args)
                    .die_with(|err| (1, format!("failed to export customers: {err:?}"))),
                ExportCommands::Items => commands::export::items(&args)
                    .die_with(|err| (1, format!("failed to export items: {err:?}"))),
            }
        }
        #[cfg(feature = "cmd-get")]
        Command::Get {
            format,
            id,
            output_path,
            command,
        } => {
            let get_args = GetArgs {
                format,
                id,
                output_path,
                quiet: args.quiet,
            };

            match command {
                GetCommands::Customer => commands::get::customer(&get_args)
                    .die_with(|err| (1, format!("failed to get customer: {err:?}"))),
                GetCommands::Item => todo!(),
            }
        }
    }
}
