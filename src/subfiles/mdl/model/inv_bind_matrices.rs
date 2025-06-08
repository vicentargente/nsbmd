use crate::{debug_info::DebugInfo, error::AppError, util::number::fixed_point::fixed_1_19_12::Fixed1_19_12};

#[derive(Debug, Clone)]
pub struct InvBindMatrices {
    matrices: Vec<InvBindMatrix>,

    // Debug info
    _debug_info: DebugInfo
}

impl InvBindMatrices {
    pub fn from_bytes(bytes: &[u8], debug_info: DebugInfo) -> Result<InvBindMatrices, AppError> {
        if bytes.len() %  InvBindMatrix::SIZE != 0 {
            return Err(AppError::new("InvBindMatrices needs a multiple of 84 bytes"))
        }

        let mut matrices = Vec::with_capacity(bytes.len() / InvBindMatrix::SIZE);

        for offset in (0..bytes.len()).step_by(InvBindMatrix::SIZE) {
            let matrix = InvBindMatrix::from_bytes(&bytes[offset..])?;
            matrices.push(matrix);
        }

        Ok(InvBindMatrices {
            matrices,
            _debug_info: debug_info
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < self.matrices.len() * InvBindMatrix::SIZE {
            return Err(AppError::new("Buffer is too small to write InvBindMatrices"));
        }

        for (i, matrix) in self.matrices.iter().enumerate() {
            let offset = i * InvBindMatrix::SIZE;
            matrix.write_bytes(&mut buffer[offset..])?;
        }

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.matrices.len() * InvBindMatrix::SIZE
    }
}


#[derive(Debug, Clone)]
pub struct InvBindMatrix {
    position_matrix: [Fixed1_19_12; 12], // 3x4
    vector_matrix: [Fixed1_19_12; 9] // 3x3
}

impl InvBindMatrix {
    const SIZE: usize = 84;

    pub fn from_bytes(bytes: &[u8]) -> Result<InvBindMatrix, AppError> {
        if bytes.len() < 84 {
            return Err(AppError::new("InvBindMatrix needs at least 84 bytes"))
        }

        let pos_0 = Fixed1_19_12::from(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
        let pos_1 = Fixed1_19_12::from(i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]));
        let pos_2 = Fixed1_19_12::from(i32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]));
        let pos_3 = Fixed1_19_12::from(i32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]));
        let pos_4 = Fixed1_19_12::from(i32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]));
        let pos_5 = Fixed1_19_12::from(i32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]));
        let pos_6 = Fixed1_19_12::from(i32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]));
        let pos_7 = Fixed1_19_12::from(i32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]));
        let pos_8 = Fixed1_19_12::from(i32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]));
        let pos_9 = Fixed1_19_12::from(i32::from_le_bytes([bytes[36], bytes[37], bytes[38], bytes[39]]));
        let pos_10 = Fixed1_19_12::from(i32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]));
        let pos_11 = Fixed1_19_12::from(i32::from_le_bytes([bytes[44], bytes[45], bytes[46], bytes[47]]));

        let vec_0 = Fixed1_19_12::from(i32::from_le_bytes([bytes[48], bytes[49], bytes[50], bytes[51]]));
        let vec_1 = Fixed1_19_12::from(i32::from_le_bytes([bytes[52], bytes[53], bytes[54], bytes[55]]));
        let vec_2 = Fixed1_19_12::from(i32::from_le_bytes([bytes[56], bytes[57], bytes[58], bytes[59]]));
        let vec_3 = Fixed1_19_12::from(i32::from_le_bytes([bytes[60], bytes[61], bytes[62], bytes[63]]));
        let vec_4 = Fixed1_19_12::from(i32::from_le_bytes([bytes[64], bytes[65], bytes[66], bytes[67]]));
        let vec_5 = Fixed1_19_12::from(i32::from_le_bytes([bytes[68], bytes[69], bytes[70], bytes[71]]));
        let vec_6 = Fixed1_19_12::from(i32::from_le_bytes([bytes[72], bytes[73], bytes[74], bytes[75]]));
        let vec_7 = Fixed1_19_12::from(i32::from_le_bytes([bytes[76], bytes[77], bytes[78], bytes[79]]));
        let vec_8 = Fixed1_19_12::from(i32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]));

        let position_matrix = [
            pos_0, pos_1, pos_2, pos_3,
            pos_4, pos_5, pos_6, pos_7,
            pos_8, pos_9, pos_10, pos_11
        ];

        let vector_matrix = [
            vec_0, vec_1, vec_2,
            vec_3, vec_4, vec_5,
            vec_6, vec_7, vec_8
        ];

        Ok(InvBindMatrix {
            position_matrix,
            vector_matrix
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < InvBindMatrix::SIZE {
            return Err(AppError::new("Buffer is too small to write InvBindMatrix"));
        }

        buffer[0..48].copy_from_slice(
            &self.position_matrix.iter()
                .flat_map(|x| x.to_le_bytes())
                .collect::<Vec<u8>>()[..]
        );

        buffer[48..84].copy_from_slice(
            &self.vector_matrix.iter()
                .flat_map(|x| x.to_le_bytes())
                .collect::<Vec<u8>>()[..]
        );

        Ok(())
    }
}
