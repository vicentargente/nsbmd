use crate::{error::AppError, util::number::{alignment::get_4_byte_alignment, fixed_point::{fixed_1_0_9::Fixed1_0_9, fixed_1_11_4::Fixed1_11_4, fixed_1_19_12::Fixed1_19_12, fixed_1_3_12::Fixed1_3_12, fixed_1_3_6::Fixed1_3_6}}};

static SIZES: [i8; 66] = [
    0, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    1,  0,  1,  1,  1,  0, 16, 12, 16, 12,  9,  3,  3, -1, -1, -1,
    1,  1,  1,  2,  1,  1,  1,  1,  1,  1,  1,  1, -1, -1, -1, -1,
    1,  1,  1,  1,  1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    1,  0
];

#[derive(Debug, Clone)]
pub struct GpuCommandList {
    render_cmds: Vec<GpuCommand>
}

impl GpuCommandList {
    pub fn from_bytes(bytes: &[u8]) -> Result<GpuCommandList, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("GpuCommandList needs at least 4 bytes"));
        }

        let mut render_cmds = Vec::new();

        let mut pos = 0;
        while pos < bytes.len() {
            let ops = [bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]];
            pos += 4;
    
            for &op in ops.iter() {
                let param_count = num_params(op)? << 2;
    
                let params = &bytes[pos..pos + param_count];
                pos += param_count;

                let command = GpuCommand::from_bytes(op, params)?;

                render_cmds.push(command);
            }
        }

        Ok(GpuCommandList {
            render_cmds
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        let extension_nops = vec![GpuCommand::Nop; self.nop_padding_ammount()];

        let mut padded_cmds_iter = self.render_cmds.iter().chain(extension_nops.iter());

        let mut offset = 0;
        while let (
            Some(cmd_0),
            Some(cmd_1),
            Some(cmd_2),
            Some(cmd_3)
        ) = (
            padded_cmds_iter.next(),
            padded_cmds_iter.next(),
            padded_cmds_iter.next(),
            padded_cmds_iter.next()
        ) {
            let commands = [cmd_0, cmd_1, cmd_2, cmd_3];

            buffer[offset..offset + 4].copy_from_slice(
                &commands.iter()
                    .map(|cmd| cmd.op_code())
                    .collect::<Result<Vec<u8>, AppError>>()?
            );

            offset += 4;

            for command in commands {
                let param_count = num_params(command.op_code()?)?;
                let param_bytes_amount = param_count << 2;

                let params_buffer = &mut buffer[offset..offset + param_bytes_amount];
                
                command.write_params_bytes(params_buffer)?;
                offset += param_bytes_amount;
            }
        }

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.render_cmds.len() + self.nop_padding_ammount() + // 1 byte for each command code
        self.render_cmds.iter()
            .map(|cmd| num_params(cmd.op_code().unwrap()).unwrap() << 2) // 4 bytes for each parameter
            .sum::<usize>()
    }

    pub fn clear(&mut self) {
        self.render_cmds.clear();
    }

    pub fn push(&mut self, command: GpuCommand) {
        self.render_cmds.push(command);
    }

    pub fn extend(&mut self, commands: Vec<GpuCommand>) {
        self.render_cmds.extend(commands);
    }

    pub fn get(&self, index: usize) -> Option<&GpuCommand> {
        self.render_cmds.get(index)
    }

    pub fn get_all(&self) -> &[GpuCommand] {
        &self.render_cmds
    }

    pub fn iter(&self) -> impl Iterator<Item = &GpuCommand> {
        self.render_cmds.iter()
    }
}

// Index and size management helpers
impl GpuCommandList {
    fn nop_padding_ammount(&self) -> usize {
        let length = self.render_cmds.len();

        let next_multiple_of_4 = get_4_byte_alignment(length);
        let padding = next_multiple_of_4 - length;

        padding
    }
}

fn num_params(opcode: u8) -> Result<usize, AppError> {
    let opcode = opcode as usize;
    if opcode >= SIZES.len() || SIZES[opcode] == -1 {
        return Err(AppError::new(&format!("Unexpected opcode: 0x{:02X}", opcode)));
    }

    Ok(SIZES[opcode] as usize)
}

#[derive(Debug, Clone)]
pub enum GpuCommand {
    Nop, // 0x00
    Unknown0x10(Box<Unknown0x10Params>), // 0x10
    Unknown0x11, // 0x11
    Unknown0x12(Box<Unknown0x12Params>), // 0x12
    Unknown0x13(Box<Unknown0x13Params>), // 0x13
    MtxRestore(Box<MtxRestoreParams>), // 0x14
    Unknown0x15, // 0x15
    Unknown0x16(Box<Unknown0x16Params>), // 0x16
    Unknown0x17(Box<Unknown0x17Params>), // 0x17
    Unknown0x18(Box<Unknown0x18Params>), // 0x18
    Unknown0x19(Box<Unknown0x19Params>), // 0x19
    Unknown0x1A(Box<Unknown0x1AParams>), // 0x1A
    MtxScale(Box<MtxScaleParams>), // 0x1B
    Unknown0x1C(Box<Unknown0x1CParams>), // 0x1C
    Color(Box<ColorParams>), // 0x20
    Normal(Box<NormalParams>), // 0x21
    TexCoord(Box<TexCoordParams>), // 0x22
    Vtx16(Box<Vtx16Params>), // 0x23
    Vtx10(Box<Vtx10Params>), // 0x24
    VtxXY(Box<VtxXYParams>), // 0x25
    VtxXZ(Box<VtxXZParams>), // 0x26
    VtxYZ(Box<VtxYZParams>), // 0x27
    VtxDiff(Box<VtxDiffParams>), // 0x28
    Unknown0x29(Box<Unknown0x29Params>), // 0x29
    Unknown0x2A(Box<Unknown0x2AParams>), // 0x2A
    Unknown0x2B(Box<Unknown0x2BParams>), // 0x2B
    Unknown0x30(Box<Unknown0x30Params>), // 0x30
    Unknown0x31(Box<Unknown0x31Params>), // 0x31
    Unknown0x32(Box<Unknown0x32Params>), // 0x32
    Unknown0x33(Box<Unknown0x33Params>), // 0x33
    Unknown0x34(Box<Unknown0x34Params>), // 0x34
    BeginVtxs(Box<BeginVtxsParams>), // 0x40
    EndVtxs // 0x41
}

