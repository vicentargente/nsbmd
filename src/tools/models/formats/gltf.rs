use crate::{error::AppError, tools::models::{primitive::Primitive, vertex::{Position, TexCoord, Vertex}}};

#[derive(Debug, Clone)]
pub struct Gltf {
    meshes: Vec<Mesh>
}

impl Gltf {
    pub fn open(path: &str) -> Result<Gltf, AppError> {
        let (document, buffers, _images) = gltf::import(path)
            .map_err(|err| AppError::new(&err.to_string()))?;

        let mut meshes: Vec<Mesh> = Vec::new();

        for node in document.nodes() {
            if let Some(mesh) = node.mesh() {
                if let Some(skin) = node.skin() {
                    let mut primitives = Vec::new();
                    let mut bones = Vec::new();

                    for primitive in mesh.primitives() {
                        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                        let indices = if let Some(iter) = reader.read_indices() {
                            iter.into_u32().collect::<Vec<u32>>()
                        }
                        else { continue; };

                        let positions = if let Some(iter) = reader.read_positions() {
                            iter.collect::<Vec<[f32; 3]>>()
                        }
                        else { continue; };

                        let tex_coords = reader.read_tex_coords(0)
                            .map(|uvs| uvs.into_f32().collect())
                            .unwrap_or(vec![[0.0, 0.0]; positions.len()]);


                        let joint_indices = if let Some(joints) = reader.read_joints(0) {
                            joints.into_u16().collect::<Vec<[u16; 4]>>()
                        }
                        else { continue; };

                        let joint_weights = if let Some(weights) = reader.read_weights(0) {
                            weights.into_f32().collect::<Vec<[f32; 4]>>()
                        }
                        else { continue; };

                        if positions.len() != joint_indices.len() {
                            return Err(AppError::new("Positions and joint indices length mismatch"));
                        }

                        if positions.len() != joint_weights.len() {
                            return Err(AppError::new("Positions and joint weights length mismatch"));
                        }

                        bones = skin.joints()
                            .map(|joint| joint.name().unwrap_or("unnamed_bone").to_string())
                            .collect::<Vec<String>>();

                        let mut vertices: Vec<Vertex> = Vec::with_capacity(positions.len());
                        for i in 0..positions.len() {
                            let weights = joint_weights[i];
                            let joints = joint_indices[i];

                            let bone_index_in_vertex = weights.iter()
                                .position(|&weight| weight == 1.0)
                                .expect("At least one weight should be 1.0");

                            let joint_index = joints[bone_index_in_vertex] as usize;

                            let vertex = Vertex::new(
                                Position {
                                    x: positions[i][0],
                                    y: positions[i][1],
                                    z: positions[i][2]
                                },
                                TexCoord {
                                    u: tex_coords[i][0],
                                    v: tex_coords[i][1]
                                },
                                joint_index as u32
                            );

                            vertices.push(vertex);
                        }

                        let primitive_info = match primitive.mode() {
                            gltf::mesh::Mode::Points => {
                                return Err(AppError::new("Points mode is not supported"));
                            },
                            gltf::mesh::Mode::Lines => {
                                return Err(AppError::new("Lines mode is not supported"));
                            },
                            gltf::mesh::Mode::LineLoop => {
                                return Err(AppError::new("LineLoop mode is not supported"));
                            },
                            gltf::mesh::Mode::LineStrip => {
                                return Err(AppError::new("LineStrip mode is not supported"));
                            },
                            gltf::mesh::Mode::Triangles => {
                                Primitive::Triangle { vertices, indices }
                            },
                            gltf::mesh::Mode::TriangleStrip => {
                                Primitive::Triangle { vertices, indices }
                            },
                            gltf::mesh::Mode::TriangleFan => {
                                return Err(AppError::new("TriangleFan mode is not supported"));
                            },
                        };

                        primitives.push(primitive_info);
                    }
                    
                    let mesh = Mesh {
                        primitives,
                        bones
                    };

                    meshes.push(mesh);

                    break; // We only take the first mesh with a skin
                }
            }
        }

        Ok(Gltf { meshes }) 
    }

    pub fn primitives(&self) -> Vec<&Primitive> {
        self.meshes.iter()
            .flat_map(|mesh| &mesh.primitives)
            .collect()
    }

    pub fn bones(&self) -> Vec<&String> {
        self.meshes.iter()
            .flat_map(|mesh| &mesh.bones)
            .collect()
    }
}

#[derive(Debug, Clone)]
struct Mesh {
    primitives: Vec<Primitive>,
    bones: Vec<String>
}
