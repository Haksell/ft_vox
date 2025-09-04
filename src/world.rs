use {
    crate::{
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
        let base_noise = self.noise.noise2d(world_x * 0.01, world_y * 0.01);
        let detail_noise = self.noise.noise2d(world_x * 0.05, world_y * 0.05) * 0.3;
        let fine_detail = self.noise.noise2d(world_x * 0.1, world_y * 0.1) * 0.1;

        let combined = base_noise + detail_noise + fine_detail;

        self.height_offset + (combined * self.height_scale)
    }

    fn generate_chunk_blocks(
        &self,
        chunk_x: i32,
        chunk_y: i32,
    ) -> [[[bool; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH] {
        let mut blocks = [[[false; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH];

        for x in 0..CHUNK_WIDTH {
            let world_x = (chunk_x * CHUNK_WIDTH as i32) + x as i32;

            for y in 0..CHUNK_WIDTH {
                let world_y = (chunk_y * CHUNK_WIDTH as i32) + y as i32;

                let height = self.generate_height_at(world_x as f64, world_y as f64);
                let solid_height = height.floor() as usize;

                for z in 0..CHUNK_HEIGHT {
                    blocks[x][y][z] = y <= solid_height && solid_height < CHUNK_HEIGHT;
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