impl GpuCommand {
    pub fn from_bytes(op_code: u8, params: &[u8]) -> Result<GpuCommand, AppError> {
        let command = match op_code {
            0x00 => GpuCommand::Nop,
            0x10 => {
                let params = Unknown0x10Params::from_bytes(params)?;
                GpuCommand::Unknown0x10(Box::new(params))
            },
            0x11 => GpuCommand::Unknown0x11,
            0x12 => {
                let params = Unknown0x12Params::from_bytes(params)?;
                GpuCommand::Unknown0x12(Box::new(params))
            },
            0x13 => {
                let params = Unknown0x13Params::from_bytes(params)?;
                GpuCommand::Unknown0x13(Box::new(params))
            },
            0x14 => {
                let params = MtxRestoreParams::from_bytes(params)?;
                GpuCommand::MtxRestore(Box::new(params))
            },
            0x15 => GpuCommand::Unknown0x15,
            0x16 => {
                let params = Unknown0x16Params::from_bytes(params)?;
                GpuCommand::Unknown0x16(Box::new(params))
            },
            0x17 => {
                let params = Unknown0x17Params::from_bytes(params)?;
                GpuCommand::Unknown0x17(Box::new(params))
            },
            0x18 => {
                let params = Unknown0x18Params::from_bytes(params)?;
                GpuCommand::Unknown0x18(Box::new(params))
            },
            0x19 => {
                let params = Unknown0x19Params::from_bytes(params)?;
                GpuCommand::Unknown0x19(Box::new(params))
            },
            0x1A => {
                let params = Unknown0x1AParams::from_bytes(params)?;
                GpuCommand::Unknown0x1A(Box::new(params))
            },
            0x1B => {
                let params = MtxScaleParams::from_bytes(params)?;
                GpuCommand::MtxScale(Box::new(params))
            },
            0x1C => {
                let params = Unknown0x1CParams::from_bytes(params)?;
                GpuCommand::Unknown0x1C(Box::new(params))
            },
            0x20 => {
                let params = ColorParams::from_bytes(params)?;
                GpuCommand::Color(Box::new(params))
            },
            0x21 => {
                let params = NormalParams::from_bytes(params)?;
                GpuCommand::Normal(Box::new(params))
            },
            0x22 => {
                let params = TexCoordParams::from_bytes(params)?;
                GpuCommand::TexCoord(Box::new(params))
            },
            0x23 => {
                let params = Vtx16Params::from_bytes(params)?;
                GpuCommand::Vtx16(Box::new(params))
            },
            0x24 => {
                let params = Vtx10Params::from_bytes(params)?;
                GpuCommand::Vtx10(Box::new(params))
            },
            0x25 => {
                let params = VtxXYParams::from_bytes(params)?;
                GpuCommand::VtxXY(Box::new(params))
            },
            0x26 => {
                let params = VtxXZParams::from_bytes(params)?;
                GpuCommand::VtxXZ(Box::new(params))
            },
            0x27 => {
                let params = VtxYZParams::from_bytes(params)?;
                GpuCommand::VtxYZ(Box::new(params))
            },
            0x28 => {
                let params = VtxDiffParams::from_bytes(params)?;
                GpuCommand::VtxDiff(Box::new(params))
            },
            0x29 => {
                let params = Unknown0x29Params::from_bytes(params)?;
                GpuCommand::Unknown0x29(Box::new(params))
            },
            0x2A => {
                let params = Unknown0x2AParams::from_bytes(params)?;
                GpuCommand::Unknown0x2A(Box::new(params))
            },
            0x2B => {
                let params = Unknown0x2BParams::from_bytes(params)?;
                GpuCommand::Unknown0x2B(Box::new(params))
            },
            0x30 => {
                let params = Unknown0x30Params::from_bytes(params)?;
                GpuCommand::Unknown0x30(Box::new(params))
            },
            0x31 => {
                let params = Unknown0x31Params::from_bytes(params)?;
                GpuCommand::Unknown0x31(Box::new(params))
            },
            0x32 => {
                let params = Unknown0x32Params::from_bytes(params)?;
                GpuCommand::Unknown0x32(Box::new(params))
            },
            0x33 => {
                let params = Unknown0x33Params::from_bytes(params)?;
                GpuCommand::Unknown0x33(Box::new(params))
            },
            0x34 => {
                let params = Unknown0x34Params::from_bytes(params)?;
                GpuCommand::Unknown0x34(Box::new(params))
            },
            0x40 => {
                let params = BeginVtxsParams::from_bytes(params)?;
                GpuCommand::BeginVtxs(Box::new(params))
            },
            0x41 => GpuCommand::EndVtxs,
            _ => return Err(AppError::new(&format!("Unknown command: 0x{:02X}", op_code))),
        };

        Ok(command)
    }

