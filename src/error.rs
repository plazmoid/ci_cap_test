#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Stream request parse error: {0}")]
    ParseError(String),
}
