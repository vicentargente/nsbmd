use crate::error::AppError;

pub trait BinarySerializable: Sized {
    fn from_bytes(bytes: &[u8]) -> Result<Self, AppError>;
    fn to_bytes(&self) -> Result<Vec<u8>, AppError>;
    fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError>;
    fn size(&self) -> usize;
}
