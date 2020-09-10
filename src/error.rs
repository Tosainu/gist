use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Box<Error> {
        Box::new(Error { kind })
    }
}

pub type Result<T> = std::result::Result<T, Box<Error>>;

#[derive(Debug)]
pub enum ErrorKind {
    Api {
        message: String,
    },
    ApiWithStatus {
        status: reqwest::StatusCode,
        message: String,
    },
    ConfigDirectoryNotDetected,
    InvalidConfigFormat {
        path: PathBuf,
        error: serde_json::Error,
    },
    SaveConfigFailure {
        path: PathBuf,
        error: serde_json::Error,
    },
    HttpClient(reqwest::Error),
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Api { message } => write!(f, "GitHub API returns error: {}", message),
            ErrorKind::ApiWithStatus { status, message } => write!(
                f,
                "GitHub API returns error with status {}: {}",
                status, message
            ),
            ErrorKind::ConfigDirectoryNotDetected =>
                write!(f, "Default configuration directory not detected. $HOME or $XDG_CONFIG_FIR may not set"),
            ErrorKind::InvalidConfigFormat { path, error } =>
                write!(f,"Cannot parse configuration file '{}': ", path.display()).and_then(move |_| error.fmt(f)),
            ErrorKind::SaveConfigFailure { path, error } =>
                write!(f,"Failed to save configuration file '{}': ", path.display()).and_then(move |_| error.fmt(f)),
            ErrorKind::HttpClient(e) => e.fmt(f),
            ErrorKind::Io(e) => e.fmt(f),
        }
    }
}

impl From<reqwest::Error> for Box<Error> {
    fn from(e: reqwest::Error) -> Box<Error> {
        Error::new(ErrorKind::HttpClient(e))
    }
}

impl From<std::io::Error> for Box<Error> {
    fn from(e: std::io::Error) -> Box<Error> {
        Error::new(ErrorKind::Io(e))
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::HttpClient(e) => Some(e),
            ErrorKind::Io(e) => Some(e),
            _ => None,
        }
    }
}
