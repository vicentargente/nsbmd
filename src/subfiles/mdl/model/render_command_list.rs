use crate::{debug_info::DebugInfo, error::AppError};

const COMMAND_CODE_MASK: u8 = 0x1F;
const COMMAND_SUBTYPE_MASK: u8 = !COMMAND_CODE_MASK;

#[derive(Debug, Clone)]
pub struct RenderCommandList {
    render_commands: Vec<RenderCommand>,

    // Debug info
    _debug_info: DebugInfo
}

impl RenderCommandList {
    pub fn from_bytes(bytes: &[u8], debug_info: DebugInfo) -> Result<RenderCommandList, AppError> {
        if bytes.len() < 1 {
            return Err(AppError::new("RenderCommandList needs at least 1 byte"));
        }

        let mut render_commands = Vec::new();

        let mut pos = 0;
        loop {
            let op_code = bytes[pos];

            let render_command = RenderCommand::from_bytes(op_code, &bytes[(pos + 1)..])?;

            pos += render_command.size();

            if let RenderCommand::End = render_command {
                render_commands.push(render_command);
                break;
            }

            render_commands.push(render_command);

            if pos >= bytes.len() {
                return Err(AppError::new("RenderCommandList ended unexpectedly"));
            }
        };

        Ok(RenderCommandList {
            render_commands,
            _debug_info: debug_info
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        let mut pos = 0;
        for render_command in self.render_commands.iter() {
            let len = render_command.size();
            if pos + len > buffer.len() {
                return Err(AppError::new(&format!("RenderCommandList needs at least {} bytes to write", pos + len)));
            }

            render_command.write_bytes(&mut buffer[pos..])?;
            pos += len;
        }

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.render_commands.iter().map(|cmd| cmd.size()).sum()
    }

    pub fn clear(&mut self) {
        self.render_commands.clear();
    }

    pub fn push(&mut self, command: RenderCommand) {
        self.render_commands.push(command);
    }

    pub fn extend(&mut self, commands: Vec<RenderCommand>) {
        self.render_commands.extend(commands);
    }

    pub fn get(&self, index: usize) -> Option<&RenderCommand> {
        self.render_commands.get(index)
    }

    pub fn get_all(&self) -> &[RenderCommand] {
        &self.render_commands
    }

    pub fn iter(&self) -> impl Iterator<Item = &RenderCommand> {
        self.render_commands.iter()
    }
}


#[derive(Debug, Clone)]
pub enum RenderCommand {
    Nop(Box<NopData>),
    End,
    Unknown0x02(Box<Unknown0x02Data>),
    LoadMatrixFromStack(Box<LoadMatrixFromStackData>),
    BindMaterial(Box<BindMaterialData>),
    DrawMesh(Box<DrawMeshData>),
    MulCurrentMatrixWithBoneMatrix(Box<MulCurrentMatrixWithBoneMatrixData>),
    Unknown0x07(Box<Unknown0x07Data>),
    Unknown0x08(Box<Unknown0x08Data>),
    CalculateSkinningEquation(Box<CalculateSkinningEquationData>),
    Scale(Box<ScaleData>),
    Unknown0x0C(Box<Unknown0x0CData>),
    Unknown0x0D(Box<Unknown0x0DData>)
}

impl RenderCommand {
    pub fn from_bytes(op_code: u8, tail: &[u8]) -> Result<RenderCommand, AppError> {
        match op_code & COMMAND_CODE_MASK { 
            0x00 => {
                let data = NopData::from_bytes(op_code)?;
                Ok(RenderCommand::Nop(Box::new(data)))
            },
            0x01 => {
                Ok(RenderCommand::End)
            },
            0x02 => {
                let data = Unknown0x02Data::from_bytes(tail)?;
                Ok(RenderCommand::Unknown0x02(Box::new(data)))
            },
            0x03 => {
                let data = LoadMatrixFromStackData::from_bytes(tail)?;
                Ok(RenderCommand::LoadMatrixFromStack(Box::new(data)))
            },
            0x04 => {
                let data = BindMaterialData::from_bytes(op_code, tail)?;
                Ok(RenderCommand::BindMaterial(Box::new(data)))
            },
            0x05 => {
                let data = DrawMeshData::from_bytes(tail)?;
                Ok(RenderCommand::DrawMesh(Box::new(data)))
            },
            0x06 => {
                let data = MulCurrentMatrixWithBoneMatrixData::from_bytes(op_code, tail)?;
                Ok(RenderCommand::MulCurrentMatrixWithBoneMatrix(Box::new(data)))
            },
            0x07 => {
                let data = Unknown0x07Data::from_bytes(op_code, tail)?;
                Ok(RenderCommand::Unknown0x07(Box::new(data)))
            },
            0x08 => {
                let data = Unknown0x08Data::from_bytes(tail)?;
                Ok(RenderCommand::Unknown0x08(Box::new(data)))
            },
            0x09 => {
                let data = CalculateSkinningEquationData::from_bytes(tail)?;
                Ok(RenderCommand::CalculateSkinningEquation(Box::new(data)))
            },
            0x0B => {
                let data = ScaleData::from_bytes(op_code)?;
                Ok(RenderCommand::Scale(Box::new(data)))
            },
            0x0C => {
                let data = Unknown0x0CData::from_bytes(tail)?;
                Ok(RenderCommand::Unknown0x0C(Box::new(data)))
            },
            0x0D => {
                let data = Unknown0x0DData::from_bytes(tail)?;
                Ok(RenderCommand::Unknown0x0D(Box::new(data)))
            },
            _ => {
                Err(AppError::new(&format!("Unknown RenderCommand: 0x{:2X}", op_code)))
            }
        }
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < self.size() {
            return Err(AppError::new(&format!("{:?} needs at least {} bytes to write", self, self.size())));
        }

        match self {
            RenderCommand::Nop(_) => {
                buffer[0] = self.command_code();
            },
            RenderCommand::End => {
                buffer[0] = self.command_code();
            },
            RenderCommand::Unknown0x02(unknown0x02_data) => {
                buffer[0] = self.command_code();
                unknown0x02_data.write_bytes(&mut buffer[1..])?;
            },
            RenderCommand::LoadMatrixFromStack(load_matrix_from_stack_data) => {
                buffer[0] = self.command_code();
                load_matrix_from_stack_data.write_bytes(&mut buffer[1..])?;
            },
            RenderCommand::BindMaterial(bind_material_data) => {
                buffer[0] = self.command_code();
                bind_material_data.write_bytes(&mut buffer[1..])?;
            },
            RenderCommand::DrawMesh(draw_mesh_data) => {
                buffer[0] = self.command_code();
                draw_mesh_data.write_bytes(&mut buffer[1..])?;
            },
            RenderCommand::MulCurrentMatrixWithBoneMatrix(mul_current_matrix_with_bone_matrix_data) => {
                buffer[0] = self.command_code();
                mul_current_matrix_with_bone_matrix_data.write_bytes(&mut buffer[1..])?;
            },
            RenderCommand::Unknown0x07(unknown0x07_data) => {
                buffer[0] = self.command_code();
                unknown0x07_data.write_bytes(&mut buffer[1..])?;
            },
            RenderCommand::Unknown0x08(unknown0x08_data) => {
                buffer[0] = self.command_code();
                unknown0x08_data.write_bytes(&mut buffer[1..])?;
            },
            RenderCommand::CalculateSkinningEquation(calculate_skinning_equation_data) => {
                buffer[0] = self.command_code();
                calculate_skinning_equation_data.write_bytes(&mut buffer[1..])?;
            },
            RenderCommand::Scale(scale_data) => {
                buffer[0] = self.command_code();
                scale_data.write_bytes(&mut buffer[1..])?;
            },
            RenderCommand::Unknown0x0C(unknown0x0_cdata) => {
                buffer[0] = self.command_code();
                unknown0x0_cdata.write_bytes(&mut buffer[1..])?;
            },
            RenderCommand::Unknown0x0D(unknown0x0_ddata) => {
                buffer[0] = self.command_code();
                unknown0x0_ddata.write_bytes(&mut buffer[1..])?;
            },
        }

        Ok(())
    }

    pub fn command_code(&self) -> u8 {
        match self {
            RenderCommand::Nop(data) => 0x00 | data.subtype,
            RenderCommand::End => 0x01,
            RenderCommand::Unknown0x02(_) => 0x02,
            RenderCommand::LoadMatrixFromStack(_) => 0x03,
            RenderCommand::BindMaterial(data) => 0x04 | data.subtype,
            RenderCommand::DrawMesh(_) => 0x05,
            RenderCommand::MulCurrentMatrixWithBoneMatrix(data) => 0x06 | data.subtype,
            RenderCommand::Unknown0x07(data) => 0x07 | data.subtype,
            RenderCommand::Unknown0x08(_) => 0x08,
            RenderCommand::CalculateSkinningEquation(_) => 0x09,
            RenderCommand::Scale(data) => 0x0B | data.subtype,
            RenderCommand::Unknown0x0C(_) => 0x0C,
            RenderCommand::Unknown0x0D(_) => 0x0D
        }
    }

    pub fn size(&self) -> usize {
        match self {
            RenderCommand::Nop(_) => 1,
            RenderCommand::End => 1,
            RenderCommand::Unknown0x02(_) => 3,
            RenderCommand::LoadMatrixFromStack(_) => 2,
            RenderCommand::BindMaterial(_) => 2,
            RenderCommand::DrawMesh(_) => 2,
            RenderCommand::MulCurrentMatrixWithBoneMatrix(data) => 1 + data.len(),
            RenderCommand::Unknown0x07(_) => 2,
            RenderCommand::Unknown0x08(_) => 2,
            RenderCommand::CalculateSkinningEquation(data) => 1 + data.len(),
            RenderCommand::Scale(_) => 1,
            RenderCommand::Unknown0x0C(_) => 3,
            RenderCommand::Unknown0x0D(_) => 3
        }
    }
}


#[derive(Debug, Clone)]
pub struct NopData {
    pub subtype: u8
}

impl NopData {
    pub fn from_bytes(op_code: u8) -> Result<NopData, AppError> {
        let subtype = op_code & COMMAND_SUBTYPE_MASK;

        if subtype != 0x00 && subtype != 0x40 && subtype != 0x80 {
            return Err(AppError::new(&format!("Invalid Nop subtype: 0x{:2X}", subtype)));
        }

        Ok(NopData {
            subtype
        })
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x02Data {
    pub unknown_0: u8,
    pub unknown_1: u8
}

impl Unknown0x02Data {
    pub fn from_bytes(data: &[u8]) -> Result<Unknown0x02Data, AppError> {
        if data.len() < 2 {
            return Err(AppError::new("Unknown0x02Data needs at least 2 bytes"));
        }

        let unknown_0 = data[0];
        let unknown_1 = data[1];

        Ok(Unknown0x02Data {
            unknown_0,
            unknown_1
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 2 {
            return Err(AppError::new("Unknown0x02Data needs at least 2 bytes to write"));
        }

        buffer[0] = self.unknown_0;
        buffer[1] = self.unknown_1;

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct LoadMatrixFromStackData {
    pub stack_index: u8
}

impl LoadMatrixFromStackData {
    pub fn from_bytes(data: &[u8]) -> Result<LoadMatrixFromStackData, AppError> {
        if data.len() < 1 {
            return Err(AppError::new("LoadMatrixFromStackParams needs at least 1 byte"));
        }

        let stack_index = data[0];

        Ok(LoadMatrixFromStackData {
            stack_index
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 1 {
            return Err(AppError::new("LoadMatrixFromStackData needs at least 1 byte to write"));
        }

        buffer[0] = self.stack_index;

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct BindMaterialData {
    pub subtype: u8,
    pub material_index: u8
}

impl BindMaterialData {
    pub fn from_bytes(op_code: u8, data: &[u8]) -> Result<BindMaterialData, AppError> {
        if data.len() < 1 {
            return Err(AppError::new("BindMaterialData needs at least 1 byte"));
        }

        let subtype = op_code & COMMAND_SUBTYPE_MASK;

        if subtype != 0x00 && subtype != 0x20 && subtype != 0x40 {
            return Err(AppError::new(&format!("Invalid BindMaterial subtype: 0x{:2X}", subtype)));
        }

        let material_index = data[0];

        Ok(BindMaterialData {
            subtype,
            material_index
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 1 {
            return Err(AppError::new("BindMaterialData needs at least 1 byte to write"));
        }

        buffer[0] = self.material_index;

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct DrawMeshData {
    pub mesh_index: u8
}

impl DrawMeshData {
    pub fn from_bytes(data: &[u8]) -> Result<DrawMeshData, AppError> {
        if data.len() < 1 {
            return Err(AppError::new("DrawMeshData needs at least 1 byte"));
        }

        let mesh_index = data[0];

        Ok(DrawMeshData {
            mesh_index
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 1 {
            return Err(AppError::new("DrawMeshData needs at least 1 byte to write"));
        }

        buffer[0] = self.mesh_index;

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct MulCurrentMatrixWithBoneMatrixData {
    pub subtype: u8,

    pub bone_index: u8, // Bone matrix index
    pub parent_index: u8,
    pub unknown: u8,
    pub param_3: Option<u8>, // Used for matrix read/write id aplicable
    pub param_4: Option<u8> // Used for matrix read/write id aplicable
}

impl MulCurrentMatrixWithBoneMatrixData {
    pub fn from_bytes(op_code: u8, data: &[u8]) -> Result<MulCurrentMatrixWithBoneMatrixData, AppError> {
        let subtype = op_code & COMMAND_SUBTYPE_MASK;

        let data = match subtype {
            0x00 => {
                if data.len() < 3 {
                    return Err(AppError::new("MulCurrentMatrixWithBoneMatrixData (subtype 0x00) needs at least 3 bytes"));
                }

                let bone_index = data[0];
                let parent_index = data[1];
                let unknown = data[2];
                let matrix_update_index = None;
                let unknown_0x60 = None;

                MulCurrentMatrixWithBoneMatrixData {
                    subtype,
                    bone_index,
                    parent_index,
                    unknown,
                    param_3: matrix_update_index,
                    param_4: unknown_0x60
                }
            },
            0x20 | 0x40 => {
                if data.len() < 4 {
                    return Err(AppError::new(&format!("MulCurrentMatrixWithBoneMatrixData (subtype 0x{:02X}) needs at least 4 bytes", subtype)));
                }

                let bone_index = data[0];
                let parent_index = data[1];
                let unknown = data[2];
                let matrix_update_index = Some(data[3]);
                let unknown_0x60 = None;

                MulCurrentMatrixWithBoneMatrixData {
                    subtype,
                    bone_index,
                    parent_index,
                    unknown,
                    param_3: matrix_update_index,
                    param_4: unknown_0x60
                }
            },
            0x60 => {
                if data.len() < 5 {
                    return Err(AppError::new("MulCurrentMatrixWithBoneMatrixData (subtype 0x60) needs at least 5 bytes"));
                }

                let bone_index = data[0];
                let parent_index = data[1];
                let unknown = data[2];
                let matrix_update_index = Some(data[3]);
                let unknown_0x60 = Some(data[4]);

                MulCurrentMatrixWithBoneMatrixData {
                    subtype,
                    bone_index,
                    parent_index,
                    unknown,
                    param_3: matrix_update_index,
                    param_4: unknown_0x60
                }
            },
            _ => {
                return Err(AppError::new(&format!("Invalid MulCurrentMatrixWithBoneMatrix subtype: 0x{:2X}", subtype)));
            }
        };

        Ok(data)
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < self.len() {
            return Err(AppError::new(&format!("MulCurrentMatrixWithBoneMatrixData needs at least {} bytes to write", self.len())));
        }

        buffer[0] = self.bone_index;
        buffer[1] = self.parent_index;
        buffer[2] = self.unknown;

        // Don't need to manage the offset dynamically, as unknown0x60 will never be set if matrix_update_index isn't
        if let Some(matrix_update_index) = self.param_3 {
            buffer[3] = matrix_update_index;
        }

        if let Some(unknown_0x60) = self.param_4 {
            buffer[4] = unknown_0x60;
        }

        Ok(())
    }

    pub fn len(&self) -> usize {
        match self.subtype {
            0x00 => 3,
            0x20 | 0x40 => 4,
            0x60 => 5,
            _ => unreachable!()
        }
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x07Data {
    pub subtype: u8,

    pub unknown: u8,
    // There might be one more parameter for 0x47?
}

impl Unknown0x07Data {
    pub fn from_bytes(op_code: u8, data: &[u8]) -> Result<Unknown0x07Data, AppError> {
        if data.len() < 1 {
            return Err(AppError::new("Unknown0x07Data needs at least 1 byte"));
        }

        let subtype = op_code & COMMAND_SUBTYPE_MASK;
        let unknown = data[0];

        Ok(Unknown0x07Data {
            subtype,
            unknown
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 1 {
            return Err(AppError::new("Unknown0x07Data needs at least 1 byte to write"));
        }

        buffer[0] = self.unknown;

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x08Data {
    pub unknown: u8
}

impl Unknown0x08Data {
    pub fn from_bytes(data: &[u8]) -> Result<Unknown0x08Data, AppError> {
        if data.len() < 1 {
            return Err(AppError::new("Unknown0x08Data needs at least 1 byte"));
        }

        let unknown = data[0];

        Ok(Unknown0x08Data {
            unknown
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 1 {
            return Err(AppError::new("Unknown0x08Data needs at least 1 byte to write"));
        }

        buffer[0] = self.unknown;

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct CalculateSkinningEquationData {
    pub store_index: u8,
    pub num_terms: u8,
    pub terms: Vec<SkinningEquationTerm>
}

#[derive(Debug, Clone)]
pub struct SkinningEquationTerm {
    pub matrix_index: u8, // Matrix stack index for local-to-world (model matrix)
    pub inv_bind_index: u8, // Index in the InvBindMatrix for bind matrix
    pub weight: u8
}

impl CalculateSkinningEquationData {
    pub fn from_bytes(data: &[u8]) -> Result<CalculateSkinningEquationData, AppError> {
        if data.len() < 2 {
            return Err(AppError::new("CalculateSkinningEquationData needs at least 2 bytes"));
        }

        let store_index = data[0];
        let num_terms = data[1];

        if data.len() < 2 + (num_terms as usize * 3) {
            return Err(AppError::new("CalculateSkinningEquationData needs at least 2 + num_terms * 3 bytes"));
        }

        let mut terms = Vec::with_capacity(num_terms as usize);
        for i in 0..num_terms {
            let offset = 2 + (i as usize * 3);
            let matrix_index = data[offset];
            let inv_bind_index = data[offset + 1];
            let weight = data[offset + 2];

            terms.push(SkinningEquationTerm {
                matrix_index,
                inv_bind_index,
                weight
            });
        }

        Ok(CalculateSkinningEquationData {
            store_index,
            num_terms,
            terms
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < self.len() {
            return Err(AppError::new(&format!("CalculateSkinningEquationData needs at least {} bytes to write", self.len())));
        }

        buffer[0] = self.store_index;
        buffer[1] = self.num_terms;

        for (i, term) in self.terms.iter().enumerate() {
            let offset = 2 + (i * 3);
            buffer[offset] = term.matrix_index;
            buffer[offset + 1] = term.inv_bind_index;
            buffer[offset + 2] = term.weight;
        }

        Ok(())
    }

    pub fn len(&self) -> usize {
        let num_terms = self.num_terms as usize;
        let terms_size = num_terms * 3;
        terms_size + 2
    }
}


#[derive(Debug, Clone)]
pub struct ScaleData {
    pub subtype: u8
}

impl ScaleData {
    pub fn from_bytes(op_code: u8) -> Result<ScaleData, AppError> {
        let subtype = op_code & COMMAND_SUBTYPE_MASK;

        if subtype != 0x00 && subtype != 0x20 {
            return Err(AppError::new(&format!("Invalid Scale subtype: 0x{:2X}", subtype)));
        }

        Ok(ScaleData {
            subtype
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 1 {
            return Err(AppError::new("ScaleData needs at least 1 byte to write"));
        }

        buffer[0] = self.subtype;

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x0CData {
    pub unknown_0: u8,
    pub unknown_1: u8
}

impl Unknown0x0CData {
    pub fn from_bytes(data: &[u8]) -> Result<Unknown0x0CData, AppError> {
        if data.len() < 2 {
            return Err(AppError::new("Unknown0x0CData needs at least 2 bytes"));
        }

        let unknown_0 = data[0];
        let unknown_1 = data[1];

        Ok(Unknown0x0CData {
            unknown_0,
            unknown_1
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 2 {
            return Err(AppError::new("Unknown0x0CData needs at least 2 bytes to write"));
        }

        buffer[0] = self.unknown_0;
        buffer[1] = self.unknown_1;

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Unknown0x0DData {
    pub unknown_0: u8,
    pub unknown_1: u8
}

impl Unknown0x0DData {
    pub fn from_bytes(data: &[u8]) -> Result<Unknown0x0DData, AppError> {
        if data.len() < 2 {
            return Err(AppError::new("Unknown0x0DData needs at least 2 bytes"));
        }

        let unknown_0 = data[0];
        let unknown_1 = data[1];

        Ok(Unknown0x0DData {
            unknown_0,
            unknown_1
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < 2 {
            return Err(AppError::new("Unknown0x0DData needs at least 2 bytes to write"));
        }

        buffer[0] = self.unknown_0;
        buffer[1] = self.unknown_1;

        Ok(())
    }
}
