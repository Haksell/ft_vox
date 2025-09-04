use {
    crate::{
        block::BlockType,
        chunk::{Chunk, CHUNK_HEIGHT, CHUNK_WIDTH},
        noise::{PerlinNoise, PerlinNoiseBuilder},
    },
    std::collections::HashMap,
};

pub struct World {
    noise: PerlinNoise,
    chunks: HashMap<(i32, i32), Chunk>,

    height_scale: f64,
    height_offset: f64,
}

impl World {
    pub fn new(seed: u64) -> Self {
        let chunks = HashMap::new();
        let noise = PerlinNoiseBuilder::new(seed).build();

        Self {
            noise,
            chunks,
            height_scale: CHUNK_HEIGHT as f64 * 0.4, // Height variation range
            height_offset: CHUNK_HEIGHT as f64 * 0.4, // Base height
        }
    }

    fn generate_height_at(&self, world_x: f64, world_y: f64) -> f64 {
        let noise = self.noise.noise2d(world_x, world_y);

        self.height_offset + (noise * self.height_scale)
    }

    fn generate_chunk_blocks(
        &self,
        chunk_x: i32,
        chunk_y: i32,
    ) -> [[[Option<BlockType>; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH] {
        let mut blocks = [[[None; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH];

        for x in 0..CHUNK_WIDTH {
            let world_x = (chunk_x * CHUNK_WIDTH as i32) + x as i32;

            for y in 0..CHUNK_WIDTH {
                let world_y = (chunk_y * CHUNK_WIDTH as i32) + y as i32;

                let height = self.generate_height_at(world_x as f64, world_y as f64) as usize;

                for z in 0..CHUNK_HEIGHT {
                    blocks[x][y][z] = if z == height {
                        Some(BlockType::Snow)
                    } else if z < height {
                        Some(BlockType::Grass)
                    } else {
                        None
                    }
                }
            }
        }

        blocks
    }

    pub fn get_chunk(&mut self, chunk_x: i32, chunk_y: i32) -> &Chunk {
        if !self.chunks.contains_key(&(chunk_x, chunk_y)) {
            let blocks = self.generate_chunk_blocks(chunk_x, chunk_y);
            let chunk = Chunk::new(blocks);
            self.chunks.insert((chunk_x, chunk_y), chunk);
        }

        &self.chunks[&(chunk_x, chunk_y)]
    }
}