    pub fn op_code(&self) -> Result<u8, AppError> {
        let op_code = match self {
            GpuCommand::Nop => 0x00,
            GpuCommand::Unknown0x10(_) => 0x10,
            GpuCommand::Unknown0x11 => 0x11,
            GpuCommand::Unknown0x12(_) => 0x12,
            GpuCommand::Unknown0x13(_) => 0x13,
            GpuCommand::MtxRestore(_) => 0x14,
            GpuCommand::Unknown0x15 => 0x15,
            GpuCommand::Unknown0x16(_) => 0x16,
            GpuCommand::Unknown0x17(_) => 0x17,
            GpuCommand::Unknown0x18(_) => 0x18,
            GpuCommand::Unknown0x19(_) => 0x19,
            GpuCommand::Unknown0x1A(_) => 0x1A,
            GpuCommand::MtxScale(_) => 0x1B,
            GpuCommand::Unknown0x1C(_) => 0x1C,
            GpuCommand::Color(_) => 0x20,
            GpuCommand::Normal(_) => 0x21,
            GpuCommand::TexCoord(_) => 0x22,
            GpuCommand::Vtx16(_) => 0x23,
            GpuCommand::Vtx10(_) => 0x24,
            GpuCommand::VtxXY(_) => 0x25,
            GpuCommand::VtxXZ(_) => 0x26,
            GpuCommand::VtxYZ(_) => 0x27,
            GpuCommand::VtxDiff(_) => 0x28,
            GpuCommand::Unknown0x29(_) => 0x29,
            GpuCommand::Unknown0x2A(_) => 0x2A,
            GpuCommand::Unknown0x2B(_) => 0x2B,
            GpuCommand::Unknown0x30(_) => 0x30,
            GpuCommand::Unknown0x31(_) => 0x31,
            GpuCommand::Unknown0x32(_) => 0x32,
            GpuCommand::Unknown0x33(_) => 0x33,
            GpuCommand::Unknown0x34(_) => 0x34,
            GpuCommand::BeginVtxs(_) => 0x40,
            GpuCommand::EndVtxs => 0x41
        };

        Ok(op_code)
    }

    pub fn write_params_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        match self {
            GpuCommand::Nop => {},
            GpuCommand::Unknown0x10(unknown0x10_params) => {
                unknown0x10_params.write_bytes(buffer)?;
            },
            GpuCommand::Unknown0x11 => {},
            GpuCommand::Unknown0x12(unknown0x12_params) => {
                unknown0x12_params.write_bytes(buffer)?;
            },
            GpuCommand::Unknown0x13(unknown0x13_params) => {
                unknown0x13_params.write_bytes(buffer)?;
            },
            GpuCommand::MtxRestore(mtx_restore_params) => {
                mtx_restore_params.write_bytes(buffer)?;
            },
            GpuCommand::Unknown0x15 => {},
            GpuCommand::Unknown0x16(unknown0x16_params) => {
                unknown0x16_params.write_bytes(buffer)?;
            },
            GpuCommand::Unknown0x17(unknown0x17_params) => {
                unknown0x17_params.write_bytes(buffer)?;
            },
            GpuCommand::Unknown0x18(unknown0x18_params) => {
                unknown0x18_params.write_bytes(buffer)?;
            },
            GpuCommand::Unknown0x19(unknown0x19_params) => {
                unknown0x19_params.write_bytes(buffer)?;
            },
            GpuCommand::Unknown0x1A(unknown0x1a_params) => {
                unknown0x1a_params.write_bytes(buffer)?;
            },
            GpuCommand::MtxScale(mtx_scale_params) => {
                mtx_scale_params.write_bytes(buffer)?;
            },
            GpuCommand::Unknown0x1C(unknown0x1c_params) => {
                unknown0x1c_params.write_bytes(buffer)?;
            },
            GpuCommand::Color(color_params) => {
                color_params.write_bytes(buffer)?;
            },
            GpuCommand::Normal(normal_params) => {
                normal_params.write_bytes(buffer)?;
            },
            GpuCommand::TexCoord(tex_coord_params) => {
                tex_coord_params.write_bytes(buffer)?;
            },
            GpuCommand::Vtx16(vtx16_params) => {
                vtx16_params.write_bytes(buffer)?;
            },
            GpuCommand::Vtx10(vtx10_params) => {
                vtx10_params.write_bytes(buffer)?;
            },
            GpuCommand::VtxXY(vtx_xyparams) => {
                vtx_xyparams.write_bytes(buffer)?;
            },
            GpuCommand::VtxXZ(vtx_xzparams) => {
                vtx_xzparams.write_bytes(buffer)?;
            },
            GpuCommand::VtxYZ(vtx_yzparams) => {
                vtx_yzparams.write_bytes(buffer)?;
            },
            GpuCommand::VtxDiff(vtx_diff_params) => {
                vtx_diff_params.write_bytes(buffer)?;
            },
            GpuCommand::Unknown0x29(unknown0x29_params) => {
                unknown0x29_params.write_bytes(buffer)?;
            },
            GpuCommand::Unknown0x2A(unknown0x2a_params) => {
                unknown0x2a_params.write_bytes(buffer)?;
            },
            GpuCommand::Unknown0x2B(unknown0x2b_params) => {
                unknown0x2b_params.write_bytes(buffer)?;
            },
            GpuCommand::Unknown0x30(unknown0x30_params) => {
                unknown0x30_params.write_bytes(buffer)?;
            },
            GpuCommand::Unknown0x31(unknown0x31_params) => {
                unknown0x31_params.write_bytes(buffer)?;
            },
            GpuCommand::Unknown0x32(unknown0x32_params) => {
                unknown0x32_params.write_bytes(buffer)?;
            },
            GpuCommand::Unknown0x33(unknown0x33_params) => {
                unknown0x33_params.write_bytes(buffer)?;
            },
            GpuCommand::Unknown0x34(unknown0x34_params) => {
                unknown0x34_params.write_bytes(buffer)?;
            },
            GpuCommand::BeginVtxs(begin_vtxs_params) => {
                begin_vtxs_params.write_bytes(buffer)?;
            },
            GpuCommand::EndVtxs => {},
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Unknown0x10Params {
    pub unknown: u32
}

impl Unknown0x10Params {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x10Params, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("Unknown0x10Params needs at least 4 bytes"));
        }

