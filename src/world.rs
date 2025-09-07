use {
    crate::{
        block::BlockType,
        chunk::{AdjacentChunks, Chunk, CHUNK_HEIGHT, CHUNK_WIDTH},
        noise::{PerlinNoise, PerlinNoiseBuilder},
        vertex::Vertex,
    },
    std::collections::HashMap,
};

pub const RENDER_DISTANCE: usize = 10;

pub struct World {
    noise: PerlinNoise,
    chunks: HashMap<(i32, i32), Chunk>,

    height_scale: f64,
    height_offset: f64,
    render_distance: usize,
}

impl World {
    pub fn new(seed: u64) -> Self {
        let chunks = HashMap::new();
        let noise = PerlinNoiseBuilder::new(seed).build();

        Self {
            noise,
            chunks,
            height_scale: CHUNK_HEIGHT as f64 * 0.6, // Height variation range
            height_offset: CHUNK_HEIGHT as f64 * 0.2, // Base height
            render_distance: RENDER_DISTANCE,
        }
    }

    pub fn get_render_distance(&self) -> usize {
        self.render_distance
    }

    pub fn get_chunk_index_from_position(&self, world_x: f32, world_y: f32) -> (i32, i32) {
        let chunk_x = (world_x / CHUNK_WIDTH as f32).floor() as i32;
        let chunk_y = (world_y / CHUNK_WIDTH as f32).floor() as i32;

        (chunk_x, chunk_y)
    }

    pub fn get_chunk_if_loaded(&self, chunk_x: i32, chunk_y: i32) -> Option<&Chunk> {
        self.chunks.get(&(chunk_x, chunk_y))
    }

    pub fn get_chunk(&mut self, chunk_x: i32, chunk_y: i32) -> &Chunk {
        if !self.chunks.contains_key(&(chunk_x, chunk_y)) {
            let blocks = self.generate_chunk_blocks(chunk_x, chunk_y);
            let index = (chunk_x, chunk_y);
            let chunk = Chunk::new(index, blocks);
            self.chunks.insert((chunk_x, chunk_y), chunk);
        }

        &self.chunks[&(chunk_x, chunk_y)]
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

                for z in 0..=height {
                    blocks[x][y][z] = Some(match (chunk_x + chunk_y).rem_euclid(5) {
                        0 => BlockType::Grass,
                        1 => BlockType::Snow,
                        2 => BlockType::Dirt,
                        3 => BlockType::Sand,
                        4 => BlockType::Stone,
                        _ => unreachable!(),
                    })
                }
            }
        }

        blocks
    }

    pub fn generate_chunk_mesh(&mut self, chunk_x: i32, chunk_y: i32) -> (Vec<Vertex>, Vec<u16>) {
        // Load the target chunk and its 4 cardinal neighbors
        self.get_chunk(chunk_x, chunk_y);
        self.get_chunk(chunk_x, chunk_y + 1); // North
        self.get_chunk(chunk_x, chunk_y - 1); // South
        self.get_chunk(chunk_x + 1, chunk_y); // East
        self.get_chunk(chunk_x - 1, chunk_y); // West

        let chunk = self.get_chunk_if_loaded(chunk_x, chunk_y).unwrap();

        let adjacent = AdjacentChunks {
            north: self.get_chunk_if_loaded(chunk_x, chunk_y + 1),
            south: self.get_chunk_if_loaded(chunk_x, chunk_y - 1),
            east: self.get_chunk_if_loaded(chunk_x + 1, chunk_y),
            west: self.get_chunk_if_loaded(chunk_x - 1, chunk_y),
        };

        chunk.generate_mesh(&adjacent)
    }
}
