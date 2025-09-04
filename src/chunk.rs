use crate::{
    block::BlockType,
    face::{Face, FACES},
    vertex::Vertex,
};

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 32;

pub struct Chunk {
    blocks: [[[Option<BlockType>; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH],
}

impl Chunk {
    pub fn new(blocks: [[[Option<BlockType>; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH]) -> Self {
        Self { blocks }
    }

    pub fn mesh(&self) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0;

        for x in 0..CHUNK_WIDTH {
            for y in 0..CHUNK_WIDTH {
                for z in 0..CHUNK_HEIGHT {
                    let Some(block) = self.blocks[x][y][z] else {
                        continue;
                    };

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
                            || self.blocks[nx as usize][ny as usize][nz as usize].is_none();

                        if is_face_visible {
                            let (face_verts, face_indices) = Self::face(face, position, block);

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

    fn face(face: Face, position: glam::Vec3, block: BlockType) -> ([Vertex; 4], [u16; 6]) {
        let positions = face.positions();
        let uvs = face.uvs();

        let vertices = std::array::from_fn(|i| Vertex {
            position: [
                position.x + positions[i][0],
                position.y + positions[i][1],
                position.z + positions[i][2],
            ],
            tex_coords: uvs[i],
            atlas_offset: block.atlas_offset(),
        });

        let indices = [0, 1, 2, 2, 3, 0];

        (vertices, indices)
    }
}