        let unknown = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        Ok(Unknown0x10Params {
            unknown
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for Unknown0x10Params"));
        }

        buffer[0..4].copy_from_slice(&self.unknown.to_le_bytes());

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Unknown0x12Params {
    pub unknown: u32
}

impl Unknown0x12Params {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x12Params, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("Unknown0x12Params needs at least 4 bytes"));
        }

        let unknown = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        Ok(Unknown0x12Params {
            unknown
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for Unknown0x12Params"));
        }

        buffer[0..4].copy_from_slice(&self.unknown.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x13Params {
    pub unknown: u32
}

impl Unknown0x13Params {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x13Params, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("Unknown0x13Params needs at least 4 bytes"));
        }

        let unknown = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        Ok(Unknown0x13Params {
            unknown
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for Unknown0x13Params"));
        }

        buffer[0..4].copy_from_slice(&self.unknown.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct MtxRestoreParams {
    pub index: u32
}

impl MtxRestoreParams {
    pub fn from_bytes(bytes: &[u8]) -> Result<MtxRestoreParams, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("MtxRestoreParams needs at least 4 bytes"));
        }

        let index = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        Ok(MtxRestoreParams {
            index
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for MtxRestoreParams"));
        }

        buffer[0..4].copy_from_slice(&self.index.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x16Params {
    pub unknown_0: u32,
    pub unknown_1: u32,
    pub unknown_2: u32,
    pub unknown_3: u32,
    pub unknown_4: u32,
    pub unknown_5: u32,
    pub unknown_6: u32,
    pub unknown_7: u32,
    pub unknown_8: u32,
    pub unknown_9: u32,
    pub unknown_10: u32,
    pub unknown_11: u32,
    pub unknown_12: u32,
    pub unknown_13: u32,
    pub unknown_14: u32,
    pub unknown_15: u32
}

impl Unknown0x16Params {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x16Params, AppError> {
        if bytes.len() < 64 {
            return Err(AppError::new("Unknown0x16Params needs at least 64 bytes"));
        }

        let unknown_0 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let unknown_1 = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let unknown_2 = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let unknown_3 = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let unknown_4 = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let unknown_5 = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        let unknown_6 = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
        let unknown_7 = u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]);
        let unknown_8 = u32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]);
        let unknown_9 = u32::from_le_bytes([bytes[36], bytes[37], bytes[38], bytes[39]]);
        let unknown_10 = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]);
        let unknown_11 = u32::from_le_bytes([bytes[44], bytes[45], bytes[46], bytes[47]]);
        let unknown_12 = u32::from_le_bytes([bytes[48], bytes[49], bytes[50], bytes[51]]);
        let unknown_13 = u32::from_le_bytes([bytes[52], bytes[53], bytes[54], bytes[55]]);
        let unknown_14 = u32::from_le_bytes([bytes[56], bytes[57], bytes[58], bytes[59]]);
        let unknown_15 = u32::from_le_bytes([bytes[60], bytes[61], bytes[62], bytes[63]]);

