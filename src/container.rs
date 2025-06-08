use crate::{debug_info::DebugInfo, error::AppError, subfiles::{jnt::Jnt, mdl::Mdl, pat::Pat, srt::Srt, tex::Tex, Type}, util::number::alignment::get_4_byte_alignment};

#[derive(Debug, Clone)]
pub struct Container {
    header: Header,
    subfile_offsets: Vec<u32>,

    // Actual data
    files: Files

    // Notas:
    // Para exportar, vamos acumulando el tamaño (partiremos siempre de 0x10 + 4 * num_subfiles)
}

impl Container {
    pub fn from_bytes(bytes: &[u8]) -> Result<Container, AppError> {
        if bytes.len() < Header::SIZE {
            return Err(AppError::new(
                "Container needs at least (16 | 0x10) bytes"
            ));
        }

        let header = Header::from_bytes(bytes)?;

        if bytes.len() < Header::SIZE + (header.num_subfiles as usize * 4) {
            return Err(AppError::new(
                &format!(
                    "Container needs at least ({0} | 0x{0:x}) bytes for {1} subfiles",
                    Header::SIZE + (header.num_subfiles as usize * 4),
                    header.num_subfiles
                )
            ));
        }

        let subfile_offsets = Self::read_subfile_offsets_from_bytes(&bytes[0x10..], header.num_subfiles as usize)?;

        // Actual files
        let files = Self::read_files(bytes, &subfile_offsets)?;

        Ok(Container {
            header,
            subfile_offsets,
            files
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, AppError> {
        let mut bytes = vec![0u8; self.header.filesize as usize];

        self.header.write_bytes(&mut bytes[0..0x10])?; // Write the header
        bytes[0x10..(0x10 + self.subfile_offsets.len() * 4)].copy_from_slice(&self.subfile_offsets
            .iter()
            .flat_map(
                |&x| x.to_le_bytes()
            ).collect::<Vec<u8>>()[..]
        ); // Write the subfile offsets

        for (global_index, &(file_type, local_index)) in self.files.sorted_indices.iter().enumerate() {
            let file_offset = self.subfile_offsets[global_index] as usize;
            match file_type {
                Type::MDL => {
                    self.files.mdl[local_index].write_bytes(&mut bytes[file_offset..])?;
                }
                Type::TEX => {
                    self.files.tex[local_index].write_bytes(&mut bytes[file_offset..])?;
                },
                Type::JNT => todo!(),
                Type::PAT => todo!(),
                Type::SRT => todo!(),
            }
        }

        Ok(bytes)
    }

    fn read_subfile_offsets_from_bytes(bytes: &[u8], num_subfiles: usize) -> Result<Vec<u32>, AppError> {
        if bytes.len() < (num_subfiles * 4) {
            return Err(AppError::new(
                &format!(
                    "Container needs at least ({0} | 0x{0:x}) bytes for {1} subfiles",
                    num_subfiles * 4 + 0x10,
                    num_subfiles
                )
            ));
        }

        let mut subfile_offsets = Vec::with_capacity(num_subfiles);
        for off in (0..(num_subfiles * 4)).step_by(4) {
            let offset = u32::from_le_bytes([
                bytes[off],
                bytes[off + 1],
                bytes[off + 2],
                bytes[off + 3]
            ]);

            subfile_offsets.push(offset);
        }

        Ok(subfile_offsets)
    }

    fn read_files(bytes: &[u8], offsets: &[u32]) -> Result<Files, AppError> {
        let mut mdl = Vec::new();
        let mut tex = Vec::new();
        let mut jnt = Vec::new();
        let mut pat = Vec::new();
        let mut srt = Vec::new();

        let mut sorted_indices = Vec::with_capacity(offsets.len());

        for &offset in offsets {
            let offset = offset as usize;
            if (offset + 3) >= bytes.len() {
                return Err(AppError::new(
                    &format!(
                        "Subfile offset {0} is out of bounds for the container size {1}",
                        offset,
                        bytes.len()
                    )
                ));
            }

            let subfile_type = Type::from_stamp(&bytes[offset..(offset + 4)])?;
            let debug_info = DebugInfo {
                offset: offset as u32
            };

            match subfile_type {
                Type::MDL => {
                    let mdl_file = Mdl::from_bytes(&bytes[offset..], debug_info)?;

                    sorted_indices.push((Type::MDL, mdl.len()));
                    mdl.push(mdl_file);
                },
                Type::TEX => {
                    let tex_file = Tex::from_bytes(&bytes[offset..], debug_info)?;

                    sorted_indices.push((Type::TEX, tex.len()));
                    tex.push(tex_file);
                },
                Type::JNT => {
                    let jnt_file = Jnt::from_bytes(&bytes[offset..])?;

                    sorted_indices.push((Type::JNT, jnt.len()));
                    jnt.push(jnt_file);
                },
                Type::PAT => {
                    let pat_file = Pat::from_bytes(&bytes[offset..])?;

                    sorted_indices.push((Type::PAT, pat.len()));
                    pat.push(pat_file);
                },
                Type::SRT => {
                    let srt_file = Srt::from_bytes(&bytes[offset..])?;

                    sorted_indices.push((Type::SRT, srt.len()));
                    srt.push(srt_file);
                }
            }
        }

        Ok(Files {
            mdl,
            tex,
            jnt,
            pat,
            srt,
            sorted_indices
        })
    }

    pub fn rebase(&mut self) {
        let mut prev_offset = (Header::SIZE + self.subfile_offsets.len() * 4) as u32;
        let mut prev_size = 0u32;

        for (global_index, &(file_type, local_index)) in self.files.sorted_indices.iter().enumerate() {
            let offset = get_4_byte_alignment((prev_offset + prev_size) as usize) as u32;
            self.subfile_offsets[global_index] = offset;

            prev_size = match file_type {
                Type::MDL => {
                    self.files.mdl[local_index].rebase();
                    self.files.mdl[local_index].size() as u32
                },
                Type::TEX => {
                    // self.files.tex[local_index].rebase();
                    self.files.tex[local_index].size() as u32
                },
                Type::JNT => todo!(),
                Type::PAT => todo!(),
                Type::SRT => todo!(),
            };

            prev_offset = offset;
        }

        self.header.filesize = prev_offset + prev_size;
    }

    pub fn get_mdl(&self, index: usize) -> Option<&Mdl> {
        self.files.mdl.get(index)
    }

    pub fn get_mdl_mut(&mut self, index: usize) -> Option<&mut Mdl> {
        self.files.mdl.get_mut(index)
    }

    pub fn get_tex(&self, index: usize) -> Option<&Tex> {
        self.files.tex.get(index)
    }

    pub fn get_tex_mut(&mut self, index: usize) -> Option<&mut Tex> {
        self.files.tex.get_mut(index)
    }
}

#[derive(Debug, Clone)]
struct Header {
    stamp: [u8; 4],
    bom: u16, // Byte Order Mark (0xFEFF for little-endian)
    version: u16,
    filesize: u32,
    header_size: u16, // Size of this header (always 16),
    num_subfiles: u16
}

impl Header {
    const SIZE: usize = 0x10;
    pub fn from_bytes(bytes: &[u8]) -> Result<Header, AppError> {
        if bytes.len() < Header::SIZE {
            return Err(AppError::new(
                "Header needs at least (16 | 0x10) bytes"
            ))
        }

        let stamp = [bytes[0], bytes[1], bytes[2], bytes[3]];
        let bom = u16::from_le_bytes([bytes[4], bytes[5]]);
        let version = u16::from_le_bytes([bytes[6], bytes[7]]);
        let filesize = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let header_size = u16::from_le_bytes([bytes[12], bytes[13]]);
        let num_subfiles = u16::from_le_bytes([bytes[14], bytes[15]]);
         
        Ok(Header {
            stamp,
            bom,
            version,
            filesize,
            header_size,
            num_subfiles
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < Header::SIZE {
            return Err(AppError::new(
                "Header needs at least (16 | 0x10) bytes"
            ))
        }

        buffer[0..4].copy_from_slice(&self.stamp);
        buffer[4..6].copy_from_slice(&self.bom.to_le_bytes());
        buffer[6..8].copy_from_slice(&self.version.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.filesize.to_le_bytes());
        buffer[12..14].copy_from_slice(&self.header_size.to_le_bytes());
        buffer[14..16].copy_from_slice(&self.num_subfiles.to_le_bytes());

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Files {
    mdl: Vec<Mdl>,
    tex: Vec<Tex>,
    jnt: Vec<Jnt>,
    pat: Vec<Pat>,
    srt: Vec<Srt>,
    sorted_indices: Vec<(Type, usize)> // To keep track of the original order of the subfiles
}
