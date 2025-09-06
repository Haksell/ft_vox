use crate::{aabb::AABB, block::BlockType};

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;

pub struct Chunk {
    index: (i32, i32),
    blocks: [[[Option<BlockType>; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH],
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
        let world_x = self.index.0 as f32 * CHUNK_WIDTH as f32;
        let world_y = self.index.1 as f32 * CHUNK_WIDTH as f32;

        AABB::new(
            glam::Vec3::new(world_x, 0.0, world_y),
            glam::Vec3::new(
                world_x + CHUNK_WIDTH as f32,
                CHUNK_HEIGHT as f32,
                world_y + CHUNK_WIDTH as f32,
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
}
