use crate::{data_structures::{name::Name, name_list::NameList}, debug_info::DebugInfo, error::AppError, util::{math::matrix::Matrix, number::fixed_point::{fixed_1_19_12::Fixed1_19_12, fixed_1_3_12::Fixed1_3_12}}};


#[derive(Debug, Clone)]
pub struct BoneList {
    bones: NameList<u32>,

    // Actual data
    bone_matrices: Vec<BoneMatrix>,

    // Debug info
    _debug_info: DebugInfo
}

impl BoneList {
    pub fn from_bytes(bytes: &[u8], debug_info: DebugInfo) -> Result<BoneList, AppError> {
        let bones = NameList::from_bytes(bytes)?;

        let mut bone_matrices = Vec::with_capacity(bones.len());
        for &offset in bones.data_iter() {
            let offset = offset as usize;

            let bone_matrix = BoneMatrix::from_bytes(&bytes[offset..])?;

            bone_matrices.push(bone_matrix);
        }

        Ok(BoneList {
            bones,
            bone_matrices,
            _debug_info: debug_info
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        self.bones.write_bytes(buffer)?;

        for (i, &offset) in self.bones.data_iter().enumerate() {
            let offset = offset as usize;

            if i >= self.bone_matrices.len() {
                return Err(AppError::new("Bone list has more offsets than bone matrices"))
            }

            self.bone_matrices[i].write_bytes(&mut buffer[offset..])?;
        }

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.bones.len()
    }

    pub fn size(&self) -> usize {
        self.bones.size() + self.bone_matrices.iter().map(|m| m.size()).sum::<usize>()
    }

    pub fn get_name(&self, index: usize) -> Option<&Name> {
        self.bones.get_name(index)
    }

    pub fn get_bone_matrix(&self, index: usize) -> Option<&BoneMatrix> {
        self.bone_matrices.get(index)
    }

    pub fn rebase(&mut self) {
        self.bones.rebase();
    }
}


#[derive(Debug, Clone)]
pub struct BoneMatrix {
    flags: BoneMatrixFlags,
    m0: Fixed1_3_12, // For rotation matrix
    translation: Option<TranslationMatrix>,
    rotation: Option<RotationMatrix>,
    scale: Option<ScaleMatrix>
}

impl BoneMatrix {
    pub fn from_bytes(bytes: &[u8]) -> Result<BoneMatrix, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("Bone matrix needs at least 4 bytes to start reading"))
        }

        let flags = BoneMatrixFlags::from_u16(u16::from_le_bytes([bytes[0], bytes[1]]));
        let m0 = Fixed1_3_12::from(i16::from_le_bytes([bytes[2], bytes[3]]));

        let mut offset = 4;

        let translation = TranslationMatrix::from_bytes(flags.t(), &bytes[offset..])?;
        if let Some(_) = translation {
            offset += TranslationMatrix::size();
        }

        let rotation = RotationMatrix::from_bytes(flags.rp(), flags.rm(), &bytes[offset..])?;
        if let Some(_) = rotation {
            offset += RotationMatrix::size(flags.rp(), flags.rm());
        }

        let scale = ScaleMatrix::from_bytes(flags.s(), &bytes[offset..])?;        

        Ok(BoneMatrix {
            flags,
            m0,
            translation,
            rotation,
            scale
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Bone matrix needs at least 4 bytes to start writing"))
        }

        buffer[0..2].copy_from_slice(&self.flags.flags.to_le_bytes());
        buffer[2..4].copy_from_slice(&self.m0.to_le_bytes());

        let mut offset = 4;

        if let Some(translation) = &self.translation {
            translation.write_bytes(self.flags.t(), &mut buffer[offset..])?;
            offset += TranslationMatrix::size();
        }

        
        if let Some(rotation) = &self.rotation {
            rotation.write_bytes(self.flags.rp(), self.flags.rm(), &mut buffer[offset..])?;
            offset += RotationMatrix::size(self.flags.rp(), self.flags.rm());
        }

        if let Some(scale) = &self.scale {
            scale.write_bytes(self.flags.s(), &mut buffer[offset..])?;
        }

        Ok(())
    }

    pub fn size(&self) -> usize {
        let mut size = 4;

        if let Some(_) = &self.translation {
            size += TranslationMatrix::size();
        }

        if let Some(_) = &self.rotation {
            size += RotationMatrix::size(self.flags.rp(), self.flags.rm());
        }

        if let Some(_) = &self.scale {
            size += ScaleMatrix::size();
        }

        size
    }

