use crate::{error::AppError, subfiles::mdl::model::mesh_list::gpu_command_list::{GpuCommand, GpuCommandList}, tools::models::vertex::Position};

#[derive(Debug, Clone)]
pub struct MeshRenderCmdVertexPosExtractor<'a> {
    render_cmds: &'a GpuCommandList,
    current_vertex: Position,
    vertices: Vec<Position>,
    
    is_in_vtx_group: bool
}

impl MeshRenderCmdVertexPosExtractor<'_> {
    pub fn new<'a>(render_cmds: &'a GpuCommandList) -> MeshRenderCmdVertexPosExtractor<'a> {
        MeshRenderCmdVertexPosExtractor {
            render_cmds,
            current_vertex: Position { x: 0.0, y: 0.0, z: 0.0 },
            vertices: Vec::new(),
            is_in_vtx_group: false
        }
    }

    pub fn execute(&mut self) -> Result<(), AppError> {
        for cmd in self.render_cmds.iter() {
            self.execute_command(cmd)?;
        }

        Ok(())
    }

    pub fn vertices(&self) -> &Vec<Position> {
        &self.vertices
    }

    fn execute_command(&mut self, cmd: &GpuCommand) -> Result<(), AppError> {
        match cmd {
            GpuCommand::Nop => {},
            GpuCommand::MtxRestore(_mtx_restore_params) => {},
            GpuCommand::MtxScale(_mtx_scale_params) => {},
            GpuCommand::Unknown0x1C(_unknown0x1_cparams) => {},
            GpuCommand::Color(_color_params) => {},
            GpuCommand::Normal(_normal_params) => {},
            GpuCommand::TexCoord(_tex_coord_params) => {},
            GpuCommand::Vtx16(vtx16_params) => {
                let vertex_pos = Position {
                    x: vtx16_params.x.to_f32(),
                    y: vtx16_params.y.to_f32(),
                    z: vtx16_params.z.to_f32()
                };

                self.current_vertex = vertex_pos.clone();
                self.vertices.push(vertex_pos);
            },
            GpuCommand::Vtx10(vtx10_params) => {
                let vertex_pos = Position {
                    x: vtx10_params.x.to_f32(),
                    y: vtx10_params.y.to_f32(),
                    z: vtx10_params.z.to_f32()
                };

                self.current_vertex = vertex_pos.clone();
                self.vertices.push(vertex_pos);
            },
            GpuCommand::VtxXY(vtx_xyparams) => {
                let vertex_pos = Position {
                    x: vtx_xyparams.x.to_f32(),
                    y: vtx_xyparams.y.to_f32(),
                    z: self.current_vertex.z
                };

                self.current_vertex = vertex_pos.clone();
                self.vertices.push(vertex_pos);
            },
            GpuCommand::VtxXZ(vtx_xzparams) => {
                let vertex_pos = Position {
                    x: vtx_xzparams.x.to_f32(),
                    y: self.current_vertex.y,
                    z: vtx_xzparams.z.to_f32()
                };

                self.current_vertex = vertex_pos.clone();
                self.vertices.push(vertex_pos);
            },
            GpuCommand::VtxYZ(vtx_yzparams) => {
                let vertex_pos = Position {
                    x: self.current_vertex.x,
                    y: vtx_yzparams.y.to_f32(),
                    z: vtx_yzparams.z.to_f32()
                };

                self.current_vertex = vertex_pos.clone();
                self.vertices.push(vertex_pos);
            },
            GpuCommand::VtxDiff(vtx_diff_params) => {
                let vertex_pos = Position {
                    x: self.current_vertex.x + vtx_diff_params.x.to_f32(),
                    y: self.current_vertex.y + vtx_diff_params.y.to_f32(),
                    z: self.current_vertex.z + vtx_diff_params.z.to_f32()
                };

                self.current_vertex = vertex_pos.clone();
                self.vertices.push(vertex_pos);
            },
            GpuCommand::BeginVtxs(_begin_vtxs_params) => {
                if self.is_in_vtx_group {
                    return Err(AppError::new("BeginVtxs called while already in a vertex group."));
                }

                self.is_in_vtx_group = true;
                self.current_vertex.x = 0.0;
                self.current_vertex.y = 0.0;
                self.current_vertex.z = 0.0;
            },
            GpuCommand::EndVtxs => {
                if !self.is_in_vtx_group {
                    return Err(AppError::new("EndVtxs called while not in a vertex group."));
                }

                self.is_in_vtx_group = false;
            },
            _ => {}
        }

        Ok(())
    }
}