use palette::PaletteList;
use texture::TextureList;

use crate::{debug_info::DebugInfo, error::AppError};

pub mod texture;
pub mod palette;

#[derive(Debug, Clone)]
pub struct Tex {
    stamp: [u8; 4],
    chunk_size: u32,
    padding_0: u32, // 0
    texture_data_size: u16, // length / 8
    texture_list_offset: u16, // Texture list offset
    padding_1: u32, // 0
    texture_data_offset: u32,
    padding_2: u32, // 0
    compressed_texture_data_size: u16, // length / 12
    compressed_texture_list_offset: u16, // Compressed texture list offset
    padding_3: u32, // 0
    compressed_texture_4x4_data_offset: u32, // Compressed Texture Offset for 4x4-Texel Data
    compressed_texture_4x4_attr_offset: u32, // Compressed Texture Offset for 4x4-Texel Attr
    padding_4: u32, // 0
    palette_data_size: u32, // length / 8
    palette_list_offset: u32, // Palette list offset
    palette_data_offset: u32,

    // Actual data
    texture_list: TextureList, // Always at 0x3C
    compressed_texture_list: TextureList, // Should be the exact same as texture_list at 0x3C
    palette_list: PaletteList,

    texture_data: Vec<u8>,
    palette_data: Vec<u8>,

    // Debug info
    _debug_info: DebugInfo
}

impl Tex {
    pub fn from_bytes(bytes: &[u8], debug_info: DebugInfo) -> Result<Tex, AppError> {
        if bytes.len() < 60 {
            return Err(AppError::new("Tex needs at least 56 bytes to start reading"));
        }

        let stamp = [
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
        ];

        let chunk_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let padding_0 = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let texture_data_size = u16::from_le_bytes([bytes[12], bytes[13]]);
        let texture_list_offset = u16::from_le_bytes([bytes[14], bytes[15]]);
        let padding_1 = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let texture_data_offset = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        let padding_2 = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
        let compressed_texture_data_size = u16::from_le_bytes([bytes[28], bytes[29]]);
        let compressed_texture_list_offset = u16::from_le_bytes([bytes[30], bytes[31]]);
        let padding_3 = u32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]);
        let compressed_texture_4x4_data_offset = u32::from_le_bytes([bytes[36], bytes[37], bytes[38], bytes[39]]);
        let compressed_texture_4x4_attr_offset = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]);
        let padding_4 = u32::from_le_bytes([bytes[44], bytes[45], bytes[46], bytes[47]]);
        let palette_data_size = u32::from_le_bytes([bytes[48], bytes[49], bytes[50], bytes[51]]);
        let palette_list_offset = u32::from_le_bytes([bytes[52], bytes[53], bytes[54], bytes[55]]);
        let palette_data_offset = u32::from_le_bytes([bytes[56], bytes[57], bytes[58], bytes[59]]);
        
        let texture_list = TextureList::from_bytes(&bytes[texture_list_offset as usize..])?;
        let compressed_texture_list = TextureList::from_bytes(&bytes[compressed_texture_list_offset as usize..])?;
        let palette_list = PaletteList::from_bytes(&bytes[palette_list_offset as usize..])?;

        let texture_data = bytes[texture_data_offset as usize..texture_data_offset as usize + texture_data_size as usize * 8].to_vec();
        let palette_data = bytes[palette_data_offset as usize..palette_data_offset as usize + palette_data_size as usize * 8].to_vec();

        let tex = Tex {
            stamp,
            chunk_size,
            padding_0,
            texture_data_size,
            texture_list_offset,
            padding_1,
            texture_data_offset,
            padding_2,
            compressed_texture_data_size,
            compressed_texture_list_offset,
            padding_3,
            compressed_texture_4x4_data_offset,
            compressed_texture_4x4_attr_offset,
            padding_4,
            palette_data_size,
            palette_list_offset,
            palette_data_offset,

            texture_list,
            compressed_texture_list,
            palette_list,
            texture_data,
            palette_data,

            _debug_info: debug_info
        };

        Ok(tex)
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < self.chunk_size as usize {
            return Err(AppError::new("Buffer is too small to write Tex"));
        }

        buffer[0..4].copy_from_slice(&self.stamp);
        buffer[4..8].copy_from_slice(&self.chunk_size.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.padding_0.to_le_bytes());
        buffer[12..14].copy_from_slice(&self.texture_data_size.to_le_bytes());
        buffer[14..16].copy_from_slice(&self.texture_list_offset.to_le_bytes());
        buffer[16..20].copy_from_slice(&self.padding_1.to_le_bytes());
        buffer[20..24].copy_from_slice(&self.texture_data_offset.to_le_bytes());
        buffer[24..28].copy_from_slice(&self.padding_2.to_le_bytes());
        buffer[28..30].copy_from_slice(&self.compressed_texture_data_size.to_le_bytes());
        buffer[30..32].copy_from_slice(&self.compressed_texture_list_offset.to_le_bytes());
        buffer[32..36].copy_from_slice(&self.padding_3.to_le_bytes());
        buffer[36..40].copy_from_slice(&self.compressed_texture_4x4_data_offset.to_le_bytes());
        buffer[40..44].copy_from_slice(&self.compressed_texture_4x4_attr_offset.to_le_bytes());
        buffer[44..48].copy_from_slice(&self.padding_4.to_le_bytes());
        buffer[48..52].copy_from_slice(&self.palette_data_size.to_le_bytes());
        buffer[52..56].copy_from_slice(&self.palette_list_offset.to_le_bytes());
        buffer[56..60].copy_from_slice(&self.palette_data_offset.to_le_bytes());
        self.texture_list.write_bytes(&mut buffer[self.texture_list_offset as usize..])?;
        self.compressed_texture_list.write_bytes(&mut buffer[self.compressed_texture_list_offset as usize..])?;
        self.palette_list.write_bytes(&mut buffer[self.palette_list_offset as usize..])?;
        buffer[self.texture_data_offset as usize..self.texture_data_offset as usize + self.texture_data_size as usize * 8].copy_from_slice(&self.texture_data);
        buffer[self.palette_data_offset as usize..self.palette_data_offset as usize + self.palette_data_size as usize * 8].copy_from_slice(&self.palette_data);


        Ok(())
    }

    pub fn size(&self) -> usize {
        self.chunk_size as usize
    }

    pub fn texture_list(&self) -> &TextureList {
        &self.texture_list
    }

    pub fn texture_list_mut(&mut self) -> &mut TextureList {
        &mut self.texture_list
    }
}
