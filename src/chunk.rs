use crate::{noise::PerlinNoise, vertex::Vertex};

pub const CHUNK_SIZE: usize = 16;

pub struct Chunk {
    blocks: [[[bool; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
}

impl Chunk {
    pub fn new(pn: &PerlinNoise, chunk_x: i32, chunk_y: i32, chunk_z: i32) -> Self {
        let mut blocks = [[[false; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

        for x in 0..CHUNK_SIZE {
            // Convert local chunk coordinates to world coordinates
            let wx = (chunk_x * CHUNK_SIZE as i32) + x as i32;
            let nx = wx as f64;

            for y in 0..CHUNK_SIZE {
                let wy = (chunk_y * CHUNK_SIZE as i32) + y as i32;
                let ny = wy as f64;

                for z in 0..CHUNK_SIZE {
                    let wz = (chunk_z * CHUNK_SIZE as i32) + z as i32;
                    let nz = wz as f64;

                    // Use world coordinates for noise generation
                    let noise_value = pn.noise3d(nx, ny, nz);

                    if noise_value > 0.5 {
                        blocks[x][y][z] = true;
                    }
                }
            }
        }

        Self {
            blocks,
            chunk_x,
            chunk_y,
            chunk_z,
        }
    }

    pub fn mesh(&self) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0;

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if !self.blocks[x][y][z] {
                        continue;
                    }

                    let position = glam::Vec3::new(x as f32, y as f32, z as f32);

                    // Check neighbors to determine visible faces
                    let faces = [
                        (1, 0, 0),  // Right
                        (-1, 0, 0), // Left
                        (0, 1, 0),  // Top
                        (0, -1, 0), // Bottom
                        (0, 0, 1),  // Front
                        (0, 0, -1), // Back
                    ];

                    for (dx, dy, dz) in faces.iter() {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        let nz = z as i32 + dz;

                        // Explicitly check if the neighboring block is empty or out of bounds
                        let is_face_visible = nx < 0
                            || ny < 0
                            || nz < 0
                            || nx >= CHUNK_SIZE as i32
                            || ny >= CHUNK_SIZE as i32
                            || nz >= CHUNK_SIZE as i32
                            || (nx >= 0
                                && ny >= 0
                                && nz >= 0
                                && nx < CHUNK_SIZE as i32
                                && ny < CHUNK_SIZE as i32
                                && nz < CHUNK_SIZE as i32
                                && !self.blocks[nx as usize][ny as usize][nz as usize]);

                        if is_face_visible {
                            let normal = glam::Vec3::new(*dx as f32, *dy as f32, *dz as f32);
                            // Generate face vertices
                            let (face_verts, face_indices) = Self::face(position, normal);

                            vertices.extend(face_verts);
                            indices.extend(face_indices.iter().map(|i| *i + index_offset));
                            index_offset += 4;
                        }
                    }
                }
            }
        }

        (vertices, indices)
    }

    fn face(position: glam::Vec3, normal: glam::Vec3) -> (Vec<Vertex>, Vec<u16>) {
        let dx = normal.x;
        let dy = normal.y;
        let dz = normal.z;
        let normal = (dx, dy, dz);

        let positions = match normal {
            (1., 0., 0.) => [
                // Right face
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 0.0],
            ],
            (-1., 0., 0.) => [
                // Left face
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [0.0, 1.0, 0.0],
            ],
            (0., 1., 0.) => [
                // Top face
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            (0., -1., 0.) => [
                // Bottom face
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ],
            (0., 0., 1.) => [
                // Front face
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            (0., 0., -1.) => [
                // Back face
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            _ => unreachable!(),
        };

        let uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        let vertices: Vec<Vertex> = positions
            .iter()
            .zip(uvs.iter())
            .map(|(pos, uv)| Vertex {
                position: [
                    position.x + pos[0],
                    position.y + pos[1],
                    position.z + pos[2],
                ],
                tex_coords: *uv,
            })
            .collect();

        let indices = vec![0, 1, 2, 2, 3, 0];

        (vertices, indices)
    }
}
