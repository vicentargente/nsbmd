use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Srt {}

impl Srt {
    pub fn from_bytes(_bytes: &[u8]) -> Result<Srt, AppError> {
        Ok(Srt {})
    }
}
