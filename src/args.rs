use std::str::FromStr;

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
    #[cfg(feature = "export")]
    Export {
        #[clap(subcommand)]
        command: ExportCommand,
    },
}

#[cfg(feature = "export")]
#[derive(Debug, PartialEq, Subcommand)]
pub enum ExportCommand {
    Items {
        #[clap(default_value_t, short, long)]
        format: OutputFormat,
        #[clap(short, long, parse(from_os_str))]
        output_path: Option<std::path::PathBuf>,
    },
}

#[derive(Debug, PartialEq)]
pub enum OutputFormat {
    Json,
    #[cfg(feature = "toml")]
    Toml,
    #[cfg(feature = "yaml")]
    Yaml,
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
            "toml" => Ok(Self::Toml),
            "yaml" => Ok(Self::Yaml),
            _ => Err("Could not parse format (OutputFormat)"),
        }
    }
}

impl ToString for OutputFormat {
    fn to_string(&self) -> String {
        match self {
            Self::Json => "json".to_string(),
            Self::Toml => "toml".to_string(),
            Self::Yaml => "yaml".to_string(),
        }
    }
}
