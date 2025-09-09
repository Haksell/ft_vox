use {
    crate::{
        biome::BiomeType,
        block::BlockType,
        chunk::{AdjacentChunks, Chunk, ChunkCoords, CHUNK_HEIGHT, CHUNK_WIDTH},
        noise::{PerlinNoise, PerlinNoiseBuilder},
        utils::lerp,
        vertex::Vertex,
    },
    rayon::prelude::*,
    std::collections::HashMap,
};

pub const RENDER_DISTANCE: usize = 15;

pub const SURFACE: usize = 64;
pub const SEA: usize = 62;

pub struct World {
    temperature_noise: PerlinNoise,
    humidity_noise: PerlinNoise,
    continentalness_noise: PerlinNoise,
    erosion_noise: PerlinNoise,
    weirdness_noise: PerlinNoise,
    cave_noise: PerlinNoise,

    chunks: HashMap<ChunkCoords, Chunk>,
}

impl World {
    pub fn new(seed: u64) -> Self {
        let chunks = HashMap::new();

        // temperature: affects hot vs cold biomes
        let temperature_noise = PerlinNoiseBuilder::new(seed.wrapping_add(0xFF446677))
            .frequency(0.0002)
            .octaves(4)
            .build();

        // humidity: affects dry vs wet biomes
        let humidity_noise = PerlinNoiseBuilder::new(seed.wrapping_add(0xAABB33CC))
            .frequency(0.0026)
            .octaves(4)
            .build();

        // continentalness: determines land vs ocean
        let continentalness_noise = PerlinNoiseBuilder::new(seed.wrapping_add(0xFF000055))
            .frequency(0.0002)
            .octaves(16)
            .build();

        // erosion: affects terrain ruggedness
        let erosion_noise = PerlinNoiseBuilder::new(seed.wrapping_add(0x44336699))
            .frequency(0.002)
            .octaves(8)
            .build();

        // weirdness: creates unusual terrain features
        let weirdness_noise = PerlinNoiseBuilder::new(seed.wrapping_add(0xFF110077))
            .frequency(0.0008)
            .octaves(4)
            .build();

        let cave_noise = PerlinNoiseBuilder::new(seed.wrapping_add(0x110099FF))
            .frequency(0.008)
            .octaves(6)
            .build();

        Self {
            temperature_noise,
            humidity_noise,
            continentalness_noise,
            erosion_noise,
            weirdness_noise,
            cave_noise,
            chunks,
        }
    }

    pub fn get_chunk_index_from_position(&self, world_x: f32, world_y: f32) -> ChunkCoords {
        let chunk_x = (world_x / CHUNK_WIDTH as f32).floor() as i32;
        let chunk_y = (world_y / CHUNK_WIDTH as f32).floor() as i32;

        (chunk_x, chunk_y)
    }

    pub fn get_chunk_if_loaded(&self, chunk_coords: ChunkCoords) -> Option<&Chunk> {
        self.chunks.get(&chunk_coords)
    }

    pub fn get_chunk(&mut self, chunk_coords: ChunkCoords) -> &Chunk {
        if !self.chunks.contains_key(&chunk_coords) {
            let blocks = self.generate_chunk_blocks(chunk_coords);
            let chunk = Chunk::new(chunk_coords, blocks);
            self.chunks.insert(chunk_coords, chunk);
        }

        &self.chunks[&chunk_coords]
    }

    fn generate_height_at(&self, world_x: f32, world_y: f32) -> f32 {
        let continentalness = self.continentalness_noise.noise2d(world_x, world_y);
        let erosion = self.erosion_noise.noise2d(world_x, world_y);
        let weirdness = self.weirdness_noise.noise2d(world_x, world_y);
        let peak_and_valley = 1.0 - (3.0 * weirdness.abs() - 2.0).abs();

        let continentalness_offset = self.continentalness_spline(continentalness);
        let pv_offset = self.peaks_valleys_spline(peak_and_valley);
        let erosion_factor = if continentalness >= -0.2 {
            self.erosion_factor(erosion)
        } else {
            1.0
        };

        let height_offset = continentalness_offset * erosion_factor + pv_offset;

        SURFACE as f32 + height_offset
    }