        Ok(Unknown0x16Params {
            unknown_0,
            unknown_1,
            unknown_2,
            unknown_3,
            unknown_4,
            unknown_5,
            unknown_6,
            unknown_7,
            unknown_8,
            unknown_9,
            unknown_10,
            unknown_11,
            unknown_12,
            unknown_13,
            unknown_14,
            unknown_15
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 64 {
            return Err(AppError::new("Buffer too small for Unknown0x16Params"));
        }

        buffer[0..4].copy_from_slice(&self.unknown_0.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.unknown_1.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.unknown_2.to_le_bytes());
        buffer[12..16].copy_from_slice(&self.unknown_3.to_le_bytes());
        buffer[16..20].copy_from_slice(&self.unknown_4.to_le_bytes());
        buffer[20..24].copy_from_slice(&self.unknown_5.to_le_bytes());
        buffer[24..28].copy_from_slice(&self.unknown_6.to_le_bytes());
        buffer[28..32].copy_from_slice(&self.unknown_7.to_le_bytes());
        buffer[32..36].copy_from_slice(&self.unknown_8.to_le_bytes());
        buffer[36..40].copy_from_slice(&self.unknown_9.to_le_bytes());
        buffer[40..44].copy_from_slice(&self.unknown_10.to_le_bytes());
        buffer[44..48].copy_from_slice(&self.unknown_11.to_le_bytes());
        buffer[48..52].copy_from_slice(&self.unknown_12.to_le_bytes());
        buffer[52..56].copy_from_slice(&self.unknown_13.to_le_bytes());
        buffer[56..60].copy_from_slice(&self.unknown_14.to_le_bytes());
        buffer[60..64].copy_from_slice(&self.unknown_15.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x17Params {
    pub unknown_0: u32,
    pub unknown_1: u32,
    pub unknown_2: u32,
    pub unknown_3: u32,
    pub unknown_4: u32,
    pub unknown_5: u32,
    pub unknown_6: u32,
    pub unknown_7: u32,
    pub unknown_8: u32,
    pub unknown_9: u32,
    pub unknown_10: u32,
    pub unknown_11: u32
}

impl Unknown0x17Params {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x17Params, AppError> {
        if bytes.len() < 48 {
            return Err(AppError::new("Unknown0x17Params needs at least 48 bytes"));
        }

        let unknown_0 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let unknown_1 = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let unknown_2 = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let unknown_3 = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let unknown_4 = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let unknown_5 = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        let unknown_6 = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
        let unknown_7 = u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]);
        let unknown_8 = u32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]);
        let unknown_9 = u32::from_le_bytes([bytes[36], bytes[37], bytes[38], bytes[39]]);
        let unknown_10 = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]);
        let unknown_11 = u32::from_le_bytes([bytes[44], bytes[45], bytes[46], bytes[47]]);

        Ok(Unknown0x17Params {
            unknown_0,
            unknown_1,
            unknown_2,
            unknown_3,
            unknown_4,
            unknown_5,
            unknown_6,
            unknown_7,
            unknown_8,
            unknown_9,
            unknown_10,
            unknown_11
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 48 {
            return Err(AppError::new("Buffer too small for Unknown0x17Params"));
        }

        buffer[0..4].copy_from_slice(&self.unknown_0.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.unknown_1.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.unknown_2.to_le_bytes());
        buffer[12..16].copy_from_slice(&self.unknown_3.to_le_bytes());
        buffer[16..20].copy_from_slice(&self.unknown_4.to_le_bytes());
        buffer[20..24].copy_from_slice(&self.unknown_5.to_le_bytes());
        buffer[24..28].copy_from_slice(&self.unknown_6.to_le_bytes());
        buffer[28..32].copy_from_slice(&self.unknown_7.to_le_bytes());
        buffer[32..36].copy_from_slice(&self.unknown_8.to_le_bytes());
        buffer[36..40].copy_from_slice(&self.unknown_9.to_le_bytes());
        buffer[40..44].copy_from_slice(&self.unknown_10.to_le_bytes());
        buffer[44..48].copy_from_slice(&self.unknown_11.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x18Params {
    pub unknown_0: u32,
    pub unknown_1: u32,
    pub unknown_2: u32,
    pub unknown_3: u32,
    pub unknown_4: u32,
    pub unknown_5: u32,
    pub unknown_6: u32,
    pub unknown_7: u32,
    pub unknown_8: u32,
    pub unknown_9: u32,
    pub unknown_10: u32,
    pub unknown_11: u32,
    pub unknown_12: u32,
    pub unknown_13: u32,
    pub unknown_14: u32,
    pub unknown_15: u32
}

impl Unknown0x18Params {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x18Params, AppError> {
        if bytes.len() < 64 {
            return Err(AppError::new("Unknown0x18Params needs at least 64 bytes"));
        }

        let unknown_0 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let unknown_1 = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let unknown_2 = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let unknown_3 = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let unknown_4 = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let unknown_5 = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        let unknown_6 = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
        let unknown_7 = u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]);
        let unknown_8 = u32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]);
        let unknown_9 = u32::from_le_bytes([bytes[36], bytes[37], bytes[38], bytes[39]]);
        let unknown_10 = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]);
        let unknown_11 = u32::from_le_bytes([bytes[44], bytes[45], bytes[46], bytes[47]]);
        let unknown_12 = u32::from_le_bytes([bytes[48], bytes[49], bytes[50], bytes[51]]);
        let unknown_13 = u32::from_le_bytes([bytes[52], bytes[53], bytes[54], bytes[55]]);
        let unknown_14 = u32::from_le_bytes([bytes[56], bytes[57], bytes[58], bytes[59]]);
        let unknown_15 = u32::from_le_bytes([bytes[60], bytes[61], bytes[62], bytes[63]]);

        Ok(Unknown0x18Params {
            unknown_0,
            unknown_1,
            unknown_2,
            unknown_3,
            unknown_4,
            unknown_5,
            unknown_6,
            unknown_7,
            unknown_8,
            unknown_9,
            unknown_10,
            unknown_11,
            unknown_12,
            unknown_13,
            unknown_14,
            unknown_15
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 64 {
            return Err(AppError::new("Buffer too small for Unknown0x18Params"));
        }

        buffer[0..4].copy_from_slice(&self.unknown_0.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.unknown_1.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.unknown_2.to_le_bytes());
        buffer[12..16].copy_from_slice(&self.unknown_3.to_le_bytes());
        buffer[16..20].copy_from_slice(&self.unknown_4.to_le_bytes());
        buffer[20..24].copy_from_slice(&self.unknown_5.to_le_bytes());
        buffer[24..28].copy_from_slice(&self.unknown_6.to_le_bytes());
        buffer[28..32].copy_from_slice(&self.unknown_7.to_le_bytes());
        buffer[32..36].copy_from_slice(&self.unknown_8.to_le_bytes());
        buffer[36..40].copy_from_slice(&self.unknown_9.to_le_bytes());
        buffer[40..44].copy_from_slice(&self.unknown_10.to_le_bytes());
        buffer[44..48].copy_from_slice(&self.unknown_11.to_le_bytes());
        buffer[48..52].copy_from_slice(&self.unknown_12.to_le_bytes());
        buffer[52..56].copy_from_slice(&self.unknown_13.to_le_bytes());
        buffer[56..60].copy_from_slice(&self.unknown_14.to_le_bytes());
        buffer[60..64].copy_from_slice(&self.unknown_15.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x19Params {
    pub unknown_0: u32,
    pub unknown_1: u32,
    pub unknown_2: u32,
    pub unknown_3: u32,
    pub unknown_4: u32,
    pub unknown_5: u32,
    pub unknown_6: u32,
    pub unknown_7: u32,
    pub unknown_8: u32,
    pub unknown_9: u32,
    pub unknown_10: u32,
    pub unknown_11: u32
}

impl Unknown0x19Params {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x19Params, AppError> {
        if bytes.len() < 48 {
            return Err(AppError::new("Unknown0x19Params needs at least 48 bytes"));
        }

        let unknown_0 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let unknown_1 = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let unknown_2 = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let unknown_3 = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let unknown_4 = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let unknown_5 = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        let unknown_6 = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
        let unknown_7 = u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]);
        let unknown_8 = u32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]);
        let unknown_9 = u32::from_le_bytes([bytes[36], bytes[37], bytes[38], bytes[39]]);
        let unknown_10 = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]);
        let unknown_11 = u32::from_le_bytes([bytes[44], bytes[45], bytes[46], bytes[47]]);

        Ok(Unknown0x19Params {
            unknown_0,
            unknown_1,
            unknown_2,
            unknown_3,
            unknown_4,
            unknown_5,
            unknown_6,
            unknown_7,
            unknown_8,
            unknown_9,
            unknown_10,
            unknown_11
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 48 {
            return Err(AppError::new("Buffer too small for Unknown0x19Params"));
        }

        buffer[0..4].copy_from_slice(&self.unknown_0.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.unknown_1.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.unknown_2.to_le_bytes());
        buffer[12..16].copy_from_slice(&self.unknown_3.to_le_bytes());
        buffer[16..20].copy_from_slice(&self.unknown_4.to_le_bytes());
        buffer[20..24].copy_from_slice(&self.unknown_5.to_le_bytes());
        buffer[24..28].copy_from_slice(&self.unknown_6.to_le_bytes());
        buffer[28..32].copy_from_slice(&self.unknown_7.to_le_bytes());
        buffer[32..36].copy_from_slice(&self.unknown_8.to_le_bytes());
        buffer[36..40].copy_from_slice(&self.unknown_9.to_le_bytes());
        buffer[40..44].copy_from_slice(&self.unknown_10.to_le_bytes());
        buffer[44..48].copy_from_slice(&self.unknown_11.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x1AParams {
    pub unknown_0: u32,
    pub unknown_1: u32,
    pub unknown_2: u32,
    pub unknown_3: u32,
    pub unknown_4: u32,
    pub unknown_5: u32,
    pub unknown_6: u32,
    pub unknown_7: u32,
    pub unknown_8: u32
}

impl Unknown0x1AParams {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x1AParams, AppError> {
        if bytes.len() < 36 {
            return Err(AppError::new("Unknown0x1AParams needs at least 36 bytes"));
        }

        let unknown_0 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let unknown_1 = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let unknown_2 = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let unknown_3 = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let unknown_4 = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let unknown_5 = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        let unknown_6 = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
        let unknown_7 = u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]);
        let unknown_8 = u32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]);

        Ok(Unknown0x1AParams {
            unknown_0,
            unknown_1,
            unknown_2,
            unknown_3,
            unknown_4,
            unknown_5,
            unknown_6,
            unknown_7,
            unknown_8
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 36 {
            return Err(AppError::new("Buffer too small for Unknown0x1AParams"));
        }

        buffer[0..4].copy_from_slice(&self.unknown_0.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.unknown_1.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.unknown_2.to_le_bytes());
        buffer[12..16].copy_from_slice(&self.unknown_3.to_le_bytes());
        buffer[16..20].copy_from_slice(&self.unknown_4.to_le_bytes());
        buffer[20..24].copy_from_slice(&self.unknown_5.to_le_bytes());
        buffer[24..28].copy_from_slice(&self.unknown_6.to_le_bytes());
        buffer[28..32].copy_from_slice(&self.unknown_7.to_le_bytes());
        buffer[32..36].copy_from_slice(&self.unknown_8.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct MtxScaleParams {
    // Scale in each axis
    pub x: Fixed1_19_12,
    pub y: Fixed1_19_12,
    pub z: Fixed1_19_12
}

impl MtxScaleParams {
    pub fn from_bytes(bytes: &[u8]) -> Result<MtxScaleParams, AppError> {
        if bytes.len() < 12 {
            return Err(AppError::new("MtxScaleParams needs at least 12 bytes"));
        }

        let x_i32 = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let y_i32 = i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let z_i32 = i32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

        let x = Fixed1_19_12::from(x_i32);
        let y = Fixed1_19_12::from(y_i32);
        let z = Fixed1_19_12::from(z_i32);

        Ok(MtxScaleParams {
            x,
            y,
            z
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 12 {
            return Err(AppError::new("Buffer too small for MtxScaleParams"));
        }

        buffer[0..4].copy_from_slice(&self.x.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.y.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.z.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x1CParams {
    pub unknown_0: u32,
    pub unknown_1: u32,
    pub unknown_2: u32
}

impl Unknown0x1CParams {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x1CParams, AppError> {
        if bytes.len() < 12 {
            return Err(AppError::new("Unknown0x1CParams needs at least 12 bytes"));
        }

        let unknown_0 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let unknown_1 = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let unknown_2 = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

        Ok(Unknown0x1CParams {
            unknown_0,
            unknown_1,
            unknown_2
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 12 {
            return Err(AppError::new("Buffer too small for Unknown0x1CParams"));
        }

        buffer[0..4].copy_from_slice(&self.unknown_0.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.unknown_1.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.unknown_2.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct ColorParams {
    pub r: u8, // 5 bits [0, 5)
    pub g: u8, // 5 bits [5, 10)
    pub b: u8, // 5 bits [10, 15)
}

impl ColorParams {
    pub fn from_bytes(bytes: &[u8]) -> Result<ColorParams, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("ColorParams needs at least 4 bytes"));
        }

        let full = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let r = (full & 0x1F) as u8;
        let g = ((full >> 5) & 0x1F) as u8;
        let b = ((full >> 10) & 0x1F) as u8;

        Ok(ColorParams {
            r,
            g,
            b
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for ColorParams"));
        }

        let full = (self.r as u32) | ((self.g as u32) << 5) | ((self.b as u32) << 10);

        buffer[0..4].copy_from_slice(&full.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct NormalParams {
    pub x: Fixed1_0_9,
    pub y: Fixed1_0_9,
    pub z: Fixed1_0_9
}

impl NormalParams {
    pub fn from_bytes(bytes: &[u8]) -> Result<NormalParams, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("NormalParams needs at least 4 bytes"));
        }

        let full = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let x_i16 = (full & 0x3FF) as i16;
        let y_i16 = ((full >> 10) & 0x3FF) as i16;
        let z_i16 = ((full >> 20) & 0x3FF) as i16;


        let x = Fixed1_0_9::from(x_i16);
        let y = Fixed1_0_9::from(y_i16);
        let z = Fixed1_0_9::from(z_i16);

        Ok(NormalParams {
            x,
            y,
            z
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for NormalParams"));
        }

        let x_i16 = self.x.to_i16() as u32;
        let y_i16 = self.y.to_i16() as u32;
        let z_i16 = self.z.to_i16() as u32;

        let full = (x_i16 & 0x3FF) | ((y_i16 & 0x3FF) << 10) | ((z_i16 & 0x3FF) << 20);

        buffer[0..4].copy_from_slice(&full.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct TexCoordParams {
    pub s: Fixed1_11_4,
    pub t: Fixed1_11_4
}

impl TexCoordParams {
    pub fn from_bytes(bytes: &[u8]) -> Result<TexCoordParams, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("TexCoordParams needs at least 4 bytes"));
        }

        let full = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let s_i16 = (full & 0xFFFF) as i16;
        let t_i16 = ((full >> 16) & 0xFFFF) as i16;

        let s = Fixed1_11_4::from(s_i16);
        let t = Fixed1_11_4::from(t_i16);

        Ok(TexCoordParams {
            s,
            t
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for TexCoordParams"));
        }

        let s_i16 = self.s.to_i16() as u32;
        let t_i16 = self.t.to_i16() as u32;

        let full = (s_i16 & 0xFFFF) | ((t_i16 & 0xFFFF) << 16);

        buffer[0..4].copy_from_slice(&full.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Vtx16Params {
    pub x: Fixed1_3_12,
    pub y: Fixed1_3_12,
    pub z: Fixed1_3_12
}

impl Vtx16Params {
    pub fn from_bytes(bytes: &[u8]) -> Result<Vtx16Params, AppError> {
        if bytes.len() < 8 {
            return Err(AppError::new("Vtx16Params needs at least 8 bytes"));
        }

        let full_0 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let full_1 = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

        let x_i16 = (full_0 & 0xFFFF) as i16;
        let y_i16 = ((full_0 >> 16) & 0xFFFF) as i16;
        let z_i16 = (full_1 & 0xFFFF) as i16;

        let x = Fixed1_3_12::from(x_i16);
        let y = Fixed1_3_12::from(y_i16);
        let z = Fixed1_3_12::from(z_i16);

        Ok(Vtx16Params {
            x,
            y,
            z
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 8 {
            return Err(AppError::new("Buffer too small for Vtx16Params"));
        }

        let x_i16 = self.x.to_i16() as u32;
        let y_i16 = self.y.to_i16() as u32;
        let z_i16 = self.z.to_i16() as u32;

        let full_0 = (x_i16 & 0xFFFF) | ((y_i16 & 0xFFFF) << 16);
        let full_1 = z_i16 & 0xFFFF;

        buffer[0..4].copy_from_slice(&full_0.to_le_bytes());
        buffer[4..8].copy_from_slice(&full_1.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Vtx10Params {
    pub x: Fixed1_3_6,
    pub y: Fixed1_3_6,
    pub z: Fixed1_3_6,
}

impl Vtx10Params {
    pub fn from_bytes(bytes: &[u8]) -> Result<Vtx10Params, AppError> {
        if bytes.len() < 8 {
            return Err(AppError::new("Vtx10Params needs at least 8 bytes"));
        }

        let full_0 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let x_i16 = (full_0 & 0x3FF) as i16;
        let y_i16 = ((full_0 >> 10) & 0x3FF) as i16;
        let z_i16 = (full_0 >> 20 & 0x3FF) as i16;

        let x = Fixed1_3_6::from(x_i16);
        let y = Fixed1_3_6::from(y_i16);
        let z = Fixed1_3_6::from(z_i16);

        Ok(Vtx10Params {
            x,
            y,
            z
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 8 {
            return Err(AppError::new("Buffer too small for Vtx10Params"));
        }

        let x_i16 = self.x.to_i16() as u32;
        let y_i16 = self.y.to_i16() as u32;
        let z_i16 = self.z.to_i16() as u32;

        let full_0 = (x_i16 & 0x3FF) | ((y_i16 & 0x3FF) << 10) | ((z_i16 & 0x3FF) << 20);

        buffer[0..4].copy_from_slice(&full_0.to_le_bytes());
        buffer[4..8].copy_from_slice(&[0, 0, 0, 0]);

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct VtxXYParams {
    pub x: Fixed1_3_12,
    pub y: Fixed1_3_12
}

impl VtxXYParams {
    pub fn from_bytes(bytes: &[u8]) -> Result<VtxXYParams, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("VtxXYParams needs at least 4 bytes"));
        }

        let full_0 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let x_i16 = (full_0 & 0xFFFF) as i16;
        let y_i16 = ((full_0 >> 16) & 0xFFFF) as i16;

        let x = Fixed1_3_12::from(x_i16);
        let y = Fixed1_3_12::from(y_i16);

        Ok(VtxXYParams {
            x,
            y
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for VtxXYParams"));
        }

        let x_i16 = self.x.to_i16() as u32;
        let y_i16 = self.y.to_i16() as u32;

        let full_0 = (x_i16 & 0xFFFF) | ((y_i16 & 0xFFFF) << 16);

        buffer[0..4].copy_from_slice(&full_0.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct VtxXZParams {
    pub x: Fixed1_3_12,
    pub z: Fixed1_3_12
}

impl VtxXZParams {
    pub fn from_bytes(bytes: &[u8]) -> Result<VtxXZParams, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("VtxXZParams needs at least 4 bytes"));
        }

        let full_0 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let x_i16 = (full_0 & 0xFFFF) as i16;
        let z_i16 = ((full_0 >> 16) & 0xFFFF) as i16;

        let x = Fixed1_3_12::from(x_i16);
        let z = Fixed1_3_12::from(z_i16);

        Ok(VtxXZParams {
            x,
            z
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for VtxXZParams"));
        }

        let x_i16 = self.x.to_i16() as u32;
        let z_i16 = self.z.to_i16() as u32;

        let full_0 = (x_i16 & 0xFFFF) | ((z_i16 & 0xFFFF) << 16);

        buffer[0..4].copy_from_slice(&full_0.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct VtxYZParams {
    pub y: Fixed1_3_12,
    pub z: Fixed1_3_12
}

impl VtxYZParams {
    pub fn from_bytes(bytes: &[u8]) -> Result<VtxYZParams, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("VtxYZParams needs at least 4 bytes"));
        }

        let full_0 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let y_i16 = (full_0 & 0xFFFF) as i16;
        let z_i16 = ((full_0 >> 16) & 0xFFFF) as i16;

        let y = Fixed1_3_12::from(y_i16);
        let z = Fixed1_3_12::from(z_i16);

        Ok(VtxYZParams {
            y,
            z
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for VtxYZParams"));
        }

        let y_i16 = self.y.to_i16() as u32;
        let z_i16 = self.z.to_i16() as u32;

        let full_0 = (y_i16 & 0xFFFF) | ((z_i16 & 0xFFFF) << 16);

        buffer[0..4].copy_from_slice(&full_0.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct VtxDiffParams {
    pub x: Fixed1_3_12,
    pub y: Fixed1_3_12,
    pub z: Fixed1_3_12
}

impl VtxDiffParams {
    pub fn from_bytes(bytes: &[u8]) -> Result<VtxDiffParams, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("VtxDiffParams needs at least 4 bytes"));
        }

        let full_0 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let x_i16 = (full_0 & 0x3FF) as i16;
        let y_i16 = ((full_0 >> 10) & 0x3FF) as i16;
        let z_i16 = ((full_0 >> 20) & 0x3FF) as i16;

        let x = Fixed1_3_12::from(x_i16);
        let y = Fixed1_3_12::from(y_i16);
        let z = Fixed1_3_12::from(z_i16);

        Ok(VtxDiffParams {
            x,
            y,
            z
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for VtxDiffParams"));
        }

        let x_i16 = self.x.to_i16() as u32;
        let y_i16 = self.y.to_i16() as u32;
        let z_i16 = self.z.to_i16() as u32;

        let full = (x_i16 & 0x3FF) | ((y_i16 & 0x3FF) << 10) | ((z_i16 & 0x3FF) << 20);

        buffer[0..4].copy_from_slice(&full.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x29Params {
    pub unknown: u32
}

impl Unknown0x29Params {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x29Params, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("Unknown0x29Params needs at least 4 bytes"));
        }

        let unknown = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        Ok(Unknown0x29Params {
            unknown
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for Unknown0x29Params"));
        }

        buffer[0..4].copy_from_slice(&self.unknown.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x2AParams {
    pub unknown: u32
}

impl Unknown0x2AParams {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x2AParams, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("Unknown0x2AParams needs at least 4 bytes"));
        }

        let unknown = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        Ok(Unknown0x2AParams {
            unknown
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for Unknown0x2AParams"));
        }

        buffer[0..4].copy_from_slice(&self.unknown.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x2BParams {
    pub unknown: u32
}

impl Unknown0x2BParams {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x2BParams, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("Unknown0x2BParams needs at least 4 bytes"));
        }

        let unknown = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        Ok(Unknown0x2BParams {
            unknown
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for Unknown0x2BParams"));
        }

        buffer[0..4].copy_from_slice(&self.unknown.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x30Params {
    pub unknown: u32
}

impl Unknown0x30Params {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x30Params, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("Unknown0x30Params needs at least 4 bytes"));
        }

        let unknown = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        Ok(Unknown0x30Params {
            unknown
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for Unknown0x30Params"));
        }

        buffer[0..4].copy_from_slice(&self.unknown.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x31Params {
    pub unknown: u32
}

impl Unknown0x31Params {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x31Params, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("Unknown0x31Params needs at least 4 bytes"));
        }

        let unknown = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        Ok(Unknown0x31Params {
            unknown
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for Unknown0x31Params"));
        }

        buffer[0..4].copy_from_slice(&self.unknown.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x32Params {
    pub unknown: u32
}

impl Unknown0x32Params {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x32Params, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("Unknown0x32Params needs at least 4 bytes"));
        }

        let unknown = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        Ok(Unknown0x32Params {
            unknown
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for Unknown0x32Params"));
        }

        buffer[0..4].copy_from_slice(&self.unknown.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x33Params {
    pub unknown: u32
}

impl Unknown0x33Params {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x33Params, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("Unknown0x33Params needs at least 4 bytes"));
        }

        let unknown = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        Ok(Unknown0x33Params {
            unknown
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for Unknown0x33Params"));
        }

        buffer[0..4].copy_from_slice(&self.unknown.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x34Params {
    pub unknown: u32
}

impl Unknown0x34Params {
    pub fn from_bytes(bytes: &[u8]) -> Result<Unknown0x34Params, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("Unknown0x34Params needs at least 4 bytes"));
        }

        let unknown = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        Ok(Unknown0x34Params {
            unknown
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 4 {
            return Err(AppError::new("Buffer too small for Unknown0x34Params"));
        }

        buffer[0..4].copy_from_slice(&self.unknown.to_le_bytes());

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct BeginVtxsParams {
    pub primitive_type: u8
}

impl BeginVtxsParams {
    pub const TRIANGLE: u8 = 0x00;
    pub const QUAD: u8 = 0x01;
    pub const TRIANGLE_STRIP: u8 = 0x02;
    pub const QUAD_STRIP: u8 = 0x03;
    
    pub fn from_bytes(bytes: &[u8]) -> Result<BeginVtxsParams, AppError> {
        if bytes.len() < 1 {
            return Err(AppError::new("BeginVtxsParams needs at least 1 byte"));
        }

        let primitive_type = bytes[0] & 0x03;

        Ok(BeginVtxsParams {
            primitive_type
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 1 {
            return Err(AppError::new("Buffer too small for BeginVtxsParams"));
        }

        buffer[0] = self.primitive_type & 0x03;

        Ok(())
    }
}
