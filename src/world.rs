use {
    crate::{
        chunk::Chunk,
        noise::{PerlinNoise, PerlinNoiseBuilder},
    },
    std::collections::HashMap,
};

pub struct World {
    seed: u64,
    noise: PerlinNoise,
    chunks: HashMap<(i32, i32, i32), Chunk>,
}

impl World {
    pub fn new(seed: u64) -> Self {
        let chunks = HashMap::new();
        let noise = PerlinNoiseBuilder::new(seed).build();

        Self {
            seed,
            noise,
            chunks,
        }
    }

    pub fn get_chunk(&mut self, x: i32, y: i32, z: i32) -> &Chunk {
        self.chunks
            .entry((x, y, z))
            .or_insert_with(|| Chunk::new(&self.noise, x, y, z))
    }
}