    // Continentalness spline: higher continentalness = higher terrain
    fn continentalness_spline(&self, continentalness: f32) -> f32 {
        match continentalness {
            x if x < -0.45 => lerp(-30.0, -20.0, (x + 1.0) / ((-0.45) - (-1.0))),
            x if x < -0.2 => lerp(-20.0, -5.0, (x - (-0.45)) / ((-0.2) - (-0.45))),
            x if x < -0.1 => lerp(-5.0, 5.0, (x - (-0.2)) / ((-0.1) - (-0.2))),
            x if x < 0.05 => lerp(5.0, 30.0, (x - (-0.1)) / (0.05 - (-0.1))),
            x if x < 0.3 => lerp(30.0, 60.0, (x - 0.05) / (0.3 - 0.05)),
            x => lerp(60.0, 120.0, (x - 0.3) / (1.0 - 0.3)),
        }
    }

    // Erosion spline: higher erosion = lower, flatter terrain
    fn erosion_factor(&self, erosion: f32) -> f32 {
        match erosion {
            x if x < -0.8 => 1.0,
            x if x < -0.38 => lerp(1.0, 0.9, (x - (-0.8)) / ((-0.38) - (-0.8))),
            x if x < -0.22 => lerp(0.9, 0.7, (x - (-0.38)) / ((-0.22) - (-0.38))),
            x if x < 0.05 => lerp(0.7, 0.5, (x - (-0.22)) / (0.05 - (-0.22))),
            x if x < 0.45 => lerp(0.5, 0.4, (x - 0.05) / (0.45 - 0.05)),
            x => lerp(0.4, 0.3, (x - 0.45) / (1.0 - 0.45)),
        }
    }

    // Peaks and valleys spline
    fn peaks_valleys_spline(&self, peak_and_valley: f32) -> f32 {
        match peak_and_valley {
            x if x < -0.85 => lerp(-40.0, -10.0, (x + 1.0) / ((-0.85) - (-1.0))),
            x if x < -0.2 => lerp(-10.0, 10.0, (x - (-0.85)) / ((-0.2) - (-0.85))),
            x if x < 0.2 => lerp(10.0, 20.0, (x - (-0.2)) / (0.2 - (-0.2))),
            x if x < 0.7 => lerp(20.0, 40.0, (x - 0.2) / (0.7 - 0.2)),
            x => lerp(40.0, 60.0, (x - 0.7) / (1.0 - 0.7)),
        }
    }

