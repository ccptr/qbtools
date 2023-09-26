use std::{
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;

#[rustfmt::skip] // for match expression
fn deserialize<S: AsRef<OsStr> + ?Sized, T: DeserializeOwned>(path: &S) -> Result<T, Error> {
    let path = Path::new(path);

    let data = std::fs::read_to_string(path)?;

    Ok(match get_extension(path).as_str() {
        "json"         => serde_json::from_str(&data)?,
        #[cfg(feature = "toml")]
        "toml"         =>       toml::from_str(&data)?,
        #[cfg(feature = "yaml")]
        "yaml" | "yml" => serde_yaml::from_str(&data)?,
        _ => panic!("programming error?"),
    })
}

/// TODO: fix this once `const VAR: [type; _] = ["...", "..."];` is stable
#[cfg(feature = "toml")]
pub const SUPPORTED_CONFIG_TYPES: [&str; 3] = ["yaml", "json", "toml"];
#[cfg(not(feature = "toml"))]
pub const SUPPORTED_CONFIG_TYPES: [&str; 2] = ["yaml", "json"];

/// # Example
/// ```
/// let files = fs::get_possible_files("foo/bar");
///    assert_eq!(files, "foo/bar.{json,toml,yaml}".to_string());
///    // AKA
///    assert_eq!(files, format!("foo/bar.{{{}}}", SUPPORTED_CONFIG_TYPES.join(",")));
/// ```
///
pub fn get_possible_files(base_path: &str) -> String {
    format!("{}.{{{}}}", base_path, SUPPORTED_CONFIG_TYPES.join(","))
}

/// path: path without extension (eg. data/products)
pub fn read_config<T>(base_path: &Path) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    deserialize(&get_first_file(base_path))
}

pub fn get_extension<S>(path: &S) -> String
where
    S: AsRef<OsStr> + ?Sized,
{
    Path::new(path)
        .extension()
        .expect("programming error?")
        .to_string_lossy()
        .to_string()
}

/// Returns the first data file that exists, defaulting to `base-path.json` if none exist. Lookup
/// order:
pub fn get_first_file(base_path: &Path) -> PathBuf {
    let json_file = util::append(base_path, ".json");

    #[cfg(feature = "toml")]
    let toml_file = util::append(base_path, ".toml");

    #[cfg(feature = "yaml")]
    let yaml_file = util::append(base_path, ".yaml");
    #[cfg(feature = "yaml")]
    let yml_file = util::append(base_path, ".yml");

    if json_file.is_file() {
        return json_file;
    }
    #[cfg(feature = "toml")]
    if toml_file.is_file() {
        return toml_file;
    }
    #[cfg(feature = "yaml")]
    if yaml_file.is_file() {
        return yaml_file;
    }
    #[cfg(feature = "yaml")]
    if yml_file.is_file() {
        return yml_file;
    }

    // return default (JSON) file if none exists
    json_file
}

pub mod util {
    use std::{
        ffi::{OsStr, OsString},
        path::PathBuf,
    };

    /// Returns a path with a new extension component appended to the end.
    /// # Example
    /// ```
    /// use util::append;
    /// use std::path::{Path, PathBuf};
    ///
    /// let path = Path::new("foo/bar/baz.txt");
    /// assert_eq!(append(path, ".app"), PathBuf::from("foo/bar/baz.txt.app"));
    /// ```
    pub fn append<P: AsRef<OsStr> + ?Sized>(path: &P, ext: impl AsRef<OsStr>) -> PathBuf {
        let mut os_string: OsString = path.into();

        os_string.push(ext.as_ref());
        os_string.into()
    }
}

#[derive(Debug)]
pub enum Error {
    Deserialize(Option<&'static str>),
    IO(io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deserialize(msg) => match msg {
                Some(msg) => write!(f, "Deserialize(\"{}\")", msg),
                None => write!(f, "Deserialize(None)"),
            },
            Self::IO(error) => {
                write!(f, "IO({})", error)
            }
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IO(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        use serde_json::error::Category;

        Self::Deserialize(match error.classify() {
            Category::Io => Some("failure to read or write bytes on an IO stream"),
            Category::Syntax => Some("input that is not syntactically valid JSON"),
            Category::Data => Some("input data that is semantically incorrect"),
            Category::Eof => Some("unexpected end of the input data"),
        })
    }
}

#[cfg(feature = "toml")]
impl From<toml::de::Error> for Error {
    fn from(_error: toml::de::Error) -> Self {
        Self::Deserialize(None)
    }
}

#[cfg(feature = "yaml")]
impl From<serde_yaml::Error> for Error {
    fn from(_error: serde_yaml::Error) -> Self {
        Self::Deserialize(None)
    }
}
