use std::{path::PathBuf, str::FromStr};

use clap::{Parser, Subcommand};

#[derive(Debug, Parser, PartialEq)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,

    #[clap(short, long, help = "do not print company info on start, etc.")]
    pub quiet: bool,

    #[clap(
        short,
        long,
        help = "print additional information (useful for debugging)"
    )]
    pub verbose: bool,
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum Command {
    #[cfg(feature = "cmd-export")]
    Export {
        #[arg(short, long, default_value = "json")]
        format: Option<OutputFormat>,
        #[arg(short, long)]
        output_path: Option<PathBuf>,
        #[arg(long)]
        pretty: bool,

        #[clap(subcommand)]
        command: ExportCommands,
    },
    #[cfg(feature = "cmd-get")]
    Get {
        #[arg(short, long, default_value = "json")]
        format: Option<OutputFormat>,
        #[arg(long)]
        id: String,
        #[arg(short, long)]
        output_path: Option<PathBuf>,

        #[clap(subcommand)]
        command: GetCommands,
    },
}

#[cfg(feature = "cmd-export")]
#[derive(Debug, PartialEq, Subcommand)]
pub enum ExportCommands {
    Customers(ExportCustomerArgs),
    Items,
}

#[cfg(feature = "cmd-export")]
#[derive(Debug, PartialEq, Subcommand)]
pub enum GetCommands {
    Customer,
    Item,
}

#[derive(clap::Args, Debug, PartialEq)]
pub struct ExportCustomerArgs {
    #[arg(long)]
    pub r#where: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OutputFormat {
    Json,
    #[cfg(feature = "toml")]
    Toml,
    #[cfg(feature = "yaml")]
    Yaml,
}

impl OutputFormat {
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Json => "json",
            #[cfg(feature = "toml")]
            Self::Toml => "toml",
            #[cfg(feature = "yaml")]
            Self::Yaml => "yaml",
        }
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Json
    }
}

impl FromStr for OutputFormat {
    // any error type implementing Display is acceptable.
    type Err = &'static str;

    fn from_str(format: &str) -> Result<Self, Self::Err> {
        match format {
            "json" => Ok(Self::Json),
            #[cfg(feature = "toml")]
            "toml" => Ok(Self::Toml),
            #[cfg(feature = "yaml")]
            "yaml" => Ok(Self::Yaml),
            _ => Err("Could not parse output format"),
        }
    }
}

impl ToString for OutputFormat {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}
