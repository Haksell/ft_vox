use crate::block::BlockType;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;

pub struct Chunk {
    blocks: [[[Option<BlockType>; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH],
}

impl Chunk {
    pub fn new(blocks: [[[Option<BlockType>; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH]) -> Self {
        Self { blocks }
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
