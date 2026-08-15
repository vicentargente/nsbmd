use crate::{error::AppError, subfiles::mdl::model::{bone_list::BoneList, inv_bind_matrices::{self, InvBindMatrices}, render_command_list::{RenderCommand, RenderCommandList}}, util::math::matrix::Matrix};

// State machine to execute model render commands
pub struct ModelRenderCmdExecutor<'a> {
    render_cmds: &'a RenderCommandList,
    bone_list: &'a BoneList,
    inv_bind_matrices: &'a InvBindMatrices,
    upscale: f32,
    downscale: f32,

    // Internal state for the executor
    matrix_stack: Vec<Matrix>, // Visit https://problemkaputt.de/gbatek.htm#ds3dvideo (DS 3D Matrix Stack) for more info
    current_matrix: Matrix,

    // Additional useful data
    loaded_bones_in_matrix: Vec<Option<String>>
}

impl ModelRenderCmdExecutor<'_> {
    pub fn new<'a>(
        render_cmds: &'a RenderCommandList,
        bone_list: &'a BoneList,
        inv_bind_matrices: &'a InvBindMatrices,
        upscale: f32,
        downscale: f32
    ) -> ModelRenderCmdExecutor<'a> {
        let matrix_stack = vec![Matrix::identity(4); 31]; // 0..30 (31 entries)
        let current_matrix = Matrix::identity(4); // Initial current matrix

        let loaded_bones_in_matrix = vec![None; 31]; // 0..30 (31 entries)

        ModelRenderCmdExecutor {
            render_cmds,
            bone_list,
            inv_bind_matrices,
            upscale,
            downscale,
            matrix_stack,
            current_matrix,
            loaded_bones_in_matrix
        }
    }

    pub fn execute(&mut self) -> Result<(), AppError> {
        for cmd in self.render_cmds.iter() {
            self.execute_command(cmd)?;
        }

        Ok(())
    }

    pub fn execute_until_next_mesh_draw(&mut self) -> Result<(), AppError> {
        for cmd in self.render_cmds.iter() {
            if let RenderCommand::DrawMesh(_) = cmd {
                return Ok(()); // Stop execution when we reach a DrawMesh command
            }

            self.execute_command(cmd)?;
        }

        Err(AppError::new("No DrawMesh command found in the render command list."))
    }

    pub fn matrix_stack(&self) -> &Vec<Matrix> {
        &self.matrix_stack
    }

    pub fn loaded_bones_in_matrix(&self) -> &Vec<Option<String>> {
        &self.loaded_bones_in_matrix
    }

    fn execute_command(&mut self, cmd: &RenderCommand) -> Result<(), AppError> {
        match cmd {
            RenderCommand::Nop(_nop_data) => {},
            RenderCommand::End => {},
            RenderCommand::Unknown0x02(_unknown0x02_data) => { /* Unknown */},
            RenderCommand::LoadMatrixFromStack(load_matrix_from_stack_data) => {
                let index = load_matrix_from_stack_data.stack_index as usize;
                if index >= self.matrix_stack.len() {
                    return Err(AppError::new(&format!("LoadMatrixFromStack::Invalid stack index. Expected 0-{}, got {}", self.matrix_stack.len() - 1, index)));
                }

                self.current_matrix = self.matrix_stack[index].clone();
            },
            RenderCommand::BindMaterial(_bind_material_data) => {
                // TODO: Implement material binding logic
            },
            RenderCommand::DrawMesh(_draw_mesh_data) => {
                // Nothing to do at the moment
            },
            RenderCommand::MulCurrentMatrixWithBoneMatrix(data) => {
                let bone_index = data.bone_index as usize;
                if bone_index >= self.bone_list.len() {
                    return Err(AppError::new(&format!("MulCurrentMatrixWithBoneMatrix::Invalid bone index. Expected 0-{}, got {}", self.bone_list.len() - 1, bone_index)));
                }

                let (store_pos, load_pos) = match data.subtype {
                    0x00 => (None, None),
                    0x20 => (Some(data.param_3.unwrap()), None),
                    0x40 => (None, Some(data.param_3.unwrap())),
                    0x60 => (Some(data.param_3.unwrap()), Some(data.param_4.unwrap())),
                    _ => return Err(AppError::new(&format!("MulCurrentMatrixWithBoneMatrix::Unknown subtype: {}", data.subtype))),
                };

                if let Some(stack_index) = load_pos {
                    self.current_matrix = self.matrix_stack[stack_index as usize].clone();
                }

                let bone_matrix = self.bone_list.get_bone_matrix(bone_index)
                    .ok_or_else(|| AppError::new(&format!("MulCurrentMatrixWithBoneMatrix::Could not find bone matrix at index {}", bone_index)))?
                    .to_matrix();
                self.current_matrix = self.current_matrix.clone() * bone_matrix;

                if let Some(stack_index) = store_pos {
                    let matrix_update_index = stack_index as usize;
                    self.matrix_stack[matrix_update_index] = self.current_matrix.clone();
                    self.loaded_bones_in_matrix[matrix_update_index] = Some(self.bone_list.get_name(bone_index).unwrap().to_not_null_string().unwrap());
                }
            },
            RenderCommand::Unknown0x07(_unknown0x07_data) => { /* Unknown */ },
            RenderCommand::Unknown0x08(_unknown0x08_data) => { /* Unknown */ },
            RenderCommand::CalculateSkinningEquation(data) => {
                let store_index = data.store_index as usize;
                if store_index >= self.matrix_stack.len() {
                    return Err(AppError::new(&format!(
                        "CalculateSkinningEquation::Invalid store index {}", store_index
                    )));
                }

                // Accumulator matrix for the blended transform
                let mut blended_matrix = Matrix::zeros(4, 4);

                for term in data.terms.iter() {
                    let matrix_idx = term.matrix_index as usize;
                    if matrix_idx >= self.matrix_stack.len() {
                        return Err(AppError::new(&format!(
                            "CalculateSkinningEquation::Invalid matrix index {}", matrix_idx
                        )));
                    }

                    let inv_bind = self.inv_bind_matrices
                        .get(term.inv_bind_index as usize)
                        .ok_or_else(|| AppError::new(&format!(
                            "CalculateSkinningEquation::InvBind index {} not found", term.inv_bind_index
                        )))?
                        .to_matrix();

                    // World transform for this bone * Inverse Bind Matrix
                    let bone_world = &self.matrix_stack[matrix_idx];
                    let term_matrix = bone_world.clone() * inv_bind;
                    let weight = term.weight_f32();

                    // Add weighted term matrix
                    for r in 0..4 {
                        for c in 0..4 {
                            let curr = blended_matrix.get(r, c)?;
                            let term_val = term_matrix.get(r, c)?;
                            blended_matrix.set(r, c, curr + term_val * weight)?;
                        }
                    }
                }

                // Preserve affine homogeneous coordinates
                blended_matrix.set(3, 3, 1.0)?;

                self.matrix_stack[store_index] = blended_matrix;
                self.loaded_bones_in_matrix[store_index] = Some(format!("SkinBlend_{}", store_index));
            },
            RenderCommand::Scale(scale_data) => {
                let scale_factor = match scale_data.subtype {
                    0x00 => self.upscale,
                    0x20 => self.downscale,
                    _ => return Err(AppError::new(&format!(
                        "Scale::Unknown subtype: 0x{:02X}", scale_data.subtype
                    ))),
                };

                let scale_matrix = Matrix::new(4, 4, vec![
                    scale_factor, 0.0,          0.0,          0.0,
                    0.0,          scale_factor, 0.0,          0.0,
                    0.0,          0.0,          scale_factor, 0.0,
                    0.0,          0.0,          0.0,          1.0,
                ])?;

                self.current_matrix = self.current_matrix.clone() * scale_matrix;
            },
            RenderCommand::Unknown0x0C(_unknown0x0c_data) => { /* Unknown */ },
            RenderCommand::Unknown0x0D(_unknown0x0d_data) => { /* Unknown */ },
        }

        Ok(())
    }
}