    pub fn to_matrix(&self) -> Matrix {
        let translation_component = if let Some(translation) = &self.translation {
            Some([translation.x.to_f32(), translation.y.to_f32(), translation.z.to_f32()])
        } else { None };

        let rotation_component = if let Some(rotation) = &self.rotation {
            rotation.matrix_data(self.flags, self.m0)
        } else { None };

        let scale_component = if let Some(scale) = &self.scale {
            Some([scale.x.to_f32(), scale.y.to_f32(), scale.z.to_f32()])
        } else { None };


        let mut matrix = Matrix::identity(4);
        if let Some(scale) = scale_component {
            matrix.set(0, 0, scale[0]).unwrap();
            matrix.set(1, 1, scale[1]).unwrap();
            matrix.set(2, 2, scale[2]).unwrap();
        }

        if let Some(rotation) = rotation_component {
            let rotation_matrix = Matrix::new(4, 4, vec![
                rotation[0], rotation[1], rotation[2], 0.0,
                rotation[3], rotation[4], rotation[5], 0.0,
                rotation[6], rotation[7], rotation[8], 0.0,
                0.0, 0.0, 0.0, 1.0
            ]).unwrap();

            matrix = rotation_matrix * matrix;
        }

        if let Some(translation) = translation_component {
            let mut translation_matrix = Matrix::identity(4);
            translation_matrix.set(0, 3, translation[0]).unwrap();
            translation_matrix.set(1, 3, translation[1]).unwrap();
            translation_matrix.set(2, 3, translation[2]).unwrap();

            matrix = translation_matrix * matrix;
        }

        matrix
    }
}


#[derive(Debug, Clone, Copy)]
pub struct BoneMatrixFlags {
    flags: u16
}

impl BoneMatrixFlags {
    pub fn from_u16(flags: u16) -> BoneMatrixFlags {
        BoneMatrixFlags { flags }
    }

    pub fn t(&self) -> bool {
        self.flags & 0x1 != 0
    }

    pub fn rm(&self) -> bool {
        self.flags & 0x2 != 0
    }

    pub fn s(&self) -> bool {
        self.flags & 0x4 != 0
    }

    pub fn rp(&self) -> bool {
        self.flags & 0x8 != 0
    }

    pub fn form(&self) -> u8 {
        ((self.flags >> 4) & 0x0F) as u8
    }

    pub fn neg_one(&self) -> bool {
        self.flags & 0x100 != 0
    }

    pub fn neg_c(&self) -> bool {
        self.flags & 0x200 != 0
    }

    pub fn neg_d(&self) -> bool {
        self.flags & 0x400 != 0
    }
}

#[derive(Debug, Clone)]
pub struct TranslationMatrix {
    x: Fixed1_19_12,
    y: Fixed1_19_12,
    z: Fixed1_19_12,
}

impl TranslationMatrix {
    pub fn from_bytes(t: bool, bytes: &[u8]) -> Result<Option<TranslationMatrix>, AppError> {
        if t {
            return Ok(None);
        }

        if bytes.len() < 12 {
            return Err(AppError::new("Translation matrix needs at least 12 bytes"))
        }

        let x_i32 = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let y_i32 = i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let z_i32 = i32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

        let x = Fixed1_19_12::from(x_i32);
        let y = Fixed1_19_12::from(y_i32);
        let z = Fixed1_19_12::from(z_i32);

        Ok(Some(TranslationMatrix { x, y, z }))
    }

    pub fn write_bytes(&self, t: bool, buffer: &mut [u8]) -> Result<(), AppError> {
        if t {
            return Ok(());
        }

        if buffer.len() < 12 {
            return Err(AppError::new("Translation matrix needs at least 12 bytes"))
        }

        buffer[0..4].copy_from_slice(&self.x.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.y.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.z.to_le_bytes());

        Ok(())
    }

    pub fn size() -> usize {
        12
    }
}


#[derive(Debug, Clone)]
pub struct RotationMatrix {
    // If rp == 1, take 2 first elements as a and b. Else if rm == 0, 3x3 matrix 
    data: [Fixed1_3_12; 8]
}

impl RotationMatrix {
    pub fn from_bytes(rp: bool, rm: bool, bytes: &[u8]) -> Result<Option<RotationMatrix>, AppError> {
        if rp {
            if bytes.len() < 4 {
                return Err(AppError::new("Rotation matrix with rp=1 needs at least 4 bytes"))
            }

            let a_i16 = i16::from_le_bytes([bytes[0], bytes[1]]);
            let b_i16 = i16::from_le_bytes([bytes[2], bytes[3]]);

            let a = Fixed1_3_12::from(a_i16);
            let b = Fixed1_3_12::from(b_i16);

            // To fill the rest of the matrix
            let zero = Fixed1_3_12::from(0i16);

            let data = [a, b, zero, zero, zero, zero, zero, zero];

            return Ok(Some(RotationMatrix { data }))
        }
        else if !rm {
            if bytes.len() < 16 {
                return Err(AppError::new("Rotation matrix with rm=0 needs at least 16 bytes"));
            }

            let m1_i16 = i16::from_le_bytes([bytes[0], bytes[1]]);
            let m2_i16 = i16::from_le_bytes([bytes[2], bytes[3]]);
            let m3_i16 = i16::from_le_bytes([bytes[4], bytes[5]]);
            let m4_i16 = i16::from_le_bytes([bytes[6], bytes[7]]);
            let m5_i16 = i16::from_le_bytes([bytes[8], bytes[9]]);
            let m6_i16 = i16::from_le_bytes([bytes[10], bytes[11]]);
            let m7_i16 = i16::from_le_bytes([bytes[12], bytes[13]]);
            let m8_i16 = i16::from_le_bytes([bytes[14], bytes[15]]);

            let m1 = Fixed1_3_12::from(m1_i16);
            let m2 = Fixed1_3_12::from(m2_i16);
            let m3 = Fixed1_3_12::from(m3_i16);
            let m4 = Fixed1_3_12::from(m4_i16);
            let m5 = Fixed1_3_12::from(m5_i16);
            let m6 = Fixed1_3_12::from(m6_i16);
            let m7 = Fixed1_3_12::from(m7_i16);
            let m8 = Fixed1_3_12::from(m8_i16);

            let data = [m1, m2, m3, m4, m5, m6, m7, m8];

            return Ok(Some(RotationMatrix { data }))
        }

        Ok(None)
    }

