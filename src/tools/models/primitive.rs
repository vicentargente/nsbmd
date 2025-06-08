use super::vertex::Vertex;

#[derive(Debug, Clone)]
pub enum Primitive {
    Triangle {
        vertices: Vec<Vertex>,
        indices: Vec<u32>
    }
}

impl Primitive {
    pub fn vertices(&self) -> &Vec<Vertex> {
        match self {
            Primitive::Triangle { vertices, .. } => vertices,
        }
    }

    pub fn vertices_mut(&mut self) -> &mut Vec<Vertex> {
        match self {
            Primitive::Triangle { vertices, .. } => vertices
        }
    }

    pub fn indices(&self) -> &Vec<u32> {
        match self {
            Primitive::Triangle { indices, .. } => indices,
        }
    }
}
