use bone_list::BoneList;
use bounding_box::BoundingBox;
use inv_bind_matrices::InvBindMatrices;
use material_list::MaterialList;
use mesh_list::MeshList;
use render_command_list::RenderCommandList;

use crate::{debug_info::DebugInfo, error::AppError, executors::model_render_cmd_executor::ModelRenderCmdExecutor, util::number::{alignment::get_4_byte_alignment, fixed_point::fixed_1_19_12::Fixed1_19_12}};

pub mod bounding_box;
pub mod bone_list;
pub mod render_command_list;
pub mod material_list;
pub mod mesh_list;
pub mod inv_bind_matrices;

#[derive(Debug, Clone)]
pub struct Model {
    size: u32,
    render_cmds_offset: u32,
    materials_offset: u32,
    meshes_offset: u32,
    inv_binds_offset: u32,
    unknown: [u8; 3],
    num_bone_matrices: u8,
    num_materials: u8,
    num_meshes: u8,
    unknown_2: [u8; 2],
    upscale: Fixed1_19_12,
    downscale: Fixed1_19_12,
    num_verts: u16,
    num_polys: u16,
    num_tris: u16,
    num_quads: u16,
    bounding_box: BoundingBox,
    unknown_3: [u8; 8],
    bone_list: BoneList,

    // Actual data
    render_commands: RenderCommandList,
    materials: MaterialList,
    meshes: MeshList,
    inv_binds_matrices: InvBindMatrices,

    // Debug info
    _debug_info: DebugInfo
}

impl Model {
    const _BASE_SIZE: usize = 52; // Size of the model header, without data nor bounding box