    pub fn write_bytes(&self, rp: bool, rm: bool, buffer: &mut [u8]) -> Result<(), AppError> {
        if rp {
            if buffer.len() < 4 {
                return Err(AppError::new("Rotation matrix with rp=1 needs at least 4 bytes"))
            }

            buffer[0..2].copy_from_slice(&self.data[0].to_le_bytes());
            buffer[2..4].copy_from_slice(&self.data[1].to_le_bytes());

            return Ok(())
        }
        else if !rm {
            if buffer.len() < 16 {
                return Err(AppError::new("Rotation matrix with rm=0 needs at least 16 bytes"));
            }

            for i in 0..8 {
                let start = i * 2;
                buffer[start..start + 2].copy_from_slice(&self.data[i].to_le_bytes());
            }

            return Ok(())
        }

        Ok(())
    }

    pub fn size(rp: bool, rm: bool) -> usize {
        if rp {
            return 4
        }
        else if !rm {
            return 16
        }

        0
    }

    pub fn matrix_data(&self, flags: BoneMatrixFlags, m0: Fixed1_3_12) -> Option<[f32; 9]> {
        if flags.rp() {
            let a = self.data[0].to_f32();
            let b = self.data[1].to_f32();
            let form = flags.form();
            let neg_one = flags.neg_one();
            let neg_c = flags.neg_c();
            let neg_d = flags.neg_d();

            if form >= 9 {
                return Some([-a, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
            }

            let one = if neg_one { -1.0 } else { 1.0 };
            let c = if neg_c { -b } else { b };
            let d = if neg_d { -a } else { a };

            let final_data = match form {
                0 => [one, 0.0, 0.0, 0.0, a, c, 0.0, b, d],
                1 => [0.0, a, c, one, 0.0, 0.0, 0.0, b, d],
                2 => [0.0, a, c, 0.0, b, d, one, 0.0, 0.0],
                3 => [0.0, one, 0.0, a, 0.0, c, b, 0.0, d],
                4 => [a, 0.0, c, 0.0, one, 0.0, b, 0.0, d],
                5 => [a, 0.0, c, b, 0.0, d, 0.0, one, 0.0],
                6 => [0.0, 0.0, one, a, c, 0.0, b, d, 0.0],
                7 => [a, c, 0.0, 0.0, 0.0, one, b, d, 0.0],
                8 => [a, c, 0.0, b, d, 0.0, 0.0, 0.0, one],
                _ => unreachable!()
            };

            Some(final_data)
        }
        else if !flags.rm() {
            Some([
                m0.to_f32(), self.data[2].to_f32(), self.data[5].to_f32(),
                self.data[0].to_f32(), self.data[3].to_f32(), self.data[6].to_f32(),
                self.data[1].to_f32(), self.data[4].to_f32(), self.data[7].to_f32()
            ])
        }
        else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScaleMatrix {
    x: Fixed1_19_12,
    y: Fixed1_19_12,
    z: Fixed1_19_12,
}

impl ScaleMatrix {
    pub fn from_bytes(s: bool, bytes: &[u8]) -> Result<Option<ScaleMatrix>, AppError> {
        if s {
            return Ok(None);
        }

        if bytes.len() < 12 {
            return Err(AppError::new("Scale matrix needs at least 12 bytes"))
        }

        let x_i32 = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let y_i32 = i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let z_i32 = i32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

        let x = Fixed1_19_12::from(x_i32);
        let y = Fixed1_19_12::from(y_i32);
        let z = Fixed1_19_12::from(z_i32);

        Ok(Some(ScaleMatrix { x, y, z }))
    }

    pub fn write_bytes(&self, s: bool, buffer: &mut [u8]) -> Result<(), AppError> {
        if s {
            return Ok(());
        }

        if buffer.len() < 12 {
            return Err(AppError::new("Scale matrix needs at least 12 bytes"))
        }

        buffer[0..4].copy_from_slice(&self.x.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.y.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.z.to_le_bytes());

        Ok(())
    }

    pub fn size() -> usize {
        12
    }
}
