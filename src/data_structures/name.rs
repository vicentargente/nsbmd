use std::fmt::Debug;

use crate::error::AppError;

#[derive(Clone)]
pub struct Name {
    pub name: [u8; 16]
}

impl Name {
    pub const SIZE: usize = 16;

    pub fn from_bytes(bytes: &[u8]) -> Result<Name, AppError> {
        if bytes.len() > 16 {
            return Err(AppError::new("Name needs at least 16 bytes"))
        }

        let mut name = [0; 16];
        name.copy_from_slice(&bytes[0..16]);

        Ok(Name {
            name
        })
    }

    pub fn from_string(name: &str) -> Result<Name, AppError> {
        let bytes = name.as_bytes();
        Self::from_bytes(bytes)
    }

    pub fn to_string(&self) -> Result<String, AppError> {
        let name = std::str::from_utf8(&self.name).map_err(|_| AppError::new("Invalid UTF-8 string"))?;
        Ok(name.to_string())
    }

    pub fn to_not_null_string(&self) -> Result<String, AppError> {
        let name = std::str::from_utf8(&self.name).map_err(|_| AppError::new("Invalid UTF-8 string"))?;
        Ok(name.trim_end_matches('\0').to_string())
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 16 {
            return Err(AppError::new("Name buffer needs at least 16 bytes"))
        }

        buffer[0..Self::SIZE].copy_from_slice(&self.name);

        Ok(())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, AppError> {
        Ok(self.name.to_vec())
    }
}

impl Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(name) = std::str::from_utf8(&self.name) {
            f.debug_struct("Name").field("name", &name).finish()
        } else {
            f.debug_struct("Name").field("name", &self.name).finish()
        }
    }
}
