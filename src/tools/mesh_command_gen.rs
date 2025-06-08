use std::collections::HashMap;

use crate::{error::AppError, subfiles::mdl::model::mesh_list::gpu_command_list::{BeginVtxsParams, GpuCommand, MtxRestoreParams, TexCoordParams, Vtx16Params}, util::number::fixed_point::{fixed_1_11_4::Fixed1_11_4, fixed_1_3_12::Fixed1_3_12}};

use super::models::{primitive::Primitive, vertex::Vertex};

#[derive(Debug, Clone)]
pub struct MeshCommandGenerator<'a> {
    primitives: &'a Vec<Primitive>,
    vertex_to_command_bone_mapping: HashMap<usize, usize>,
    texture_size: (f32, f32)
}

impl MeshCommandGenerator<'_> {
    pub fn new<'a>(
        primitives: &'a Vec<Primitive>,
        vertex_bones: &'a Vec<String>,
        command_bones: &'a Vec<Option<String>>,
        texture_size: (f32, f32)
    ) -> Result<MeshCommandGenerator<'a>, AppError> {
        let vertex_to_command_bone_mapping = Self::generate_vertex_to_command_bone_mapping(primitives, vertex_bones, command_bones)?;

        Ok(MeshCommandGenerator {
            primitives,
            vertex_to_command_bone_mapping,
            texture_size
        })
    }

    pub fn generate_commands(&self) -> Result<Vec<GpuCommand>, AppError> {
        let command_groups = self.generate_command_groups()?;
        let mut commands = Vec::new();

        // Generate commands for single-bonned triangles
        self.generate_single_bonned_triangle_commands(&command_groups.single_bonned_triangles, &mut commands)?;

        // Generate commands for multi-bonned triangles
        self.generate_multi_bonned_triangle_commands(&command_groups.multi_bonned_triangles, &mut commands)?;

        Ok(commands)
    }

    fn get_vertex_to_cmd_bone_mapped_index(&self, vertex_bone_index: usize) -> Result<u32, AppError> {
        match self.vertex_to_command_bone_mapping.get(&vertex_bone_index) {
            Some(id) => Ok(*id as u32),
            None => { return Err(AppError::new(&format!("Bone ID {} not found in command bone mapping.", vertex_bone_index))); },
        }
    }

    fn generate_vertex_to_command_bone_mapping(primitives: &Vec<Primitive>, vertex_bones: &Vec<String>, command_bones: &Vec<Option<String>>) -> Result<HashMap<usize, usize>, AppError> {
        let mut vertex_bone_is_used = vec![false; vertex_bones.len()];
        for primitive in primitives {
            for vertex in primitive.vertices().iter() {
                vertex_bone_is_used[vertex.bone_id as usize] = true;
            }
        }

        let mut vertex_to_command_bone_mapping = HashMap::new();
        for (vertex_bone_index, vertex_bone) in vertex_bones.iter().enumerate() {
            if !vertex_bone_is_used[vertex_bone_index] {
                continue;
            }

            if let Some(command_bone) = command_bones.iter().position(|cmd_bone| cmd_bone.as_ref() == Some(vertex_bone)) {
                vertex_to_command_bone_mapping.insert(vertex_bone_index, command_bone);
            }
            else {
                return Err(AppError::new(&format!("Every bone in model must exist in original nsbmd. Bone '{}' not found in command bones.", vertex_bone)));
            }
        }

        // println!("Vertex to command bone mapping: {:#?}", vertex_to_command_bone_mapping);

        Ok(vertex_to_command_bone_mapping)
    }

    fn generate_command_groups(&self) -> Result<CommandGroups, AppError> {
        let mut command_groups = CommandGroups::new();

        for primitive in self.primitives {
            match primitive {
                Primitive::Triangle { vertices, indices } => {
                    if indices.len() % 3 != 0 {
                        return Err(AppError::new("Indices length must be a multiple of 3 for triangles."));
                    }

                    for i in (0..indices.len()).step_by(3) {
                        let v1 = vertices[indices[i] as usize].clone();
                        let v2 = vertices[indices[i + 1] as usize].clone();
                        let v3 = vertices[indices[i + 2] as usize].clone();
                        let triangle = PolygonTriangle::new(v1, v2, v3);
                        command_groups.add_triangle(triangle);
                    }
                },
            }
        }

        Ok(command_groups)
    }

    fn generate_single_bonned_triangle_commands(&self, triangles: &HashMap<usize, Vec<PolygonTriangle>>, commands: &mut Vec<GpuCommand>) -> Result<(), AppError> {
        for (&bone_id, triangles) in triangles {
            if triangles.is_empty() {
                continue;
            }

            let cmd_bone_id = self.get_vertex_to_cmd_bone_mapped_index(bone_id)?;

            commands.push(GpuCommand::BeginVtxs(Box::new(BeginVtxsParams { primitive_type: BeginVtxsParams::TRIANGLE })));
            commands.push(GpuCommand::MtxRestore(Box::new(MtxRestoreParams { index: cmd_bone_id })));
            for triangle in triangles {
                let current_triangle_vertices = [&triangle.v1, &triangle.v2, &triangle.v3];

                for vertex in current_triangle_vertices {
                    let s = Fixed1_11_4::from_f32(vertex.tex_coord.u * self.texture_size.0);
                    let t = Fixed1_11_4::from_f32(vertex.tex_coord.v * self.texture_size.1);
                    commands.push(GpuCommand::TexCoord(Box::new(TexCoordParams { s, t })));
    
                    let x = Fixed1_3_12::from(vertex.position.x);
                    let y = Fixed1_3_12::from(vertex.position.y);
                    let z = Fixed1_3_12::from(vertex.position.z);
                    commands.push(GpuCommand::Vtx16(Box::new(Vtx16Params { x, y, z })));
                }
            }
            commands.push(GpuCommand::EndVtxs);
        }

        Ok(())
    }

    fn generate_multi_bonned_triangle_commands(&self, triangles: &Vec<PolygonTriangle>, commands: &mut Vec<GpuCommand>) -> Result<(), AppError> {
        if triangles.is_empty() {
            return Ok(());
        }

        let mut prev_bone_id = self.get_vertex_to_cmd_bone_mapped_index(triangles[0].v1.bone_id as usize)?;

        commands.push(GpuCommand::BeginVtxs(Box::new(BeginVtxsParams { primitive_type: BeginVtxsParams::TRIANGLE })));
        commands.push(GpuCommand::MtxRestore(Box::new(MtxRestoreParams { index: prev_bone_id })));
        for triangle in triangles {
            let current_triangle_vertices = [&triangle.v1, &triangle.v2, &triangle.v3];
            for vertex in current_triangle_vertices {
                let current_bone_id = self.get_vertex_to_cmd_bone_mapped_index(vertex.bone_id as usize)?;

                if current_bone_id != prev_bone_id {
                    commands.push(GpuCommand::MtxRestore(Box::new(MtxRestoreParams { index: current_bone_id })));
                    prev_bone_id = current_bone_id;
                }
    
                let s = Fixed1_11_4::from_f32(vertex.tex_coord.u * self.texture_size.0);
                let t = Fixed1_11_4::from_f32(vertex.tex_coord.v * self.texture_size.1);
                commands.push(GpuCommand::TexCoord(Box::new(TexCoordParams { s, t })));
    
                let x = Fixed1_3_12::from(vertex.position.x);
                let y = Fixed1_3_12::from(vertex.position.y);
                let z = Fixed1_3_12::from(vertex.position.z);
                commands.push(GpuCommand::Vtx16(Box::new(Vtx16Params { x, y, z })));
            }
        }
        commands.push(GpuCommand::EndVtxs);
        
        Ok(())
    }
}

struct PolygonTriangle {
    v1: Vertex,
    v2: Vertex,
    v3: Vertex
}

impl PolygonTriangle {
    pub fn new(v1: Vertex, v2: Vertex, v3: Vertex) -> Self {
        PolygonTriangle { v1, v2, v3 }
    }

    pub fn is_single_bonned(&self) -> bool {
        self.v1.bone_id == self.v2.bone_id && self.v1.bone_id == self.v3.bone_id
    }
}

struct CommandGroups {
    single_bonned_triangles: HashMap<usize, Vec<PolygonTriangle>>,
    multi_bonned_triangles: Vec<PolygonTriangle>,
}

impl CommandGroups {
    pub fn new() -> Self {
        CommandGroups {
            single_bonned_triangles: HashMap::new(),
            multi_bonned_triangles: Vec::new(),
        }
    }

    pub fn add_triangle(&mut self, triangle: PolygonTriangle) {
        if triangle.is_single_bonned() {
            let bone_id = triangle.v1.bone_id as usize;
            self.single_bonned_triangles
                .entry(bone_id)
                .or_default()
                .push(triangle);
        }
        else {
            self.multi_bonned_triangles.push(triangle);
        }
    }
}
