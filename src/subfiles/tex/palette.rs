use std::fmt::Debug;

use crate::{data_structures::name_list::NameList, error::AppError, traits::BinarySerializable};

#[derive(Debug, Clone)]
pub struct PaletteList {
    palettes: NameList<Palette>
}

impl PaletteList {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, AppError> {
        let palette_list = NameList::<Palette>::from_bytes(bytes)?;

        Ok(PaletteList {
            palettes: palette_list,
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < self.palettes.size() {
            return Err(AppError::new("Buffer is too small to write PaletteList"));
        }

        self.palettes.write_bytes(buffer)
    }
}



#[derive(Debug, Clone)]
pub struct Palette {
    pltt_base: PlttBase,
    unknown: u16
}

impl Palette {
    const SIZE: usize = 4;
}

impl BinarySerializable for Palette {
    fn from_bytes(bytes: &[u8]) -> Result<Self, AppError> {
        if bytes.len() < Self::SIZE {
            return Err(AppError::new("Palette needs at least 4 bytes to start reading"));
        }

        let pltt_base = PlttBase::new(u16::from_le_bytes([bytes[0], bytes[1]]));
        let unknown = u16::from_le_bytes([bytes[2], bytes[3]]);

        Ok(Palette {
            pltt_base,
            unknown,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, AppError> {
        let mut buffer = vec![0; 4];
        self.write_bytes(&mut buffer)?;
        Ok(buffer)
    }

    fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < Self::SIZE {
            return Err(AppError::new("Buffer is too small to write Palette"));
        }

        buffer[0..2].copy_from_slice(&self.pltt_base.data.to_le_bytes());
        buffer[2..4].copy_from_slice(&self.unknown.to_le_bytes());

        Ok(())
    }

    fn size(&self) -> usize {
        Self::SIZE
    }
}



#[derive(Clone, Copy)]
pub struct PlttBase {
    data: u16
}

impl PlttBase {
    pub fn new(data: u16) -> Self {
        PlttBase { data }
    }

    pub fn palette_base(&self) -> u16 {
        self.data &0x1FFF
    }

    pub fn unused(&self) -> u8 {
        // Always 0?
        ((self.data >> 13) & 0x03) as u8
    }
}

impl Debug for PlttBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlttBase")
            .field("data", &self.data)
            .field("palette_base", &self.palette_base())
            .field("unused", &self.unused())
            .finish()
    }
}