    pub fn from_bytes(bytes: &[u8], debug_info: DebugInfo) -> Result<Model, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("Model needs at least 4 bytes to start reading"))
        }

        let size = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        if bytes.len() < size as usize {
            return Err(AppError::new(&format!("Model needs at least {} bytes", size)));
        }

        let render_cmds_offset = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let materials_offset = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let meshes_offset = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let inv_binds_offset = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);

        let unknown = [bytes[20], bytes[21], bytes[22]];

        let num_bone_matrices = bytes[23];
        let num_materials = bytes[24];
        let num_meshes = bytes[25];
        let unknown_2 = [bytes[26], bytes[27]];

        let upscale = i32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]);
        let downscale = i32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]);

        let num_verts = u16::from_le_bytes([bytes[36], bytes[37]]);
        let num_polys = u16::from_le_bytes([bytes[38], bytes[39]]);
        let num_tris = u16::from_le_bytes([bytes[40], bytes[41]]);
        let num_quads = u16::from_le_bytes([bytes[42], bytes[43]]);

        let bounding_box = BoundingBox::from_bytes(&bytes[44..])?;

        let unknown_3 = [
            bytes[56], bytes[57], bytes[58], bytes[59],
            bytes[60], bytes[61], bytes[62], bytes[63],
        ];

        let bone_list = BoneList::from_bytes(&bytes[64..], DebugInfo { offset: debug_info.offset + 64 })?;

        let render_commands = RenderCommandList::from_bytes(&bytes[(render_cmds_offset as usize)..], DebugInfo { offset: debug_info.offset + render_cmds_offset })?;
        let materials = MaterialList::from_bytes(&bytes[(materials_offset as usize)..], DebugInfo { offset: debug_info.offset + materials_offset })?;
        let meshes = MeshList::from_bytes(&bytes[(meshes_offset as usize)..], DebugInfo { offset: debug_info.offset + meshes_offset })?;
        let inv_binds_matrices = InvBindMatrices::from_bytes(&bytes[(inv_binds_offset as usize)..], DebugInfo { offset: debug_info.offset + inv_binds_offset })?;

        Ok(Model {
            size,
            render_cmds_offset,
            materials_offset,
            meshes_offset,
            inv_binds_offset,
            unknown,
            num_bone_matrices,
            num_materials,
            num_meshes,
            unknown_2,
            upscale: Fixed1_19_12::from(upscale),
            downscale: Fixed1_19_12::from(downscale),
            num_verts,
            num_polys,
            num_tris,
            num_quads,
            bounding_box,
            unknown_3,
            bone_list,
            render_commands,
            materials,
            meshes,
            inv_binds_matrices,
            _debug_info: debug_info
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < self.size as usize {
            return Err(AppError::new(&format!("Model buffer needs at least {} bytes to write", self.size)));
        }

        buffer[0..4].copy_from_slice(&self.size.to_le_bytes());

        buffer[4..8].copy_from_slice(&self.render_cmds_offset.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.materials_offset.to_le_bytes());
        buffer[12..16].copy_from_slice(&self.meshes_offset.to_le_bytes());
        buffer[16..20].copy_from_slice(&self.inv_binds_offset.to_le_bytes());

        buffer[20..23].copy_from_slice(&self.unknown);
        buffer[23] = self.num_bone_matrices;
        buffer[24] = self.num_materials;
        buffer[25] = self.num_meshes;
        buffer[26..28].copy_from_slice(&self.unknown_2);

        buffer[28..32].copy_from_slice(&self.upscale.to_le_bytes());
        buffer[32..36].copy_from_slice(&self.downscale.to_le_bytes());

        buffer[36..38].copy_from_slice(&self.num_verts.to_le_bytes());
        buffer[38..40].copy_from_slice(&self.num_polys.to_le_bytes());
        buffer[40..42].copy_from_slice(&self.num_tris.to_le_bytes());
        buffer[42..44].copy_from_slice(&self.num_quads.to_le_bytes());

        self.bounding_box.write_bytes(&mut buffer[44..])?;
        buffer[56..64].copy_from_slice(&self.unknown_3);

        self.bone_list.write_bytes(&mut buffer[64..])?;

        self.render_commands.write_bytes(&mut buffer[self.render_cmds_offset as usize..])?;
        self.materials.write_bytes(&mut buffer[self.materials_offset as usize..])?;
        self.meshes.write_bytes(&mut buffer[self.meshes_offset as usize..])?;
        self.inv_binds_matrices.write_bytes(&mut buffer[self.inv_binds_offset as usize..])?;

        Ok(())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, AppError> {
        let mut buffer = vec![0u8; self.size as usize];

        self.write_bytes(&mut buffer)?;

        Ok(buffer)
    }

    pub fn size(&self) -> usize {
        // Self::BASE_SIZE + BoundingBox::SIZE + self.bone_list.size() + self.render_commands.size() + self.materials.size() + self.meshes.size() + self.inv_binds_matrices.size()
        self.inv_binds_offset as usize + self.inv_binds_matrices.size() as usize
    }

    pub fn rebase(&mut self) {
        self.bone_list.rebase();
        // No need to rebase render commands, every size is dynamically calculated and not stored
        self.materials.rebase();
        self.meshes.rebase();
        

        let render_command_list_offset = 64 + get_4_byte_alignment(self.bone_list.size());
        let material_list_offset = render_command_list_offset + get_4_byte_alignment(self.render_commands.size());
        let mesh_list_offset = material_list_offset + get_4_byte_alignment(self.materials.size());
        let inv_bind_matrices_offset = mesh_list_offset + get_4_byte_alignment(self.meshes.size());

        self.render_cmds_offset = render_command_list_offset as u32;
        self.materials_offset = material_list_offset as u32;
        self.meshes_offset = mesh_list_offset as u32;
        self.inv_binds_offset = inv_bind_matrices_offset as u32;


        let size = self.size();

        self.size = size as u32;
    }

    pub fn get_bone_list(&self) -> &BoneList {
        &self.bone_list
    }

    pub fn get_bone_list_mut(&mut self) -> &mut BoneList {
        &mut self.bone_list
    }

    pub fn get_bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }

    pub fn get_bounding_box_mut(&mut self) -> &mut BoundingBox {
        &mut self.bounding_box
    }

    pub fn get_inv_bind_matrices(&self) -> &InvBindMatrices {
        &self.inv_binds_matrices
    }

    pub fn get_inv_bind_matrices_mut(&mut self) -> &mut InvBindMatrices {
        &mut self.inv_binds_matrices
    }

    pub fn get_mesh_list(&self) -> &MeshList {
        &self.meshes
    }

    pub fn get_mesh_list_mut(&mut self) -> &mut MeshList {
        &mut self.meshes
    }

    pub fn get_render_cmds_list(&self) -> &RenderCommandList {
        &self.render_commands
    }

    pub fn get_render_cmds_list_mut(&mut self) -> &mut RenderCommandList {
        &mut self.render_commands
    }

    pub fn get_render_command_executor(&self) -> ModelRenderCmdExecutor {
        ModelRenderCmdExecutor::new(&self.render_commands, &self.bone_list)
    }
}
