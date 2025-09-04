use {
    crate::{
        chunk::Chunk,
        noise::{PerlinNoise, PerlinNoiseBuilder},
    },
    std::collections::HashMap,
};

pub struct World {
    noise: PerlinNoise,
    chunks: HashMap<(i32, i32), Chunk>,
}

impl World {
    pub fn new(seed: u64) -> Self {
        let chunks = HashMap::new();
        let noise = PerlinNoiseBuilder::new(seed).build();

        Self { noise, chunks }
    }

    pub fn get_chunk(&mut self, x: i32, z: i32) -> &Chunk {
        self.chunks
            .entry((x, z))
            .or_insert_with(|| Chunk::new(&self.noise, x, z))
    }
}
