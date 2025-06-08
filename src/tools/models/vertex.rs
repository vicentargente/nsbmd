use crate::{error::AppError, util::math::matrix::Matrix};

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Position,
    pub tex_coord: TexCoord,
    pub bone_id: u32
}

impl Vertex {
    pub fn new(position: Position, tex_coord: TexCoord, bone_id: u32) -> Self {
        Vertex {
            position,
            tex_coord,
            bone_id
        }
    }

    pub fn apply_transform(&mut self, transform: &Matrix) -> Result<(), AppError> {
        if transform.width() != 4 || transform.height() != 4 {
            return Err(AppError::new("Transform matrix must be 4x4."));
        }
        
        let pos = Matrix::new(1, 4, vec![self.position.x, self.position.y, self.position.z, 1.0])?;
        let transformed_pos = transform.clone() * pos;
        self.position.x = transformed_pos.get(0, 0)?;
        self.position.y = transformed_pos.get(1, 0)?;
        self.position.z = transformed_pos.get(2, 0)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

#[derive(Debug, Clone)]
pub struct TexCoord {
    pub u: f32,
    pub v: f32
}
