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
    pub north: Option<&'a Chunk>, // +y direction
    pub south: Option<&'a Chunk>, // -y direction
    pub east: Option<&'a Chunk>,  // +x direction
    pub west: Option<&'a Chunk>,  // -x direction
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

    fn get_neighbor_block(
        &self,
        neighbor_x: i32,
        neighbor_y: i32,
        neighbor_z: i32,
        adjacent: &AdjacentChunks,
        lod_step: usize,
    ) -> Option<BlockType> {
        if neighbor_z < 0 || neighbor_z >= CHUNK_HEIGHT as i32 {
            return None;
        }

        if neighbor_x >= 0
            && neighbor_x < CHUNK_WIDTH as i32
            && neighbor_y >= 0
            && neighbor_y < CHUNK_WIDTH as i32
        {
            return self.get_block(
                neighbor_x as usize,
                neighbor_y as usize,
                neighbor_z as usize,
            );
        }

        match (neighbor_x, neighbor_y) {
            (x, y) if x >= CHUNK_WIDTH as i32 && y >= 0 && y < CHUNK_WIDTH as i32 => {
                adjacent.east?.get_block(0, y as usize, neighbor_z as usize)
            }
            (x, y) if x < 0 && y >= 0 && y < CHUNK_WIDTH as i32 => {
                adjacent
                    .west?
                    .get_block(CHUNK_WIDTH - lod_step, y as usize, neighbor_z as usize)
            }
            (x, y) if y >= CHUNK_WIDTH as i32 && x >= 0 && x < CHUNK_WIDTH as i32 => adjacent
                .north?
                .get_block(x as usize, 0, neighbor_z as usize),
            (x, y) if y < 0 && x >= 0 && x < CHUNK_WIDTH as i32 => {
                adjacent
                    .south?
                    .get_block(x as usize, CHUNK_WIDTH - lod_step, neighbor_z as usize)
            }
            _ => None,
        }
    }

    fn create_face(
        block: BlockType,
        position: glam::Vec3,
        face: Face,
        lod_step: usize,
    ) -> ([Vertex; 4], [u16; 6]) {
        let positions = face.positions();
        let uvs = face.uvs();

        let vertices = std::array::from_fn(|i| Vertex {
            position: [
                position.x + positions[i][0] * lod_step as f32,
                position.y + positions[i][1] * lod_step as f32,
                position.z + positions[i][2] * lod_step as f32,
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
                for local_z in (0..CHUNK_HEIGHT).step_by(lod_step) {
                    let Some(block) = self.get_block(local_x, local_y, local_z) else {
                        continue;
                    };

                    let position = glam::Vec3::new(local_x as f32, local_y as f32, local_z as f32);

                    for face in FACES {
                        let (dx, dy, dz) = face.normal();
                        let neighbor_x = local_x as i32 + dx * lod_step as i32;
                        let neighbor_y = local_y as i32 + dy * lod_step as i32;
                        let neighbor_z = local_z as i32 + dz * lod_step as i32;

                        let is_face_visible = neighbor_z >= 0
                            && self
                                .get_neighbor_block(
                                    neighbor_x, neighbor_y, neighbor_z, adjacent, lod_step,
                                )
                                .is_none();

                        if is_face_visible {
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