    pub fn determine_biome(&self, world_x: f32, world_y: f32) -> BiomeType {
        let temperature = self.temperature_noise.noise2d(world_x, world_y);
        let humidity = self.humidity_noise.noise2d(world_x, world_y);
        let continentalness = self.continentalness_noise.noise2d(world_x, world_y);
        let erosion = self.erosion_noise.noise2d(world_x, world_y);
        let weirdness = self.weirdness_noise.noise2d(world_x, world_y);
        let peak_and_valley = 1.0 - (3.0 * weirdness.abs() - 2.0).abs();

        #[rustfmt::skip]
        let temperature_level = match temperature {
            x if x >= -1.0  && x < -0.45 => 0,
            x if x >= -0.45 && x < -0.15 => 1,
            x if x >= -0.15 && x <  0.2  => 2,
            x if x >=  0.2  && x <  0.55 => 3,
            x if x >=  0.55 && x <  1.0  => 4,
            _ => 4,
        };

        #[rustfmt::skip]
        let humidity_level = match humidity {
            x if x >= -1.0  && x < -0.35 => 0,
            x if x >= -0.35 && x < -0.1  => 1,
            x if x >= -0.1  && x <  0.1  => 2,
            x if x >=  0.1  && x <  0.3  => 3,
            x if x >=  0.3  && x <  1.0  => 4,
            _ => 4,
        };

        #[rustfmt::skip]
        let continentalness_level = match continentalness {
            x if x >= -1.0  && x < -0.45 => 0,
            x if x >= -0.45 && x < -0.2  => 1,
            x if x >= -0.2  && x < -0.1  => 2,
            x if x >= -0.1  && x <  0.05 => 3,
            x if x >=  0.05 && x <  0.3  => 4,
            x if x >=  0.3  && x <  1.0  => 5,
            _ => 5,
        };

        #[rustfmt::skip]
        let erosion_level = match erosion {
            x if x >= -1.0  && x < -0.8  => 0,
            x if x >= -0.8  && x < -0.38 => 1,
            x if x >= -0.38 && x < -0.22 => 2,
            x if x >= -0.22 && x <  0.05 => 3,
            x if x >=  0.05 && x <  0.45 => 4,
            x if x >=  0.45 && x <  0.55 => 5,
            x if x >=  0.55 && x <  1.0  => 6,
            _ => 6,
        };

        #[rustfmt::skip]
        let pv_level = match peak_and_valley {
            x if x >= -1.0  && x < -0.85 => 0,
            x if x >= -0.85 && x < -0.2  => 1,
            x if x >= -0.2  && x <  0.2  => 2,
            x if x >=  0.2  && x <  0.7  => 3,
            x if x >=  0.7  && x <  1.0  => 4,
            _ => 6,
        };

        let weirdness_level = (weirdness >= 0.0) as i32;

        match (
            continentalness_level,
            temperature_level,
            humidity_level,
            erosion_level,
            pv_level,
            weirdness_level,
        ) {
            // Ocean biomes
            (0, 0, _, _, _, _) => BiomeType::FrozenOcean,
            (0, 1, _, _, _, _) => BiomeType::ColdOcean,
            (0, 2..=3, _, _, _, _) => BiomeType::Ocean,
            (1, 0, _, _, _, _) => BiomeType::DeepFrozenOcean,
            (1, 1, _, _, _, _) => BiomeType::DeepColdOcean,
            (1, 2..=3, _, _, _, _) => BiomeType::DeepOcean,
            (0..=1, 4, _, _, _, _) => BiomeType::WarmOcean,

            // Valleys Biomes
            (2, 0, _, _, 0, _) => BiomeType::FrozenRiver,
            (2, _, _, _, 0, _) => BiomeType::River,
            (3, 0, _, 0..=5, 0, _) => BiomeType::FrozenRiver,
            (3, _, _, 0..=5, 0, _) => BiomeType::River,
            (3..=5, 0, _, 6, 0, _) => BiomeType::FrozenRiver,
            (3..=5, 1..=2, _, 6, 0, _) => BiomeType::Swamp,
            (3..=5, 3..=4, _, 6, 0, _) => BiomeType::Mangrove,
            (4..=5, 0..=3, _, 0..=5, 0, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (4..=5, 4, _, 0..=5, 0, _) => {
                self.determine_badlands_biome(humidity_level, weirdness_level)
            }

            // Low Biomes
            (2, _, _, 0..=2, 1, _) => BiomeType::StonyShore,
            (2, _, _, 3..=4, 1, _) => self.determine_beach_biome(temperature_level),
            (2, _, _, 5, 1, 0) => self.determine_beach_biome(temperature_level),
            (2, 0..=1, _, 5, 1, 1) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2, _, 4, 5, 1, 1) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2, 2..=4, 0..=3, 5, 1, 1) => BiomeType::WindsweptSavanna,
            (2, _, _, 6, 1, _) => self.determine_beach_biome(temperature_level),

            (3, 0..=3, _, 0..=1, 1, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3, 4, _, 0..=1, 1, _) => {
                self.determine_badlands_biome(humidity_level, weirdness_level)
            }
            (3, _, _, 2..=4, 1, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3, 0..=1, _, 5, 1, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3, _, 4, 5, 1, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3, _, _, 5, 1, 0) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3, 2..=4, 0..=3, 5, 1, 1) => BiomeType::WindsweptSavanna,
            (3..=5, 0, _, 6, 1, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3..=5, 1..=2, _, 6, 1, _) => BiomeType::Swamp,
            (3..=5, 3..=4, _, 6, 1, _) => BiomeType::Mangrove,
            (4..=5, 0, 0..=1, 0..=1, 1, _) => BiomeType::SnowySlopes,
            (4..=5, 0, 2..=4, 0..=1, 1, _) => BiomeType::Grove,
            (4..=5, 1..=3, _, 0..=1, 1, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (4..=5, 4, _, 0..=1, 1, _) => {
                self.determine_badlands_biome(humidity_level, weirdness_level)
            }
            (4..=5, 0..=3, _, 2..=3, 1, _) => {
                self.determine_badlands_biome(humidity_level, weirdness_level)
            }
            (4..=5, 4, _, 2..=3, 1, _) => {
                self.determine_badlands_biome(humidity_level, weirdness_level)
            }
            (4..=5, _, _, 4..=5, 1, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }

            // Mid Biomes
            (2, _, _, 0..=2, 2, _) => BiomeType::StonyShore,
            (2, _, _, 3, 2, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2, _, _, 4, 2, 0) => self.determine_beach_biome(temperature_level),
            (2, _, _, 4, 2, 1) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2, _, _, 5, 2, 0) => self.determine_beach_biome(temperature_level),
            (2, 0..=1, _, 5, 2, 1) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2, _, 4, 5, 2, 1) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2, 2..=4, 0..=3, 5, 2, 1) => BiomeType::WindsweptSavanna,
            (2, _, _, 6, 2, 0) => self.determine_beach_biome(temperature_level),
            (2, _, _, 6, 2, 1) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3..=5, 0..=2, 0..=1, 0, 2, _) => BiomeType::SnowySlopes,
            (3..=5, 0..=2, 2..=4, 0, 2, _) => BiomeType::Grove,
            (3..=5, 3..=4, _, 0, 2, _) => {
                self.determine_plateau_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3..=4, 0, 0..=1, 1, 2, _) => BiomeType::SnowySlopes,
            (3..=4, 0, 2..=4, 1, 2, _) => BiomeType::Grove,
            (3..=4, 1..=3, _, 1, 2, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3..=4, 4, _, 1, 2, _) => {
                self.determine_badlands_biome(humidity_level, weirdness_level)
            }
            (5, 0, 0..=1, 1, 2, _) => BiomeType::SnowySlopes,
            (5, 0, 2..=4, 1, 2, _) => BiomeType::Grove,
            (5, 1..=4, _, 1, 2, _) => {
                self.determine_plateau_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3, _, _, 2..=4, 2, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (4, 0..=3, _, 2..=3, 2, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (4, 4, _, 2..=3, 2, _) => {
                self.determine_badlands_biome(humidity_level, weirdness_level)
            }
            (5, _, _, 2, 2, _) => {
                self.determine_plateau_biome(temperature_level, humidity_level, weirdness_level)
            }
            (5, 0..=3, _, 3, 2, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (5, 4, _, 3, 2, _) => self.determine_badlands_biome(humidity_level, weirdness_level),
            (4..=5, _, _, 4, 2, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3, 0..=1, _, 5, 2, 0) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3, _, 4, 5, 2, 0) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3, 2..=4, 0..=3, 5, 2, 1) => BiomeType::WindsweptSavanna,
            (4..=5, _, _, 5, 2, _) => {
                self.determine_shattered_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3..=5, 0, _, 6, 2, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3..=5, 1..=2, _, 6, 2, _) => BiomeType::Swamp,
            (3..=5, 3..=4, _, 6, 2, _) => BiomeType::Mangrove,

            //High Biomes
            (2, _, _, 0..=4, 3, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3, 0..=2, 0..=1, 0, 3, _) => BiomeType::SnowySlopes,
            (3, 0..=2, 2..=4, 0, 3, _) => BiomeType::Grove,
            (3, 3..=4, _, 0, 3, _) => {
                self.determine_plateau_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3, 0, 0..=1, 1, 3, _) => BiomeType::SnowySlopes,
            (3, 0, 2..=4, 1, 3, _) => BiomeType::Grove,
            (3, 1..=3, _, 1, 3, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (3, 4, _, 1, 3, _) => self.determine_badlands_biome(humidity_level, weirdness_level),
            (3, _, _, 2..=4, 3, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2..=3, 0..=1, _, 5, 3, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2..=3, _, 4, 5, 3, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2..=3, _, _, 5, 3, 0) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2..=3, 2..=4, 0..=3, 5, 3, 1) => BiomeType::WindsweptSavanna,
            (4..=5, 0..=2, _, 0, 3, 0) => BiomeType::JaggedPeaks,
            (4..=5, 0..=2, _, 0, 3, 1) => BiomeType::FrozenPeaks,
            (4..=5, 3, _, 0, 3, _) => BiomeType::StonyPeaks,
            (4..=5, 4, _, 0, 3, _) => {
                self.determine_badlands_biome(humidity_level, weirdness_level)
            }
            (4..=5, 0..=2, 0..=1, 1, 3, _) => BiomeType::SnowySlopes,
            (4..=5, 0..=2, 2..=4, 1, 3, _) => BiomeType::Grove,
            (4..=5, 3..=4, _, 1, 3, _) => {
                self.determine_plateau_biome(temperature_level, humidity_level, weirdness_level)
            }
            (4, _, _, 2, 3, _) => {
                self.determine_plateau_biome(temperature_level, humidity_level, weirdness_level)
            }
            (4, 0..=3, _, 3, 3, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (4, 4, _, 3, 3, _) => self.determine_badlands_biome(humidity_level, weirdness_level),
            (5, _, _, 2..=3, 3, _) => {
                self.determine_plateau_biome(temperature_level, humidity_level, weirdness_level)
            }
            (5, _, _, 4, 3, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (4..=5, _, _, 5, 3, _) => {
                self.determine_shattered_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2..=5, _, _, 6, 3, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }

            // Peaks Biomes
            (2..=3, 0..=2, _, 0, 4, 0) => BiomeType::JaggedPeaks,
            (2..=3, 0..=2, _, 0, 4, 1) => BiomeType::FrozenPeaks,
            (2..=3, 3, _, 0, 4, _) => BiomeType::StonyPeaks,
            (2..=3, 4, _, 0, 4, _) => {
                self.determine_badlands_biome(humidity_level, weirdness_level)
            }
            (2..=3, 0, 0..=1, 1, 4, _) => BiomeType::SnowySlopes,
            (2..=3, 0, 2..=4, 1, 4, _) => BiomeType::Grove,
            (2..=3, 1..=3, _, 1, 4, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2..=3, 4, _, 1, 4, _) => {
                self.determine_badlands_biome(humidity_level, weirdness_level)
            }
            (2..=3, _, _, 2..=4, 4, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2..=3, _, _, 5, 4, 0) => {
                self.determine_shattered_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2..=3, 0..=1, _, 5, 4, _) => {
                self.determine_shattered_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2..=3, _, 4, 5, 4, _) => {
                self.determine_shattered_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2..=3, 2..=4, 0..=3, 5, 4, 1) => BiomeType::WindsweptSavanna,
            (4..=5, 0..=2, _, 0..=1, 4, 0) => BiomeType::JaggedPeaks,
            (4..=5, 0..=2, _, 0..=1, 4, 1) => BiomeType::FrozenPeaks,
            (4..=5, 3, _, 0..=1, 4, _) => BiomeType::StonyPeaks,
            (4..=5, 4, _, 0..=1, 4, _) => {
                self.determine_badlands_biome(humidity_level, weirdness_level)
            }
            (4, _, _, 2, 4, _) => {
                self.determine_plateau_biome(temperature_level, humidity_level, weirdness_level)
            }
            (4, 0..=3, _, 3..=4, 4, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (4, 4, _, 3..=4, 4, _) => {
                self.determine_badlands_biome(humidity_level, weirdness_level)
            }
            (5, _, _, 2..=4, 4, _) => {
                self.determine_plateau_biome(temperature_level, humidity_level, weirdness_level)
            }
            (4..=5, _, _, 4, 4, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }
            (4..=5, _, _, 5, 4, _) => {
                self.determine_shattered_biome(temperature_level, humidity_level, weirdness_level)
            }
            (2..=5, _, _, 6, 4, _) => {
                self.determine_middle_biome(temperature_level, humidity_level, weirdness_level)
            }

            // Default
            _ => BiomeType::Plains,
        }
    }

    fn determine_beach_biome(&self, temperature_level: i32) -> BiomeType {
        match temperature_level {
            0 => BiomeType::SnowyBeach,
            1..=3 => BiomeType::Beach,
            4 => BiomeType::Desert,
            _ => BiomeType::Beach,
        }
    }

    fn determine_badlands_biome(&self, humidity_level: i32, weirdness_level: i32) -> BiomeType {
        match (humidity_level, weirdness_level) {
            (0..=1, 0) => BiomeType::Badlands,
            (0..=1, 1) => BiomeType::ErrodedBadlands,
            (2, _) => BiomeType::Badlands,
            _ => BiomeType::WoodedBadlands,
        }
    }

    fn determine_middle_biome(
        &self,
        temperature_level: i32,
        humidity_level: i32,
        weirdness_level: i32,
    ) -> BiomeType {
        match (temperature_level, humidity_level, weirdness_level) {
            (0, 0, 0) => BiomeType::SnowyPlains,
            (0, 0, 1) => BiomeType::IceSpikes,
            (0, 1, _) => BiomeType::SnowyPlains,
            (0, 2, 0) => BiomeType::SnowyPlains,
            (0, 2, 1) => BiomeType::SnowyTaiga,
            (0, 3, _) => BiomeType::SnowyTaiga,
            (0, 4, _) => BiomeType::Taiga,
            (1, 0..=1, _) => BiomeType::Plains,
            (1, 3, _) => BiomeType::Taiga,
            (1, 4, 0) => BiomeType::OldGrowthSpruceTaiga,
            (1, 4, 1) => BiomeType::OldGrowthPineTaiga,
            (2, 0, 0) => BiomeType::FlowerForest,
            (2, 0, 1) => BiomeType::SunflowerForest,
            (2, 1, _) => BiomeType::Plains,
            (1..=2, 2, _) => BiomeType::Forest,
            (2, 3, 0) => BiomeType::BirchForest,
            (2, 3, 1) => BiomeType::OldGrowthBirchForest,
            (2, 4, _) => BiomeType::DarkForest,
            (3, 0..=1, _) => BiomeType::Savanna,
            (3, 2, 0) => BiomeType::Forest,
            (3, 2, 1) => BiomeType::Plains,
            (3, 3, 0) => BiomeType::Jungle,
            (3, 3, 1) => BiomeType::SparseJungle,
            (3, 4, 0) => BiomeType::Jungle,
            (3, 4, 1) => BiomeType::BambooJungle,
            (4, _, _) => BiomeType::Desert,

            _ => BiomeType::Plains,
        }
    }

    fn determine_plateau_biome(
        &self,
        temperature_level: i32,
        humidity_level: i32,
        weirdness_level: i32,
    ) -> BiomeType {
        match (temperature_level, humidity_level, weirdness_level) {
            (0, 0, 0) => BiomeType::SnowyPlains,
            (0, 0, 1) => BiomeType::IceSpikes,
            (0, 1..=2, _) => BiomeType::SnowyPlains,
            (0, 3..=4, _) => BiomeType::SnowyTaiga,
            (1, 0, 0) => BiomeType::Meadow,
            (1, 0, 1) => BiomeType::CherryGrove,
            (1, 1, _) => BiomeType::Meadow,
            (1, 2, 0) => BiomeType::Forest,
            (1, 2, 1) => BiomeType::Meadow,
            (1, 3, 0) => BiomeType::Taiga,
            (1, 3, 1) => BiomeType::Meadow,
            (1, 4, 0) => BiomeType::OldGrowthSpruceTaiga,
            (1, 4, 1) => BiomeType::OldGrowthPineTaiga,
            (2, 0..=1, 0) => BiomeType::Meadow,
            (2, 0..=1, 1) => BiomeType::CherryGrove,
            (2, 2, 0) => BiomeType::Meadow,
            (2, 2, 1) => BiomeType::Forest,
            (2, 3, 0) => BiomeType::Meadow,
            (2, 3, 1) => BiomeType::BirchForest,
            (2, 4, _) => BiomeType::PaleGarden,
            (3, 0..=1, _) => BiomeType::SavannaPlateau,
            (3, 2..=3, _) => BiomeType::Forest,
            (3, 4, _) => BiomeType::Jungle,
            (4, 0..=1, 0) => BiomeType::Badlands,
            (4, 0..=1, 1) => BiomeType::ErrodedBadlands,
            (4, 2, _) => BiomeType::Badlands,
            (4, 3..=4, _) => BiomeType::WoodedBadlands,

            _ => BiomeType::Plains,
        }
    }

    fn determine_shattered_biome(
        &self,
        temperature_level: i32,
        humidity_level: i32,
        weirdness_level: i32,
    ) -> BiomeType {
        match (temperature_level, humidity_level, weirdness_level) {
            (0..=1, 0..=1, _) => BiomeType::WindsweptGravellyHills,
            (0..=1, 2, _) => BiomeType::WindsweptHills,
            (2, 0..=2, _) => BiomeType::WindsweptHills,
            (0..=2, 3..=4, _) => BiomeType::WindsweptForest,
            (3, 0..=1, _) => BiomeType::Savanna,
            (3, 2, 0) => BiomeType::Forest,
            (3, 2, 1) => BiomeType::Plains,
            (3, 3, 0) => BiomeType::Jungle,
            (3, 3, 1) => BiomeType::SparseJungle,
            (3, 4, 0) => BiomeType::Jungle,
            (3, 4, 1) => BiomeType::BambooJungle,
            (4, _, _) => BiomeType::Desert,

            _ => BiomeType::Plains,
        }
    }

    fn has_cave_at(&self, world_x: i32, world_y: i32, world_z: i32, surface_height: i32) -> bool {
        // TODO: spaghetti caves

        let noise = self
            .cave_noise
            .noise3d(world_x as f32, world_y as f32, world_z as f32);

        let normalized_z = ((world_z - 8) as f32 / (surface_height) as f32).clamp(0.0, 1.0);
        let probability = 4.0 * normalized_z * (1.0 - normalized_z);

        let spaghetti = noise.abs() < 0.02 * probability;
        let cheese = noise * probability > 0.2;

        cheese
    }

    // fn render_threaded(
    //     world: Arc<World>,
    //     camera: Arc<Camera>,
    //     canvas: Arc<Mutex<Canvas>>,
    //     num_threads: usize,
    // ) {
    //     println!(
    //         "Starting master-worker rendering with {} threads",
    //         num_threads
    //     );

    //     // Create work queue - each work item is a pixel coordinate
    //     let total_pixels = camera.hsize * camera.vsize;
    //     let (work_sender, work_receiver) = mpsc::channel::<(usize, usize)>();
    //     let (result_sender, result_receiver) = mpsc::channel::<(usize, usize, u32)>();

    //     // Share the work receiver among all worker threads
    //     let work_receiver = Arc::new(Mutex::new(work_receiver));

    //     // Spawn worker threads
    //     let mut handles = Vec::new();
    //     for thread_id in 0..num_threads {
    //         let world = Arc::clone(&world);
    //         let camera = Arc::clone(&camera);
    //         let work_receiver = Arc::clone(&work_receiver);
    //         let result_sender = result_sender.clone();

    //         let handle = thread::spawn(move || {
    //             loop {
    //                 // Try to get work from the queue
    //                 let work = {
    //                     let receiver = work_receiver.lock().unwrap();
    //                     receiver.recv()
    //                 };

    //                 match work {
    //                     Ok((x, y)) => {
    //                         // Render this pixel
    //                         let ray = camera.ray_for_pixel(x, y);
    //                         let color = world.color_at(&ray, camera.max_reflections);

    //                         // Send result back to master
    //                         if result_sender.send((x, y, color.to_u32())).is_err() {
    //                             break; // Master thread has finished
    //                         }
    //                     }
    //                     Err(_) => {
    //                         // No more work available
    //                         break;
    //                     }
    //                 }
    //             }
    //             println!("Worker thread {} finished", thread_id);
    //         });

    //         handles.push(handle);
    //     }

    //     // Master thread: distribute work
    //     thread::spawn(move || {
    //         for y in 0..camera.vsize {
    //             for x in 0..camera.hsize {
    //                 if work_sender.send((x, y)).is_err() {
    //                     return; // Workers have shut down
    //                 }
    //             }
    //         }
    //         // Close the work channel to signal no more work
    //         drop(work_sender);
    //     });

    //     // Master thread: collect results and update canvas
    //     let mut pixels_completed = 0;
    //     let progress_interval = total_pixels / 20; // Report progress every 5%

    //     for (x, y, pixel) in result_receiver {
    //         // Write pixel to canvas
    //         {
    //             let mut canvas_lock = canvas.lock().unwrap();
    //             canvas_lock.write_pixel(x, y, pixel);
    //         }

    //         pixels_completed += 1;

    //         // Progress reporting
    //         if pixels_completed % progress_interval == 0 || pixels_completed == total_pixels {
    //             let progress = (pixels_completed as f64 / total_pixels as f64) * 100.0;
    //             println!(
    //                 "Progress: {:.1}% ({}/{} pixels)",
    //                 progress, pixels_completed, total_pixels
    //             );
    //         }

    //         // Break if all pixels are done
    //         if pixels_completed >= total_pixels {
    //             break;
    //         }
    //     }

    //     // Wait for all worker threads to finish
    //     for handle in handles {
    //         handle.join().unwrap();
    //     }

    //     println!("Rendering complete!");
    // }

    fn generate_chunk_blocks(
        &self,
        (chunk_x, chunk_y): ChunkCoords,
    ) -> [[[Option<BlockType>; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH] {
        let mut blocks = [[[None; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH];

        // Coerce the 3D array to a mutable slice so Rayon can par_iter_mut over it.
        (&mut blocks[..])
            .par_iter_mut()
            .enumerate()
            .for_each(|(x, plane)| {
                let world_x = (chunk_x * CHUNK_WIDTH as i32) + x as i32;

                // Fill each column at (x, y) sequentially inside the worker.
                for (y, column) in plane.iter_mut().enumerate() {
                    let world_y = (chunk_y * CHUNK_WIDTH as i32) + y as i32;
                    *column = self.generate_column(world_x, world_y);
                }
            });

        blocks
    }

    // Also fix the z-loop bound to avoid indexing at CHUNK_HEIGHT.
    fn generate_column(&self, world_x: i32, world_y: i32) -> [Option<BlockType>; CHUNK_HEIGHT] {
        let height = self.generate_height_at(world_x as f32, world_y as f32) as usize;
        let biome = self.determine_biome(world_x as f32, world_y as f32);
        let mut column = [None; CHUNK_HEIGHT];

        for z in 0..CHUNK_HEIGHT {
            if z <= height {
                if self.has_cave_at(world_x, world_y, z as i32, height as i32) {
                    continue;
                }
                let depth_from_surface = height.saturating_sub(z);
                column[z] = Some(match depth_from_surface {
                    0..=5 => biome.get_surface_block(),
                    _ => biome.get_deep_block(),
                });
            } else if z <= SEA {
                column[z] = Some(BlockType::Water);
            }
        }
        column
    }

    pub fn generate_chunk_mesh(
        &mut self,
        (chunk_x, chunk_y): ChunkCoords,
    ) -> (Vec<Vertex>, Vec<u16>) {
        // Load the target chunk and its 4 cardinal neighbors
        self.get_chunk((chunk_x, chunk_y));
        self.get_chunk((chunk_x, chunk_y + 1));
        self.get_chunk((chunk_x, chunk_y - 1));
        self.get_chunk((chunk_x + 1, chunk_y));
        self.get_chunk((chunk_x - 1, chunk_y));

        let chunk = self.get_chunk_if_loaded((chunk_x, chunk_y)).unwrap();

        let adjacent = AdjacentChunks {
            north: self.get_chunk_if_loaded((chunk_x, chunk_y + 1)),
            south: self.get_chunk_if_loaded((chunk_x, chunk_y - 1)),
            east: self.get_chunk_if_loaded((chunk_x + 1, chunk_y)),
            west: self.get_chunk_if_loaded((chunk_x - 1, chunk_y)),
        };

        chunk.generate_mesh(&adjacent)
    }
}
