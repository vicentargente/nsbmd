use std::fmt::Debug;

use crate::{data_structures::{name::Name, name_list::NameList}, error::AppError, traits::BinarySerializable};

#[derive(Debug, Clone)]
pub struct TextureList {
    textures: NameList<Texture>
}

impl TextureList {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, AppError> {
        let texture_list = NameList::<Texture>::from_bytes(bytes)?;

        Ok(TextureList {
            textures: texture_list,
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < self.textures.size() {
            return Err(AppError::new("Buffer is too small to write TextureList"));
        }

        self.textures.write_bytes(buffer)
    }

    pub fn get_texture(&self, index: usize) -> Option<&Texture> {
        self.textures.get(index)
    }

    pub fn get_texture_mut(&mut self, index: usize) -> Option<&mut Texture> {
        self.textures.get_mut(index)
    }

    pub fn get_texture_name(&self, index: usize) -> Option<&Name> {
        self.textures.get_name(index)
    }

    pub fn size(&self) -> usize {
        self.textures.size()
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    teximage_params: TeximageParams,
    width_height: WidthHeight
}

impl Texture {
    const SIZE: usize = 8;

    pub fn width(&self) -> u16 {
        self.width_height.width()
    }

    pub fn height(&self) -> u16 {
        self.width_height.height()
    }
}

impl BinarySerializable for Texture {
    fn from_bytes(bytes: &[u8]) -> Result<Self, AppError> {
        if bytes.len() < 8 {
            return Err(AppError::new("Texture needs at least 8 bytes to start reading"));
        }

        let teximage_params = TeximageParams::new(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
        let width_height = WidthHeight::new(u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]));

        Ok(Texture {
            teximage_params,
            width_height,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, AppError> {
        let mut buffer = vec![0; Self::SIZE];
        self.write_bytes(&mut buffer)?;
        Ok(buffer)
    }

    fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 8 {
            return Err(AppError::new("Buffer is too small to write Texture"));
        }

        buffer[0..4].copy_from_slice(&self.teximage_params.data.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.width_height.data.to_le_bytes());

        Ok(())
    }

    fn size(&self) -> usize {
        Self::SIZE
    }
}

#[derive(Clone, Copy)]
pub struct TeximageParams {
    data: u32
}

impl TeximageParams {
    pub fn new(data: u32) -> TeximageParams {
        TeximageParams { data }
    }

    pub fn texture_data(&self) -> u16 {
        (self.data & 0xFFFF) as u16
    }

    pub fn unknown_0(&self) -> u8 {
        // Zero in TEX0, derived from Model's Material
        ((self.data >> 16) & 0x0F) as u8
    }

    pub fn texture_s_size(&self) -> u8 {
        // 8 << this = texture width
        ((self.data >> 20) & 0x07) as u8
    }

    pub fn texture_t_size(&self) -> u8 {
        // 8 << this = texture height
        ((self.data >> 23) & 0x07) as u8
    }

    pub fn texture_format(&self) -> u8 {
        ((self.data >> 26) & 0x03) as u8
    }

    pub fn palette_color_0_transparent(&self) -> bool {
        (self.data & 0x20000000) != 0
    }

    pub fn unknown_1(&self) -> u8 {
        // Zero in TEX0, derived from Model's Material
        ((self.data >> 30) & 0x03) as u8
    }

}

impl Debug for TeximageParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TeximageParams")
            .field("data", &self.data)
            .field("texture_data", &self.texture_data())
            .field("unknown_0", &self.unknown_0())
            .field("texture_s_size", &self.texture_s_size())
            .field("texture_t_size", &self.texture_t_size())
            .field("texture_format", &self.texture_format())
            .field("palette_color_0_transparent", &self.palette_color_0_transparent())
            .field("unknown_1", &self.unknown_1())
            .finish()
    }
}


#[derive(Clone, Copy)]
pub struct WidthHeight {
    data: u32
}

impl WidthHeight {
    pub fn new(data: u32) -> WidthHeight {
        WidthHeight { data }
    }

    pub fn width(&self) -> u16 {
        (self.data & 0x07FF) as u16
    }

    pub fn height(&self) -> u16 {
        ((self.data >> 11) & 0x07FF) as u16
    }
}

impl Debug for WidthHeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WidthHeight")
            .field("data", &self.data)
            .field("width", &self.width())
            .field("height", &self.height())
            .finish()
    }
}
