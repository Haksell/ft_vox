use crate::{
    aabb::AABB,
    block::BlockType,
    face::{Face, FACES},
    vertex::Vertex,
};

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;

pub struct Chunk {
    index: (i32, i32),
    blocks: [[[Option<BlockType>; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH],
}

pub struct AdjacentChunks<'a> {
    pub north: Option<(&'a Chunk, usize)>, // +y direction
    pub south: Option<(&'a Chunk, usize)>, // -y direction
    pub east: Option<(&'a Chunk, usize)>,  // +x direction
    pub west: Option<(&'a Chunk, usize)>,  // -x direction
}

impl Chunk {
    pub fn new(
        index: (i32, i32),
        blocks: [[[Option<BlockType>; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH],
    ) -> Self {
        Self { index, blocks }
    }

    pub fn get_index(&self) -> (i32, i32) {
        self.index
    }

    pub fn bounding_box(&self) -> AABB {
        let (x, y) = self.index;
        let world_x = x as f32 * CHUNK_WIDTH as f32;
        let world_y = y as f32 * CHUNK_WIDTH as f32;

        AABB::new(
            glam::Vec3::new(world_x, world_y, 0.0),
            glam::Vec3::new(
                world_x + CHUNK_WIDTH as f32,
                world_y + CHUNK_WIDTH as f32,
                CHUNK_HEIGHT as f32,
            ),
        )
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<BlockType> {
        if x < CHUNK_WIDTH && y < CHUNK_WIDTH && z < CHUNK_HEIGHT {
            self.blocks[x][y][z]
        } else {
            None
        }
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Option<BlockType>) {
        if x < CHUNK_WIDTH && y < CHUNK_WIDTH && z < CHUNK_HEIGHT {
            self.blocks[x][y][z] = block;
        }
    }

    fn is_face_visible(
        &self,
        neighbor_x: i32,
        neighbor_y: i32,
        neighbor_z: i32,
        adjacent: &AdjacentChunks,
        lod_step: usize,
    ) -> bool {
        if neighbor_z < 0 {
            return false;
        }
        if neighbor_z >= CHUNK_HEIGHT as i32 {
            return true;
        }

        if neighbor_x >= 0
            && neighbor_x < CHUNK_WIDTH as i32
            && neighbor_y >= 0
            && neighbor_y < CHUNK_WIDTH as i32
        {
            return self
                .get_block(
                    neighbor_x as usize,
                    neighbor_y as usize,
                    neighbor_z as usize,
                )
                .is_none();
        }

        // TODO: no loops, just check if used neighbor is present

        match (neighbor_x, neighbor_y) {
            (x, y) if x >= CHUNK_WIDTH as i32 && y >= 0 && y < CHUNK_WIDTH as i32 => {
                let Some((adj_chunk, adj_lod_step)) = adjacent.east else {
                    return true;
                };
                for dx in 0..lod_step {
                    for dy in 0..lod_step {
                        if adj_chunk
                            .get_block(dx, y as usize + dy, neighbor_z as usize)
                            .is_none()
                        {
                            return true;
                        }
                    }
                }
                false
            }
            (x, y) if x < 0 && y >= 0 && y < CHUNK_WIDTH as i32 => {
                let Some((adj_chunk, adj_lod_step)) = adjacent.west else {
                    return true;
                };
                for dx in 0..lod_step {
                    for dy in 0..lod_step {
                        if adj_chunk
                            .get_block(
                                CHUNK_WIDTH - lod_step + dx,
                                y as usize + dy,
                                neighbor_z as usize,
                            )
                            .is_none()
                        {
                            return true;
                        }
                    }
                }
                false
            }
            (x, y) if y >= CHUNK_WIDTH as i32 && x >= 0 && x < CHUNK_WIDTH as i32 => {
                let Some((adj_chunk, adj_lod_step)) = adjacent.north else {
                    return true;
                };
                for dx in 0..lod_step {
                    for dy in 0..lod_step {
                        if adj_chunk
                            .get_block(x as usize + dx, dy, neighbor_z as usize)
                            .is_none()
                        {
                            return true;
                        }
                    }
                }
                false
            }
            (x, y) if y < 0 && x >= 0 && x < CHUNK_WIDTH as i32 => {
                let Some((adj_chunk, adj_lod_step)) = adjacent.south else {
                    return true;
                };
                for dx in 0..lod_step {
                    for dy in 0..lod_step {
                        if adj_chunk
                            .get_block(
                                x as usize + dx,
                                CHUNK_WIDTH - lod_step + dy,
                                neighbor_z as usize,
                            )
                            .is_none()
                        {
                            return true;
                        }
                    }
                }
                false
            }
            _ => unreachable!(),
        }
    }

    fn create_face(
        block: BlockType,
        position: glam::Vec3,
        face: Face,
        lod_step: usize,
    ) -> ([Vertex; 4], [u16; 6]) {
        let positions = face.positions();

        // stretch uv in x and y but not z direction
        let mut uvs = face.uvs();
        for y in 0..4 {
            uvs[y][0] *= lod_step as f32;
            if matches!(face, Face::Top | Face::Bottom) {
                uvs[y][1] *= lod_step as f32;
            }
        }

        let vertices = std::array::from_fn(|i| Vertex {
            position: [
                position.x + positions[i][0] * lod_step as f32,
                position.y + positions[i][1] * lod_step as f32,
                position.z + positions[i][2] as f32,
            ],
            tex_coords: uvs[i],
            atlas_offset: match face {
                Face::Top => block.atlas_offset_top(),
                Face::Bottom => block.atlas_offset_bottom(),
                Face::Left | Face::Right | Face::Front | Face::Back => block.atlas_offset_side(),
            },
        });

        let indices = [0, 1, 2, 2, 3, 0];

        (vertices, indices)
    }

    pub fn generate_mesh(
        &self,
        lod_step: usize,
        adjacent: &AdjacentChunks,
    ) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0;

        for local_x in (0..CHUNK_WIDTH).step_by(lod_step) {
            for local_y in (0..CHUNK_WIDTH).step_by(lod_step) {
                for local_z in 0..CHUNK_HEIGHT {
                    let Some(block) = self.get_block(local_x, local_y, local_z) else {
                        continue;
                    };

                    let position = glam::Vec3::new(local_x as f32, local_y as f32, local_z as f32);

                    for face in FACES {
                        let (dx, dy, dz) = face.normal();
                        let neighbor_x = local_x as i32 + dx * lod_step as i32;
                        let neighbor_y = local_y as i32 + dy * lod_step as i32;
                        let neighbor_z = local_z as i32 + dz as i32;

                        if self
                            .is_face_visible(neighbor_x, neighbor_y, neighbor_z, adjacent, lod_step)
                        {
                            let (face_verts, face_indices) =
                                Self::create_face(block, position, face, lod_step);

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
}
