use std::fmt;

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
        status: reqwest::StatusCode,
        message: String,
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
            ErrorKind::Api { status, message } => {
                write!(f, "GitHub API returns error {}: {}", status, message)
            }
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
