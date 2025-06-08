use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Jnt {}

impl Jnt {
    pub fn from_bytes(_bytes: &[u8]) -> Result<Jnt, AppError> {
        Ok(Jnt {})
    }
}
