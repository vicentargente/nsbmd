use crate::{error::AppError, traits::BinarySerializable};

use super::name::Name;

#[derive(Debug, Clone)]
pub struct NameList<T> {
    dummy: u8,
    count: u8,
    size: u16, // Important to have this size always up to date
    unknown: Unknown,
    element_size: u16,
    data_section_size: u16,
    data: Vec<T>,
    names: Vec<Name>
}

impl<T> NameList<T>
where T: BinarySerializable
{
    pub fn from_bytes(bytes: &[u8]) -> Result<NameList<T>, AppError> {
        if bytes.len() < 4 {
            return Err(AppError::new("NameList needs at least 4 bytes"));
        }

        let dummy = bytes[0];
        let count = bytes[1];
        let size = u16::from_le_bytes([bytes[2], bytes[3]]);

        if size as usize > bytes.len() {
            return Err(AppError::new(&format!("NameList size is bigger than the buffer size. Expected: {}, got: {}", size, bytes.len())));
        }

        let unknown = Unknown::from_bytes(&bytes[4..], count)?;

        let base_offset = unknown.header.unknown_size as usize;
        
        let element_size = u16::from_le_bytes([bytes[base_offset], bytes[base_offset + 1]]);
        let data_section_size = u16::from_le_bytes([bytes[base_offset + 2], bytes[base_offset + 3]]);

        let mut data = Vec::with_capacity(count as usize);
        let data_offset = base_offset + 4;
        for i in 0..count {
            let offset = data_offset + (i as usize * element_size as usize);
            let element = T::from_bytes(&bytes[offset..])?; // We pass the whole slice from offset, as some data structures need to read data farther than its size
            data.push(element);
        }
        
        let mut names = Vec::with_capacity(count as usize);
        let names_offset = data_offset + (count as usize * element_size as usize);
        for i in 0..count {
            let offset = names_offset + (i as usize * Name::SIZE);
            let name = Name::from_bytes(&bytes[offset..offset + Name::SIZE])?;
            names.push(name);
        }

        Ok(NameList {
            dummy,
            count,
            size,
            unknown,
            element_size,
            data_section_size,
            data,
            names
        })
    }

    pub fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        if buffer.len() < self.size as usize {
            return Err(AppError::new(&format!("NameList size is bigger than the buffer size. Expected: {}, got: {}", self.size, buffer.len())));
        }

        buffer[0] = self.dummy;
        buffer[1] = self.count;
        buffer[2..4].copy_from_slice(&self.size.to_le_bytes());
        self.unknown.write_bytes(&mut buffer[4..])?;

        let base_offset = self.unknown.header.unknown_size as usize;
        buffer[base_offset..base_offset + 2].copy_from_slice(&self.element_size.to_le_bytes());
        buffer[base_offset + 2..base_offset + 4].copy_from_slice(&self.data_section_size.to_le_bytes());

        let data_offset = base_offset + 4;
        for i in 0..self.count {
            let offset = data_offset + (i as usize * self.element_size as usize);

            self.data[i as usize].write_bytes(&mut buffer[offset..])?;
        }

        let names_offset = data_offset + (self.count as usize * self.element_size as usize);
        for i in 0..self.count {
            let offset = names_offset + (i as usize * Name::SIZE);
            self.names[i as usize].write_bytes(&mut buffer[offset..])?;
        }

        Ok(())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, AppError> {
        let mut bytes = vec![0u8; self.size as usize];

        self.write_bytes(&mut bytes)?;

        Ok(bytes)
    }

    pub fn size(&self) -> usize {
        self.size as usize
    }

    pub fn rebase(&mut self) {
        self.size = (
            4 + // dummy + count + size
            self.unknown.size() +
            4 + // element_size + data_section_size
            self.data.len() * self.element_size as usize +
            self.names.len() * Name::SIZE
        ) as u16;
    }
}

// Implementing the array-like interface for NameList
impl<T> NameList<T> {
    pub fn len(&self) -> usize {
        self.count as usize
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    pub fn get_name(&self, index: usize) -> Option<&Name> {
        self.names.get(index)
    }

    pub fn data_iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    pub fn data_iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut()
    }

    pub fn names_iter(&self) -> impl Iterator<Item = &Name> {
        self.names.iter()
    }

    pub fn names_iter_mut(&mut self) -> impl Iterator<Item = &mut Name> {
        self.names.iter_mut()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }

    pub fn get_name_mut(&mut self, index: usize) -> Option<&mut Name> {
        self.names.get_mut(index)
    }
}

#[derive(Debug, Clone)]
struct Unknown {
    header: UnknownHeader,
    unknown: Vec<u32>
}

impl Unknown {
    fn from_bytes(bytes: &[u8], count: u8) -> Result<Unknown, AppError> {
        let header = UnknownHeader::from_bytes(bytes)?;
        let mut unknown = Vec::with_capacity(count as usize);

        let unknown_offset = 8;
        for i in 0..count {
            let offset = unknown_offset + (i as usize * 4);
            let value = u32::from_le_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]);
            unknown.push(value);
        }

        Ok(Unknown {
            header,
            unknown
        })
    }

    fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        self.header.write_bytes(buffer)?;
        buffer[UnknownHeader::SIZE..(UnknownHeader::SIZE + self.unknown.len() * 4)].copy_from_slice(
            &self.unknown.iter().flat_map(
                |&x| x.to_le_bytes()
            ).collect::<Vec<u8>>()[..]
        );

        Ok(())
    }

    fn size(&self) -> usize {
        UnknownHeader::SIZE + self.unknown.len() * 4
    }
}

#[derive(Debug, Clone)]
struct UnknownHeader {
    subheader_size: u16, // Size of this UnknownHeader?
    unknown_size: u16, // Size of the full Unknown?
    unknown: u32
}

impl UnknownHeader {
    const SIZE: usize = 8;

    fn from_bytes(bytes: &[u8]) -> Result<UnknownHeader, AppError> {
        Self::check_size(bytes.len())?;

        let subheader_size = u16::from_le_bytes([bytes[0], bytes[1]]);
        let unknown_size = u16::from_le_bytes([bytes[2], bytes[3]]);
        let unknown = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

        Ok(UnknownHeader {
            subheader_size,
            unknown_size,
            unknown
        })
    }

    fn write_bytes(&self, buffer: &mut [u8]) -> Result<(), AppError> {
        Self::check_size(buffer.len())?;

        buffer[0..2].copy_from_slice(&self.subheader_size.to_le_bytes());
        buffer[2..4].copy_from_slice(&self.unknown_size.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.unknown.to_le_bytes());

        Ok(())
    }

    fn check_size(size: usize) -> Result<(), AppError> {
        if size < UnknownHeader::SIZE {
            return Err(AppError::new("UnknownHeader needs at least 8 bytes"));
        }

        Ok(())
    }
}
