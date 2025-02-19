use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilationError {
    #[error("File IO error: {0}")]
    IOERR(#[from] io::Error),

    #[error("Assembling/Linking error: {0}")]
    ASMLINK(String),

    #[error("Parse error: {0}")]
    PARSE(String),

    #[error("Misc. error: {0}")]
    MISC(String),

    #[error("#error found in source code")]
    HASHERR
}
