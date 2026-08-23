use gpu_command_list::GpuCommandList;

use crate::{data_structures::name_list::NameList, debug_info::DebugInfo, error::AppError};

pub mod gpu_command_list;

#[derive(Debug, Clone)]
pub struct MeshList {
    meshes: NameList<u32>,
    mesh_data: Vec<Mesh>,

    // Debug info
    _debug_info: DebugInfo
}

impl MeshList {
    pub fn from_bytes(bytes: &[u8], debug_info: DebugInfo) -> Result<MeshList, AppError> {
        let meshes = NameList::from_bytes(bytes)?;

        let mut mesh_data = Vec::with_capacity(meshes.len());
        for &offset in meshes.data_iter() {
            let offset = offset as usize;
            let mesh = Mesh::from_bytes(&bytes[offset..])?;
            mesh_data.push(mesh);
        }

        Ok(MeshList {
            meshes,
            mesh_data,
            _debug_info: debug_info
        })
    }

    pub fn write_bytes(&self, buffer: &mut[u8]) -> Result<(), AppError> {
        self.meshes.write_bytes(buffer)?;

        for (i, &offset) in self.meshes.data_iter().enumerate() {
            let offset = offset as usize;
            let mesh = &self.mesh_data[i];
            mesh.write_bytes(&mut buffer[offset..])?;
        }

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.meshes.size() + self.mesh_data.len() * Mesh::SIZE + self.mesh_data.iter().map(|m| m.render_cmds_list.size()).sum::<usize>()
    }

    pub fn len(&self) -> usize {
        self.mesh_data.len()
    }

    pub fn names_iter(&self) -> impl Iterator<Item = &crate::data_structures::name::Name> {
        self.meshes.names_iter()
    }

    pub fn get_name(&self, index: usize) -> Option<&crate::data_structures::name::Name> {
        self.meshes.get_name(index)
    }

    pub fn rebase(&mut self) {
        self.meshes.rebase();

        let base_offset = self.meshes.size();
        let num_meshes = self.mesh_data.len();
        let mut current_cmds_abs_offset = base_offset + num_meshes * Mesh::SIZE;

        for (i, (offset, mesh)) in self.meshes.data_iter_mut().zip(self.mesh_data.iter_mut()).enumerate() {
            let mesh_header_offset = base_offset + i * Mesh::SIZE;
            *offset = mesh_header_offset as u32;

            mesh.cmds_offset = (current_cmds_abs_offset - mesh_header_offset) as u32;
            mesh.cmds_len = mesh.render_cmds_list.size() as u32;

            current_cmds_abs_offset += mesh.cmds_len as usize;
        }
    }

    pub fn get_mesh(&self, index: usize) -> Option<&Mesh> {
        self.mesh_data.get(index)
    }

    pub fn get_mesh_mut(&mut self, index: usize) -> Option<&mut Mesh> {
        self.mesh_data.get_mut(index)
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    dummy: u16,
    size: u16, // Always 0x10 (size of this struct?)
    unknown: u32,
    cmds_offset: u32,
    cmds_len: u32,

    render_cmds_list: GpuCommandList
}

impl Mesh {
    const SIZE: usize = 16; // Size of the Mesh struct (without render_cmds_list)

    pub fn from_bytes(bytes: &[u8]) -> Result<Mesh, AppError> {
        if bytes.len() < Mesh::SIZE {
            return Err(AppError::new("Mesh needs at least 16 bytes"));
        }

        let dummy = u16::from_le_bytes([bytes[0], bytes[1]]);
        let size = u16::from_le_bytes([bytes[2], bytes[3]]);
        let unknown = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let cmds_offset = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let cmds_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);

        if bytes.len() < (cmds_offset + cmds_len) as usize {
            return Err(AppError::new(&format!(
                "Mesh needs at least {} bytes",
                cmds_offset + cmds_len
            )));
        }

        let render_cmds = &bytes[cmds_offset as usize..(cmds_offset + cmds_len) as usize];

        let render_cmds_list = GpuCommandList::from_bytes(render_cmds)?;

        Ok(Mesh {
            dummy,
            size,
            unknown,
            cmds_offset,
            cmds_len,
            render_cmds_list
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < Mesh::SIZE {
            return Err(AppError::new("Mesh needs at least 16 bytes"));
        }

        buffer[0..2].copy_from_slice(&self.dummy.to_le_bytes());
        buffer[2..4].copy_from_slice(&self.size.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.unknown.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.cmds_offset.to_le_bytes());
        buffer[12..16].copy_from_slice(&self.cmds_len.to_le_bytes());

        let cmds_len = self.render_cmds_list.size() as u32;

        if buffer.len() < (self.cmds_offset + cmds_len) as usize {
            return Err(AppError::new(&format!(
                "Mesh needs at least {} bytes",
                self.cmds_offset + cmds_len
            )));
        }
        
        self.render_cmds_list.write_bytes(&mut buffer[self.cmds_offset as usize..(self.cmds_offset + cmds_len) as usize])?;

        Ok(())
    }

    pub fn rebase(&mut self) {
        self.cmds_len = self.render_cmds_list.size() as u32;
    }

    pub fn size(&self) -> usize {
        Mesh::SIZE + self.render_cmds_list.size()
    }

    pub fn cmds_offset(&self) -> u32 {
        self.cmds_offset
    }

    pub fn cmds_len(&self) -> u32 {
        self.cmds_len
    }

    pub fn set_render_cmds_list(&mut self, cmds: GpuCommandList) {
        self.render_cmds_list = cmds;
        self.rebase();
    }

    pub fn get_render_cmds_list(&self) -> &GpuCommandList {
        &self.render_cmds_list
    }

    pub fn get_render_cmds_list_mut(&mut self) -> &mut GpuCommandList {
        &mut self.render_cmds_list
    }
}
