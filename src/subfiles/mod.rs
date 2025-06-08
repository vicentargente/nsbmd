use crate::error::AppError;

pub mod mdl;
pub mod tex;
pub mod jnt;
pub mod pat;
pub mod srt;

#[derive(Debug, Clone, Copy)]
pub enum Type {
    MDL,
    TEX,
    JNT,
    PAT,
    SRT
}

impl Type {
    pub fn from_stamp(stamp: &[u8]) -> Result<Type, AppError> {
        if stamp.len() != 4 {
            return Err(AppError::new(&format!("Invalid stamp length: {}", stamp.len())))
        }

        match stamp {
            b"MDL0" => Ok(Type::MDL),
            b"TEX0" => Ok(Type::TEX),
            b"JNT0" => Ok(Type::JNT),
            b"PAT0" => Ok(Type::PAT),
            b"SRT0" => Ok(Type::SRT),
            _ => Err(AppError::new(&format!("Unknown subfile type for stamp: {:?}", stamp)))
        }
    }
}
