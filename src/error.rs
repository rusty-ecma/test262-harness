use std::path::PathBuf;
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    WalkDir(walkdir::Error),
    Yaml(serde_yaml::Error),
    Regex(regex::Error),
    DescriptionInvalid(PathBuf),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(e) => e.fmt(f),
            Error::WalkDir(e) => e.fmt(f),
            Error::Yaml(e) => e.fmt(f),
            Error::Regex(e) => e.fmt(f),
            Error::DescriptionInvalid(path) => {
                write!(f, "Unable to extract description for {:?}", path)
            }
        }
    }
}
impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(other: std::io::Error) -> Self {
        Self::Io(other)
    }
}

impl From<walkdir::Error> for Error {
    fn from(other: walkdir::Error) -> Self {
        Self::WalkDir(other)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(other: serde_yaml::Error) -> Self {
        Self::Yaml(other)
    }
}

impl From<regex::Error> for Error {
    fn from(other: regex::Error) -> Self {
        Self::Regex(other)
    }
}
