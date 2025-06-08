use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Pat {}

impl Pat {
    pub fn from_bytes(_bytes: &[u8]) -> Result<Pat, AppError> {
        Ok(Pat {})
    }
}
