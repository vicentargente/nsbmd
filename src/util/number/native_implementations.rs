use crate::{error::AppError, traits::BinarySerializable};

impl BinarySerializable for u8 {
    fn from_bytes(bytes: &[u8]) -> Result<Self, AppError> {
        Ok(bytes[0])
    }

    fn to_bytes(&self) -> Result<Vec<u8>, AppError> {
        Ok(vec![*self])
    }
    
    fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        Ok(buffer[0] = *self)
    }

    fn size(&self) -> usize {
        1
    }
}

impl BinarySerializable for u16 {
    fn from_bytes(bytes: &[u8]) -> Result<Self, AppError> {
        if bytes.len() < 2 {
            return Err(AppError::new("u16 needs at least 2 bytes"))
        }

        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn to_bytes(&self) -> Result<Vec<u8>, AppError> {
        Ok(self.to_le_bytes().to_vec())
    }
    
    fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 2 {
            return Err(AppError::new("u16 needs at least 2 bytes"))
        }

        buffer[0..2].copy_from_slice(&self.to_le_bytes());

        Ok(())
    }

    fn size(&self) -> usize {
        2
    }
}

impl BinarySerializable for u32 {
    fn from_bytes(bytes: &[u8]) -> Result<Self, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("u32 needs at least 4 bytes"))
        }

        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn to_bytes(&self) -> Result<Vec<u8>, AppError> {
        Ok(self.to_le_bytes().to_vec())
    }
    
    fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("u32 needs at least 4 bytes"))
        }

        buffer[0..4].copy_from_slice(&self.to_le_bytes());

        Ok(())
    }

    fn size(&self) -> usize {
        4
    }
}

impl BinarySerializable for u64 {
    fn from_bytes(bytes: &[u8]) -> Result<Self, AppError> {
        if bytes.len() < 8 {
            return Err(AppError::new("u64 needs at least 8 bytes"))
        }

        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7]
        ]))
    }

    fn to_bytes(&self) -> Result<Vec<u8>, AppError> {
        Ok(self.to_le_bytes().to_vec())
    }
    
    fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 8 {
            return Err(AppError::new("u64 needs at least 8 bytes"))
        }

        buffer[0..8].copy_from_slice(&self.to_le_bytes());

        Ok(())
    }

    fn size(&self) -> usize {
        8
    }
}
