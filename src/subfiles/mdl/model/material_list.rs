use crate::{data_structures::name_list::NameList, debug_info::DebugInfo, error::AppError, traits::BinarySerializable, util::number::alignment::get_4_byte_alignment};

#[derive(Debug, Clone)]
pub struct MaterialList {
    texture_pairings_offset: u16,
    palette_pairings_offset: u16,
    materials: NameList<u32>,

    // Actual data
    texture_pairing_list: TexturePairingList,
    palette_pairing_list: PalettePairingList,
    materials_data: Vec<Material>,

    // Debug info
    _debug_info: DebugInfo
}

impl MaterialList {
    pub fn from_bytes(bytes: &[u8], debug_info: DebugInfo) -> Result<MaterialList, AppError> {
        if bytes.len() < 44 { // 4 bytes for offsets + 40 bytes for material list
            return Err(AppError::new("MaterialList needs at least 44 bytes"));
        }

        let texture_pairings_offset = u16::from_le_bytes([bytes[0], bytes[1]]);
        let palette_pairings_offset = u16::from_le_bytes([bytes[2], bytes[3]]);
        let materials = NameList::from_bytes(&bytes[4..])?;

        let mut materials_data = Vec::with_capacity(materials.len());
        for &offset in materials.data_iter() {
            let offset = offset as usize;

            let material = Material::from_bytes(&bytes[offset..], DebugInfo { offset: debug_info.offset + offset as u32 })?;
            materials_data.push(material);
        }

        let mut texture_pairing_list = TexturePairingList::from_bytes(
            &bytes[texture_pairings_offset as usize..],
            DebugInfo { offset: debug_info.offset + texture_pairings_offset as u32 }
        )?;

        let mut palette_pairing_list = PalettePairingList::from_bytes(
            &bytes[palette_pairings_offset as usize..],
            DebugInfo { offset: debug_info.offset + palette_pairings_offset as u32 }
        )?;

        // Read indices for the pairing lists
        texture_pairing_list.read_indices(bytes)?;
        palette_pairing_list.read_indices(bytes)?;

        Ok(MaterialList {
            texture_pairings_offset,
            palette_pairings_offset,
            materials,
            materials_data,
            texture_pairing_list,
            palette_pairing_list,
            _debug_info: debug_info
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 44 { // 4 bytes for offsets + 40 bytes for material list
            return Err(AppError::new("MaterialList needs at least 44 bytes"));
        }

        buffer[0..2].copy_from_slice(&self.texture_pairings_offset.to_le_bytes());
        buffer[2..4].copy_from_slice(&self.palette_pairings_offset.to_le_bytes());
        self.materials.write_bytes(&mut buffer[4..])?;

        for (i, &offset) in self.materials.data_iter().enumerate() {
            let offset = offset as usize;
            let material = &self.materials_data[i];
            material.write_bytes(&mut buffer[offset..])?;
        }

        self.texture_pairing_list.write_bytes(&mut buffer[self.texture_pairings_offset as usize..])?;
        self.palette_pairing_list.write_bytes(&mut buffer[self.palette_pairings_offset as usize..])?;
        
        self.texture_pairing_list.write_indices(buffer)?;
        self.palette_pairing_list.write_indices(buffer)?;
        Ok(())
    }

    pub fn size(&self) -> usize {
        // We get it like this, since there might be empty bytes in the middle of the material list
        usize::max(usize::max(
            *(self.materials.data_iter().max().unwrap_or(&0)) as usize + Material::SIZE, // Last material
            self.texture_pairings_offset as usize + self.texture_pairing_list.size()), // Texture pairing
            self.palette_pairings_offset as usize + self.palette_pairing_list.size() // Palette pairing
        )
    }

    pub fn rebase(&mut self) {
        self.materials.rebase();
        self.texture_pairing_list.rebase();
        self.palette_pairing_list.rebase();

        let mut offset = 4; // texture_pairings_offset (2 bytes) + palette_pairings_offset (2 bytes)
        offset += self.materials.size();

        self.texture_pairings_offset = offset as u16;
        offset += self.texture_pairing_list.size();

        self.palette_pairings_offset = offset as u16;
        offset += self.palette_pairing_list.size();

        // Indices from pairing lists go after all the pairing lists and before the materials. They don't need to be aligned (they are individual bytes)
        self.texture_pairing_list.set_begin_indices_offset(offset as u16);
        offset += self.texture_pairing_list.total_indices_count();
        self.palette_pairing_list.set_begin_indices_offset(offset as u16);
        offset += self.palette_pairing_list.total_indices_count();

        offset = get_4_byte_alignment(offset); // Material data must be 4-byte aligned

        for material_offset in self.materials.data_iter_mut() {
            *material_offset = offset as u32;
            offset += Material::SIZE;
        }
    }
}


#[derive(Debug, Clone)]
pub struct Material {
    dummy: u16,
    size: u16,

    dif_amb: u32, // Value for DIFF_AMB register
    spe_emi: u32, // Value for SPE_EMI register
    polygon_attr: u32, // Value for POLYGON_ATTR register
    unknown_0: u32, // Mask for POLYGON_ATTR register??
    teximage_params: TexImageParams,

    unknown_1: u32,
    unknown_2: u32,

    texture_width: u16,
    texture_height: u16,

    remaining_fields: [u8; 8],

    // Debug info
    _debug_info: DebugInfo
}

impl Material {
    const SIZE: usize = 44;

    pub fn from_bytes(bytes: &[u8], debug_info: DebugInfo) -> Result<Material, AppError> {
        if bytes.len() < Material::SIZE {
            return Err(AppError::new("Material needs at least 44 bytes"));
        }

        let dummy = u16::from_le_bytes([bytes[0], bytes[1]]);
        let size = u16::from_le_bytes([bytes[2], bytes[3]]);

        let dif_amb = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let spe_emi = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let polygon_attr = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let unknown_0 = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let teximage_params = TexImageParams::from_u32(u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]));

        let unknown_1 = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
        let unknown_2 = u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]);

        let texture_width = u16::from_le_bytes([bytes[32], bytes[33]]);
        let texture_height = u16::from_le_bytes([bytes[34], bytes[35]]);

        let remaining_fields = [bytes[36], bytes[37], bytes[38], bytes[39], bytes[40], bytes[41], bytes[42], bytes[43]];

        Ok(Material {
            dummy,
            size,
            dif_amb,
            spe_emi,
            polygon_attr,
            unknown_0,
            teximage_params,
            unknown_1,
            unknown_2,
            texture_width,
            texture_height,
            remaining_fields,
            _debug_info: debug_info
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < Material::SIZE {
            return Err(AppError::new("Material needs at least 44 bytes"));
        }

        buffer[0..2].copy_from_slice(&self.dummy.to_le_bytes());
        buffer[2..4].copy_from_slice(&self.size.to_le_bytes());

        buffer[4..8].copy_from_slice(&self.dif_amb.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.spe_emi.to_le_bytes());
        buffer[12..16].copy_from_slice(&self.polygon_attr.to_le_bytes());
        buffer[16..20].copy_from_slice(&self.unknown_0.to_le_bytes());
        self.teximage_params.write_bytes(&mut buffer[20..24])?;

        buffer[24..28].copy_from_slice(&self.unknown_1.to_le_bytes());
        buffer[28..32].copy_from_slice(&self.unknown_2.to_le_bytes());

        buffer[32..34].copy_from_slice(&self.texture_width.to_le_bytes());
        buffer[34..36].copy_from_slice(&self.texture_height.to_le_bytes());

        buffer[36..44].copy_from_slice(&self.remaining_fields);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TexImageParams {
    data: u32
}

impl TexImageParams {
    pub fn from_u32(data: u32) -> TexImageParams {
        TexImageParams {
            data
        }
    }

    pub fn repeat_s(&self) -> bool {
        (self.data & 0x00010000) != 0
    }

    pub fn set_repeat_s(&mut self, repeat: bool) {
        if repeat {
            self.data |= 0x00010000;
        } else {
            self.data &= !0x00010000;
        }
    }

    pub fn repeat_t(&self) -> bool {
        (self.data & 0x00020000) != 0
    }

    pub fn set_repeat_t(&mut self, repeat: bool) {
        if repeat {
            self.data |= 0x00020000;
        } else {
            self.data &= !0x00020000;
        }
    }

    pub fn mirror_s(&self) -> bool {
        (self.data & 0x00040000) != 0
    }

    pub fn set_mirror_s(&mut self, mirror: bool) {
        if mirror {
            self.data |= 0x00040000;
        } else {
            self.data &= !0x00040000;
        }
    }

    pub fn mirror_t(&self) -> bool {
        (self.data & 0x00080000) != 0
    }

    pub fn set_mirror_t(&mut self, mirror: bool) {
        if mirror {
            self.data |= 0x00080000;
        } else {
            self.data &= !0x00080000;
        }
    }

    pub fn texcoords_transform_mode(&self) -> u8 {
        ((self.data >> 30) & 0x03) as u8
    }

    pub fn set_texcoords_transform_mode(&mut self, mode: u8) -> Result<(), AppError> {
        if mode > 3 {
            return Err(AppError::new("Invalid texture coordinates transform mode. Expected two bits"));
        }

        self.data &= !0xC0000000;
        self.data |= (mode as u32) << 30;

        Ok(())
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("TexImageParams needs at least 4 bytes"));
        }

        buffer[0..4].copy_from_slice(&self.data.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct TexturePairingList {
    texture_pairings: NameList<MaterialIdxList>,

    // Debug info
    _debug_info: DebugInfo
}

impl TexturePairingList {
    pub fn from_bytes(bytes: &[u8], debug_info: DebugInfo) -> Result<TexturePairingList, AppError> {
        // No bound checks, since NameList has its own checks
        let texture_pairings = NameList::from_bytes(bytes)?;

        Ok(TexturePairingList {
            texture_pairings,
            _debug_info: debug_info
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        // No bound checks, since NameList has its own checks
        self.texture_pairings.write_bytes(buffer)?;

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.texture_pairings.size()
    }

    pub fn rebase(&mut self) {
        self.texture_pairings.rebase();

        for pairing in self.texture_pairings.data_iter_mut() {
            pairing.rebase();
        }
    }

    pub fn read_indices(&mut self, material_list_bytes: &[u8]) -> Result<(), AppError> {
        for pairing in self.texture_pairings.data_iter_mut() {
            pairing.read_indices(material_list_bytes)?;
        }

        Ok(())
    }

    pub fn write_indices(&self, material_list_buffer: &mut [u8]) -> Result<(), AppError> {
        for pairing in self.texture_pairings.data_iter() {
            pairing.write_indices(material_list_buffer)?;
        }

        Ok(())
    }

    pub fn total_indices_count(&self) -> usize {
        self.texture_pairings.data_iter()
            .map(|pairing| pairing.count as usize)
            .sum()
    }

    pub fn set_begin_indices_offset(&mut self, offset: u16) {
        let mut offset = offset;
        for pairing in self.texture_pairings.data_iter_mut() {
            pairing.offset = offset;
            offset += pairing.count as u16;
        }
    }
}


#[derive(Debug, Clone)]
pub struct PalettePairingList {
    palette_pairings: NameList<MaterialIdxList>,

    // Debug info
    _debug_info: DebugInfo
}

impl PalettePairingList {
    pub fn from_bytes(bytes: &[u8], debug_info: DebugInfo) -> Result<PalettePairingList, AppError> {
        let palette_pairings = NameList::from_bytes(bytes)?;

        Ok(PalettePairingList {
            palette_pairings,
            _debug_info: debug_info
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        self.palette_pairings.write_bytes(buffer)?;

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.palette_pairings.size()
    }

    pub fn rebase(&mut self) {
        self.palette_pairings.rebase();
        
        for pairing in self.palette_pairings.data_iter_mut() {
            pairing.rebase();
        }
    }

    pub fn read_indices(&mut self, material_list_bytes: &[u8]) -> Result<(), AppError> {
        for pairing in self.palette_pairings.data_iter_mut() {
            pairing.read_indices(material_list_bytes)?;
        }

        Ok(())
    }

    pub fn write_indices(&self, material_list_buffer: &mut [u8]) -> Result<(), AppError> {
        for pairing in self.palette_pairings.data_iter() {
            pairing.write_indices(material_list_buffer)?;
        }

        Ok(())
    }

    pub fn total_indices_count(&self) -> usize {
        self.palette_pairings.data_iter()
            .map(|pairing| pairing.count as usize)
            .sum()
    }

    pub fn set_begin_indices_offset(&mut self, offset: u16) {
        let mut offset = offset;
        for pairing in self.palette_pairings.data_iter_mut() {
            pairing.offset = offset;
            offset += pairing.count as u16;
        }
    }
}


#[derive(Debug, Clone)]
pub struct MaterialIdxList {
    offset: u16,

    count: u8,
    dummy: u8,

    // Data pointed to by offset
    indices: Vec<u8>

}

impl MaterialIdxList {
    const SIZE: usize = 4; // Offset (2 bytes) + Count (1 byte) + Dummy (1 byte)

    fn read_indices(&mut self, material_list_bytes: &[u8]) -> Result<(), AppError> {
        if material_list_bytes.len() < (self.offset + self.count as u16) as usize {
            return Err(AppError::new(&format!("MaterialIdxList needs at least {} bytes from the MaterialList to read indices", self.offset + self.count as u16)));
        }

        if self.indices.len() > 0 {
            self.indices.clear(); // Clear previous indices if any (should never happen)
        }

        for i in 0..self.count {
            let index = material_list_bytes[self.offset as usize + i as usize];
            self.indices.push(index);
        }

        Ok(())
    }

    fn write_indices(&self, material_list_buffer: &mut [u8]) -> Result<(), AppError> {
        if material_list_buffer.len() < (self.offset + self.count as u16) as usize {
            return Err(AppError::new(&format!("MaterialIdxList needs at least {} bytes from the MaterialList to write indices", self.offset + self.count as u16)));
        }

        for (i, &index) in self.indices.iter().enumerate() {
            material_list_buffer[self.offset as usize + i] = index;
        }

        Ok(())
    }

    pub fn rebase(&mut self) {
        self.count = self.indices.len() as u8;
    }
}

impl BinarySerializable for MaterialIdxList {
    fn from_bytes(bytes: &[u8]) -> Result<Self, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("MaterialIdxList needs at least 4 bytes"));
        }

        let offset = u16::from_le_bytes([bytes[0], bytes[1]]);
        let count = bytes[2];
        let dummy = bytes[3];

        if bytes.len() < (offset + count as u16) as usize {
            return Err(AppError::new(&format!("MaterialIdxList needs at least {} bytes", offset + count as u16)));
        }

        Ok(MaterialIdxList {
            offset,
            count,
            dummy,
            // As indices offset is from the material list, we cannot read them here
            indices: Vec::with_capacity(count as usize)
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, AppError> {
        let mut bytes = Vec::with_capacity(4);
        bytes.extend_from_slice(&self.offset.to_le_bytes());
        bytes.push(self.count);
        bytes.push(self.dummy);

        // Not returning the indices, since they can be far appart from the struct. To do that, use write_bytes instead

        Ok(bytes)
    }

    fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() <= self.offset as usize {
            return Err(AppError::new(&format!("MaterialIdxList needs at least {} bytes", self.offset)));
        }

        buffer[0..2].copy_from_slice(&self.offset.to_le_bytes());
        buffer[2] = self.count;
        buffer[3] = self.dummy;

        // We do not write the indices, as offset is from the material list, not from this struct

        Ok(())
    }
    
    fn size(&self) -> usize {
        Self::SIZE
    }
}
