use crate::{error::AppError, util::number::fixed_point::fixed_1_3_12::Fixed1_3_12};

#[derive(Debug, Clone)]
pub struct BoundingBox {
    x: Fixed1_3_12,
    y: Fixed1_3_12,
    z: Fixed1_3_12,

    w: Fixed1_3_12,
    h: Fixed1_3_12,
    d: Fixed1_3_12
}

impl BoundingBox {
    pub const SIZE: usize = 12;

    pub fn from_bytes(bytes: &[u8]) -> Result<BoundingBox, AppError> {
        if bytes.len() < BoundingBox::SIZE {
            return Err(AppError::new("Bounding box needs at least 12 bytes"))
        }

        let x = Fixed1_3_12::from(i16::from_le_bytes([bytes[0], bytes[1]]));
        let y = Fixed1_3_12::from(i16::from_le_bytes([bytes[2], bytes[3]]));
        let z = Fixed1_3_12::from(i16::from_le_bytes([bytes[4], bytes[5]]));

        let w = Fixed1_3_12::from(i16::from_le_bytes([bytes[6], bytes[7]]));
        let h = Fixed1_3_12::from(i16::from_le_bytes([bytes[8], bytes[9]]));
        let d = Fixed1_3_12::from(i16::from_le_bytes([bytes[10], bytes[11]]));

        Ok(BoundingBox {
            x,
            y,
            z,
            w,
            h,
            d
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < BoundingBox::SIZE {
            return Err(AppError::new("Bounding box needs at least 12 bytes to write"));
        }

        buffer[0..2].copy_from_slice(&self.x.to_i16().to_le_bytes());
        buffer[2..4].copy_from_slice(&self.y.to_i16().to_le_bytes());
        buffer[4..6].copy_from_slice(&self.z.to_i16().to_le_bytes());

        buffer[6..8].copy_from_slice(&self.w.to_i16().to_le_bytes());
        buffer[8..10].copy_from_slice(&self.h.to_i16().to_le_bytes());
        buffer[10..12].copy_from_slice(&self.d.to_i16().to_le_bytes());

        Ok(())
    }
}
