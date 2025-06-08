use model::Model;

use crate::{data_structures::name_list::NameList, debug_info::DebugInfo, error::AppError};

pub mod model;

#[derive(Debug, Clone)]
pub struct Mdl {
    stamp: [u8; 4],
    filesize: u32,
    models: NameList<u32>,

    // Actual data
    models_data: Vec<Model>,

    // Debug info
    _debug_info: DebugInfo
}

impl Mdl {
    pub fn from_bytes(bytes: &[u8], debug_info: DebugInfo) -> Result<Mdl, AppError> {
        if bytes.len() < 8 {
            return Err(AppError::new("MDL needs at least 8 bytes to start reading"))
        }

        let stamp = [
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3]
        ];

        let filesize = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

        if bytes.len() < filesize as usize {
            return Err(AppError::new(&format!("MDL needs at least {} bytes", filesize)))
        }

        let bytes = &bytes[..filesize as usize];

        let models = NameList::from_bytes(&bytes[8..])?;

        let mut models_data = Vec::with_capacity(models.len());
        for &offset in models.data_iter() {
            let debug_info = DebugInfo {
                offset: debug_info.offset + offset
            };

            let offset = offset as usize;
            let model = Model::from_bytes(&bytes[offset..], debug_info)?;
            models_data.push(model);
        }

        Ok(Mdl {
            stamp,
            filesize,
            models,
            models_data,
            _debug_info: debug_info
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < self.filesize as usize {
            return Err(AppError::new("Buffer is too small to write MDL"));
        }

        buffer[0..4].copy_from_slice(&self.stamp); // Write stamp
        buffer[4..8].copy_from_slice(&self.filesize.to_le_bytes()); // Write filesize
        self.models.write_bytes(&mut buffer[8..]).unwrap(); // Write models

        for (i, &offset) in self.models.data_iter().enumerate() {
            let offset = offset as usize;
            let model = &self.models_data[i];
            model.write_bytes(&mut buffer[offset..])?;
        }

        Ok(())
    }

    pub fn rebase(&mut self) {
        if self.models.len() != self.models_data.len() {
            // This should never happen
            panic!("Unexpected mismatch between models header and models data");
        }

        let mut prev_offset = 8 + self.models.size() as u32;
        let mut prev_size = 0u32;

        let iter = self.models.data_iter_mut().zip(self.models_data.iter_mut());
        for (offset, model) in iter {
            model.rebase();

            let size = model.size() as u32;
            
            let new_offset = prev_offset + prev_size;
            *offset = new_offset;

            prev_offset = new_offset;
            prev_size = size;
        }

        // Update the filesize
        self.filesize = prev_offset + prev_size;
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0u8; self.filesize as usize]; // write buffer

        self.write_bytes(&mut bytes).unwrap(); // Write the header

        bytes
    }

    pub fn size(&self) -> usize {
        self.filesize as usize
    }

    pub fn get_model_mut(&mut self, index: usize) -> Option<&mut Model> {
        self.models_data.get_mut(index)
    }
}
