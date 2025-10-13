// stubbing out

use std::io;

#[derive(Debug)]
pub enum RcwtError {
    Io(io::Error),
    Json(serde_json::Error),
    InvalidHeader,
    UnexpectedEOF,
    Eof
}

impl From<io::Error> for RcwtError {
    fn from(err: io::Error) -> RcwtError {
        RcwtError::Io(err)
    }
}

impl From<serde_json::Error> for RcwtError {
    fn from(err: serde_json::Error) -> RcwtError {
        RcwtError::Json(err)
    }
}

impl std::error::Error for RcwtError {}

impl std::fmt::Display for RcwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RcwtError::Eof => write!(f, "End of file"),
            RcwtError::Io(e) => write!(f, "I/O error: {}", e),
            RcwtError::InvalidHeader => write!(f, "Invalid RCWT header"),
            RcwtError::Json(e) => write!(f, "JSON error: {}", e),
            RcwtError::UnexpectedEOF => write!(f, "Unexpected end of file")
        }
    }
}
