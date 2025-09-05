use {
    crate::{
        block::BlockType,
        chunk::{Chunk, CHUNK_HEIGHT, CHUNK_WIDTH},
        face::{Face, FACES},
        noise::{PerlinNoise, PerlinNoiseBuilder},
        vertex::Vertex,
    },
    std::collections::HashMap,
};

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
            height_scale: CHUNK_HEIGHT as f64 * 0.4, // Height variation range
            height_offset: CHUNK_HEIGHT as f64 * 0.4, // Base height
            render_distance: 10,
        }
    }

    pub fn get_render_distance(&self) -> usize {
        self.render_distance
    }

    pub fn get_chunk_if_loaded(&self, chunk_x: i32, chunk_y: i32) -> Option<&Chunk> {
        if !self.chunks.contains_key(&(chunk_x, chunk_y)) {
            return None;
        }

        Some(&self.chunks[&(chunk_x, chunk_y)])
    }

    pub fn get_chunk(&mut self, chunk_x: i32, chunk_y: i32) -> &Chunk {
        if !self.chunks.contains_key(&(chunk_x, chunk_y)) {
            let blocks = self.generate_chunk_blocks(chunk_x, chunk_y);
            let chunk = Chunk::new(blocks);
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

                for z in 0..CHUNK_HEIGHT {
                    if z <= height {
                        blocks[x][y][z] = Some(BlockType::Grass)
                    }
                }
            }
        }

        blocks
    }

    fn get_block_at(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<BlockType> {
        if world_z < 0 || world_z >= CHUNK_HEIGHT as i32 {
            return None;
        }

        let chunk_x = (world_x as f32 / CHUNK_WIDTH as f32).floor() as i32;
        let chunk_y = (world_y as f32 / CHUNK_WIDTH as f32).floor() as i32;

        let local_x = world_x - chunk_x * CHUNK_WIDTH as i32;
        let local_y = world_y - chunk_y * CHUNK_WIDTH as i32;

        if local_x < 0
            || local_x >= CHUNK_WIDTH as i32
            || local_y < 0
            || local_y >= CHUNK_WIDTH as i32
        {
            return None;
        }

        if let Some(chunk) = self.get_chunk_if_loaded(chunk_x, chunk_y) {
            return chunk.get_block(local_x as usize, local_y as usize, world_z as usize);
        }

        None
    }

    fn create_face(face: Face, position: glam::Vec3, block: BlockType) -> ([Vertex; 4], [u16; 6]) {
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

    pub fn generate_chunk_mesh(&mut self, chunk_x: i32, chunk_y: i32) -> (Vec<Vertex>, Vec<u16>) {
        for dx in -1..=1 {
            for dy in -1..=1 {
                let neighbor_x = chunk_x + dx;
                let neighbor_y = chunk_y + dy;
                self.get_chunk(neighbor_x, neighbor_y);
            }
        }
        
        let chunk = self.get_chunk_if_loaded(chunk_x, chunk_y).unwrap();
        
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0;

        let world_offset_x = chunk_x * CHUNK_WIDTH as i32;
        let world_offset_y = chunk_y * CHUNK_WIDTH as i32;

        for local_x in 0..CHUNK_WIDTH {
            for local_y in 0..CHUNK_WIDTH {
                for local_z in 0..CHUNK_HEIGHT {
                    let Some(block) = chunk.get_block(local_x, local_y, local_z) else {
                        continue;
                    };

                    let position = glam::Vec3::new(local_x as f32, local_y as f32, local_z as f32);

                    let world_x = world_offset_x + local_x as i32;
                    let world_y = world_offset_y + local_y as i32;
                    let world_z = local_z as i32;

                    for face in FACES {
                        let (dx, dy, dz) = face.normal();
                        let neighbor_world_x = world_x + dx;
                        let neighbor_world_y = world_y + dy;
                        let neighbor_world_z = world_z + dz;

                        let is_face_visible = neighbor_world_z >= 0
                            && self
                                .get_block_at(neighbor_world_x, neighbor_world_y, neighbor_world_z)
                                .is_none();

                        if is_face_visible {
                            let (face_verts, face_indices) =
                                Self::create_face(face, position, block);

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
