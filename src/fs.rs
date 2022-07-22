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
        #[cfg(feature = "toml")]
        "toml"         =>       toml::from_str(&data)?,
        "yaml" | "yml" => serde_yaml::from_str(&data)?,
        "json"         => serde_json::from_str(&data)?,
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
///    assert_eq!(files, "foo/bar.{toml,yaml,json}".to_string());
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
    return deserialize(&get_first_file(base_path));
}

pub fn get_extension<S>(path: &S) -> String
where
    S: AsRef<OsStr> + ?Sized,
{
    let path = Path::new(path);
    path.extension()
        .expect("programming error?")
        .to_string_lossy()
        .to_string()
}

/// Returns the first data file that exists, defaulting to `base_path.yml`. Follows (Go)Hugo's lookup order.
pub fn get_first_file(base_path: &Path) -> PathBuf {
    #[cfg(feature = "toml")]
    let toml_file = util::append(base_path, ".toml");

    let yaml_file = util::append(base_path, ".yaml");
    let yml_file = util::append(base_path, ".yml");

    let json_file = util::append(base_path, ".json");

    #[cfg(feature = "toml")]
    if toml_file.is_file() {
        return toml_file;
    }
    if yaml_file.is_file() {
        return yaml_file;
    }
    if yml_file.is_file() {
        return yml_file;
    }
    if json_file.is_file() {
        return json_file;
    }

    // return default (YAML) file if none exists
    return yml_file;
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
    /// use std::path::PathBuf;
    /// let path = PathBuf::from("foo/bar/baz.txt");
    ///    assert_eq!(append(path, ".app"), PathBuf::from("foo/bar/baz.txt.app"));
    /// ```
    ///
    pub fn append<S: AsRef<OsStr> + ?Sized>(path: &S, ext: impl AsRef<OsStr>) -> PathBuf {
        let mut os_string: OsString = path.into();

        os_string.push(ext.as_ref());

        return os_string.into();
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
                Some(msg) => return write!(f, "Deserialize(\"{}\")", msg),
                None => return write!(f, "Deserialize(None)"),
            },
            Self::IO(error) => {
                return write!(f, "IO({})", error);
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

impl From<serde_yaml::Error> for Error {
    fn from(_error: serde_yaml::Error) -> Self {
        Self::Deserialize(None)
    }
}

#[cfg(feature = "toml")]
impl From<toml::de::Error> for Error {
    fn from(_error: toml::de::Error) -> Self {
        Self::Deserialize(None)
    }
}
