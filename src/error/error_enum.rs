use thiserror::Error;

#[derive(Error, Debug)]
#[error("RequestError")]
pub enum RequestError {
    #[error("Failed to parse url")]
    ParseUrlError,

    #[error("Failed to parse header: {0}")]
    ParseHeaderError(String),
}
