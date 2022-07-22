#[cfg(feature = "export")]
pub mod export;

use crate::args::OutputFormat;

use std::{fs::File, path::PathBuf, process::exit};

/// serializes `value` to `output_path` (or stdout if None) as `format`
pub fn to_output_path<T>(value: &T, output_path: &Option<PathBuf>, format: &OutputFormat)
where
    T: ?Sized + serde::ser::Serialize,
{
    fn expect<T, E>(result: Result<T, E>, format: &OutputFormat) -> T
    where
        E: std::fmt::Debug,
    {
        result.unwrap_or_else(|err| {
            panic!(
                "failed to serialize value to output_path as {}: {:?}",
                format.to_string(),
                err
            )
        })
    }

    fn to_writer<W, T>(mut writer: W, value: &T, format: &OutputFormat)
    where
        W: std::io::Write,
        T: ?Sized + serde::ser::Serialize,
    {
        match format {
            OutputFormat::Json => expect(serde_json::to_writer(writer, value), format),
            OutputFormat::Toml => expect(
                writer.write_all(
                    toml::to_string(value)
                        .unwrap_or_else(|err| {
                            panic!(
                                "failed to serialize value to {}: {}",
                                format.to_string(),
                                err
                            )
                        })
                        .as_bytes(),
                ),
                format,
            ),
            OutputFormat::Yaml => expect(serde_yaml::to_writer(writer, value), format),
        };
    }

    match output_path {
        Some(output_path) => match File::create(output_path) {
            Ok(file) => to_writer(file, value, format),
            Err(error) => {
                log::error!("failed to create output file: {}", error);
                exit(1);
            }
        },
        None => {
            let writer = std::io::stdout();
            to_writer(writer, value, format);
        }
    }
}
