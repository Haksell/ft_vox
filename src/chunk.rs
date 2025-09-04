use crate::{
    face::{Face, FACES},
    noise::PerlinNoise,
    vertex::Vertex,
};

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 64;

pub struct Chunk {
    blocks: [[[bool; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH],
    chunk_x: i32,
    chunk_y: i32,
}

impl Chunk {
    pub fn new(pn: &PerlinNoise, chunk_x: i32, chunk_y: i32) -> Self {
        let mut blocks = [[[false; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH];

        for x in 0..CHUNK_WIDTH {
            // Convert local chunk coordinates to world coordinates
            let wx = (chunk_x * CHUNK_WIDTH as i32) + x as i32;
            let nx = wx as f64;

            for y in 0..CHUNK_WIDTH {
                let wy = (chunk_y * CHUNK_WIDTH as i32) + y as i32;
                let ny = wy as f64;

                // Use world coordinates for noise generation
                let noise_value = pn.noise2d(nx, ny);

                for z in 0..CHUNK_HEIGHT {
                    blocks[x][y][z] = (z as f64) < noise_value * CHUNK_HEIGHT as f64;
                }
            }
        }

        Self {
            blocks,
            chunk_x,
            chunk_y,
        }
    }

    pub fn mesh(&self) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0;

        for x in 0..CHUNK_WIDTH {
            for y in 0..CHUNK_WIDTH {
                for z in 0..CHUNK_HEIGHT {
                    if !self.blocks[x][y][z] {
                        continue;
                    }

                    let position = glam::Vec3::new(x as f32, y as f32, z as f32);

                    // Check neighbors to determine visible faces
                    for face in FACES {
                        let (dx, dy, dz) = face.normal();
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        let nz = z as i32 + dz;

                        // Explicitly check if the neighboring block is empty or out of bounds
                        let is_face_visible = nx < 0
                            || ny < 0
                            || nz < 0
                            || nx >= CHUNK_WIDTH as i32
                            || ny >= CHUNK_WIDTH as i32
                            || nz >= CHUNK_HEIGHT as i32
                            || !self.blocks[nx as usize][ny as usize][nz as usize];

                        if is_face_visible {
                            let (face_verts, face_indices) = Self::face(face, position);

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

    fn face(face: Face, position: glam::Vec3) -> (Vec<Vertex>, Vec<u16>) {
        let positions = face.positions();
        let uvs = match face.normal() {
            (1, 0, 0) => [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            (-1, 0, 0) => [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            (0, 1, 0) => [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            (0, -1, 0) => [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            (0, 0, 1) => [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
            (0, 0, -1) => [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            _ => unreachable!(),
        };

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
