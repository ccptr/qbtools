#[cfg(feature = "cmd-export")]
pub mod export;
#[cfg(feature = "cmd-get")]
pub mod get;

use quickbooks_ureq::config::QueryConfig;

use crate::{args::OutputFormat, config::get_authorized_qb};

use std::{fs::File, io, path::PathBuf, slice};

#[derive(Debug)]
pub enum CommandError {
    /// failed to serialize initial response into a `serde_json::Value`
    FailedToSerializeResponse(std::io::Error),

    QbUreq(quickbooks_ureq::Error),
    OutputError(OutputError),
}

impl From<quickbooks_ureq::Error> for CommandError {
    fn from(error: quickbooks_ureq::Error) -> Self {
        Self::QbUreq(error)
    }
}

impl From<OutputError> for CommandError {
    fn from(error: OutputError) -> Self {
        Self::OutputError(error)
    }
}

impl From<std::io::Error> for CommandError {
    fn from(error: std::io::Error) -> Self {
        Self::OutputError(OutputError::Io(error))
    }
}

#[derive(Debug)]
pub enum SerializationError {
    Json(serde_json::Error),
    #[cfg(feature = "toml")]
    Toml(toml::ser::Error),
    #[cfg(feature = "yaml")]
    Yaml(serde_yaml::Error),
}

impl From<serde_json::Error> for SerializationError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[cfg(feature = "toml")]
impl From<toml::ser::Error> for SerializationError {
    fn from(error: toml::ser::Error) -> Self {
        Self::Toml(error)
    }
}

#[cfg(feature = "yaml")]
impl From<serde_yaml::Error> for SerializationError {
    fn from(error: serde_yaml::Error) -> Self {
        Self::Yaml(error)
    }
}

#[derive(Debug)]
pub enum OutputError {
    Io(io::Error),
    Serialization(SerializationError),
}

impl From<io::Error> for OutputError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<SerializationError> for OutputError {
    fn from(error: SerializationError) -> Self {
        Self::Serialization(error)
    }
}

impl From<serde_json::Error> for OutputError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(SerializationError::Json(error))
    }
}

#[cfg(feature = "toml")]
impl From<toml::ser::Error> for OutputError {
    fn from(error: toml::ser::Error) -> Self {
        Self::Serialization(SerializationError::Toml(error))
    }
}

#[cfg(feature = "yaml")]
impl From<serde_yaml::Error> for OutputError {
    fn from(error: serde_yaml::Error) -> Self {
        Self::Serialization(SerializationError::Yaml(error))
    }
}

fn process_response_for_desired_array(
    response: quickbooks_ureq::Response,
    key: &str,
) -> Result<serde_json::Value, CommandError> {
    let mut response: serde_json::Value = response
        .into_json()
        .map_err(|err| CommandError::FailedToSerializeResponse(err))?;

    let values = response
        .get_mut("QueryResponse")
        .expect("to be guaranteed by the QB API");

    let values = values.get_mut(key).expect("programming error");

    Ok(values.take())
}

/// returns QB API response as array of items
fn get_desired_array(
    quiet: bool,
    key: &str,
    options: &QueryConfig,
) -> Result<serde_json::Value, CommandError> {
    let qb = get_authorized_qb(quiet)?;
    let response = qb.query(key, options)?;
    process_response_for_desired_array(response, key)
}

/// serializes `value` to `output_path` (or stdout if None) as `format`
pub fn to_output_path<T>(
    value: &T,
    output_path: &Option<PathBuf>,
    format: &OutputFormat,
    pretty: bool,
) -> Result<(), OutputError>
where
    T: ?Sized + serde::ser::Serialize,
{
    if let Some(output_path) = output_path {
        // TODO: change map_err to inspect_err once stable
        let file = File::create(output_path).map_err(|err| {
            log::error!("{err}");
            err
        })?;

        to_writer(file, value, format, pretty)?;
    } else {
        let writer = std::io::stdout();
        to_writer(writer, value, format, pretty)?;
    }

    Ok(())
}

pub(crate) fn to_writer<W, T>(
    mut writer: W,
    value: &T,
    format: &OutputFormat,
    pretty: bool,
) -> Result<(), OutputError>
where
    W: std::io::Write,
    T: ?Sized + serde::ser::Serialize,
{
    fn expect<T, E>(result: Result<T, E>, format: &OutputFormat) -> Result<T, OutputError>
    where
        E: std::fmt::Debug,
        OutputError: From<E>,
    {
        match result {
            Ok(result) => Ok(result),
            Err(err) => {
                log::error!(
                    "failed to serialize/write value to output_path as {}: {err:?}",
                    format.as_str()
                );
                return Err(OutputError::from(err));
            }
        }
    }

    match format {
        OutputFormat::Json => expect(
            if !pretty {
                serde_json::to_writer
            } else {
                serde_json::to_writer_pretty
            }(&mut writer, value),
            format,
        )?,
        #[cfg(feature = "toml")]
        OutputFormat::Toml => expect(
            writer.write_all(
                match toml::to_string(value) {
                    Ok(toml_str) => toml_str,
                    Err(err) => {
                        log::error!("failed to serialize value to {}: {err}", format.as_str());
                        return Err(err)?;
                    }
                }
                .as_bytes(),
            ),
            format,
        )?,
        #[cfg(feature = "yaml")]
        OutputFormat::Yaml => expect(serde_yaml::to_writer(&mut writer, value), format)?,
    }

    if let Err(err) = writer.write_all(slice::from_ref(&b'\n')) {
        log::error!("failed to write a new line to writer: {err}");
        return Err(err)?;
    }

    Ok(())
}

fn we_do_a_bit_of_logging(values: &Vec<serde_json::Value>, key: &str) {
    log::trace!("{key}: {values:?}");
    log::info!("number of {key}s: {}", values.len());

    if values.len() == 1000 {
        log::error!("number of {key}s is equal to 1,000; if you have more than 1,000 {key}s, the response has been reduced to 1,000 {key}s");
    }
}
