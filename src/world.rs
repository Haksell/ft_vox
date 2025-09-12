use {
    crate::{
        biome::BiomeType,
        block::BlockType,
        camera::Camera,
        chunk::{AdjacentChunks, Chunk, CHUNK_HEIGHT, CHUNK_WIDTH},
        coords::{camera_to_world_coords, split_coords, BlockCoords, ChunkCoords, WorldCoords},
        noise::{SimplexNoise, SimplexNoiseInfo},
        spline::{Spline, SplinePoint},
        utils::{ceil_div, lerp, sign},
        vertex::Vertex,
    },
    glam::Vec3,
    std::{
        collections::{HashMap, HashSet},
        thread,
    },
};

pub const SURFACE: usize = 64;
pub const SEA: usize = 62;
pub const MAGMA_CORE: usize = 31;

pub const MAX_DELETE_DISTANCE: f32 = 48.0;

pub struct NoiseValues {
    temperature: f32,
    humidity: f32,
    continentalness: f32,
    erosion: f32,
    weirdness: f32,
    pv: f32,
}

pub struct World {
    temperature_noise: SimplexNoise,
    humidity_noise: SimplexNoise,
    continentalness_noise: SimplexNoise,
    erosion_noise: SimplexNoise,
    weirdness_noise: SimplexNoise,

    cave_low_noise: SimplexNoise,
    cave_high_noise: SimplexNoise,

    chunks: HashMap<ChunkCoords, Chunk>,
    deleted_blocks: HashMap<ChunkCoords, HashSet<BlockCoords>>,
}
impl World {
    pub fn new(seed: u64) -> Self {
        // temperature: affects hot vs cold biomes
        let temperature_noise = SimplexNoise::new(
            seed.wrapping_add(0xFF446677),
            SimplexNoiseInfo {
                frequency: 0.000336,
                octaves: 2,
                ..Default::default()
            },
        );

        // humidity: affects dry vs wet biomes
        let humidity_noise = SimplexNoise::new(
            seed.wrapping_add(0xAABB33CC),
            SimplexNoiseInfo {
                frequency: 0.000246,
                octaves: 2,
                persistence: 0.6,
                ..Default::default()
            },
        );

        // continentalness: determines land vs ocean
        let continentalness_noise = SimplexNoise::new(
            seed.wrapping_add(0xFF000055),
            SimplexNoiseInfo {
                frequency: 0.000974,
                octaves: 6,
                persistence: 0.8,
                lacunarity: 1.2,
            },
        );

        // erosion: affects terrain ruggedness
        let erosion_noise = SimplexNoise::new(
            seed.wrapping_add(0x44336699),
            SimplexNoiseInfo {
                frequency: 0.00998,
                octaves: 6,
                persistence: 0.42,
                ..Default::default()
            },
        );

        // weirdness: creates unusual terrain features
        let weirdness_noise = SimplexNoise::new(
            seed.wrapping_add(0xFF110077),
            SimplexNoiseInfo {
                frequency: 0.00196,
                octaves: 6,
                persistence: 0.66,
                ..Default::default()
            },
        );

        // cave lower bound
        let cave_low_noise = SimplexNoise::new(
            seed.wrapping_add(0x1F326321),
            SimplexNoiseInfo {
                frequency: 0.007,
                octaves: 6,
                persistence: 0.6,
                lacunarity: 2.0,
            },
        );

        // cave upper bound
        let cave_high_noise = SimplexNoise::new(
            seed.wrapping_add(0x15444555),
            SimplexNoiseInfo {
                frequency: 0.007,
                octaves: 6,
                persistence: 0.6,
                lacunarity: 2.0,
            },
        );

        Self {
            temperature_noise,
            humidity_noise,
            continentalness_noise,
            erosion_noise,
            weirdness_noise,
            cave_low_noise,
            cave_high_noise,
            chunks: HashMap::new(),
            deleted_blocks: HashMap::new(),
        }
    }

    pub fn get_chunk_if_loaded(&self, chunk_coords: ChunkCoords) -> Option<&Chunk> {
        self.chunks.get(&chunk_coords)
    }

    pub fn get_mut_chunk_if_loaded(&mut self, chunk_coords: ChunkCoords) -> Option<&mut Chunk> {
        self.chunks.get_mut(&chunk_coords)
    }

    pub fn load_chunk(&mut self, chunk_coords: ChunkCoords) {
        if !self.chunks.contains_key(&chunk_coords) {
            let mut blocks = self.generate_chunk_blocks(chunk_coords);
            if let Some(deleted) = self.deleted_blocks.get(&chunk_coords) {
                for &(x, y, z) in deleted {
                    blocks[x][y][z] = None;
                }
            }
            let chunk = Chunk::new(chunk_coords, blocks);
            self.chunks.insert(chunk_coords, chunk);
        }
    }

    pub fn retain_chunks(&mut self, chunks_to_keep: &HashSet<(i32, i32)>) {
        self.chunks
            .retain(|&coords, _| chunks_to_keep.contains(&coords));
    }

    fn generate_height_at(&self, values: &NoiseValues) -> f32 {
        let continentalness_offset = self.continentalness_spline(values.continentalness);
        let pv_offset = self.peaks_valleys_spline(values.pv);
        let erosion_factor = if values.continentalness < -0.2 {
            self.erosion_factor(values.erosion)
        } else {
            1.0
        };

        SURFACE as f32 + continentalness_offset + pv_offset * erosion_factor
    }

    // Continentalness spline: higher continentalness = higher terrain
    fn continentalness_spline(&self, continentalness: f32) -> f32 {
        let spline = Spline::new(vec![
            SplinePoint::new(-1.0, -40.0),
            SplinePoint::new(-0.45, -20.0),
            SplinePoint::new(-0.2, -2.0),
            SplinePoint::new(-0.1, -1.0),
            SplinePoint::new(0.15, 2.0),
            SplinePoint::new(0.3, 8.0),
            SplinePoint::new(0.5, 10.0),
            SplinePoint::new(0.7, 18.0),
            SplinePoint::new(0.8, 20.0),
            SplinePoint::new(1.0, 30.0),
        ]);

        spline.sample(continentalness)
    }

    // Erosion spline: higher erosion = lower, flatter terrain
    fn erosion_factor(&self, erosion: f32) -> f32 {
        let spline = Spline::new(vec![
            SplinePoint::new(-1.0, 1.0),
            SplinePoint::new(-0.8, 0.9),
            SplinePoint::new(-0.38, 0.8),
            SplinePoint::new(-0.22, 0.6),
            SplinePoint::new(0.05, 0.5),
            SplinePoint::new(0.45, 0.4),
            SplinePoint::new(0.9, 0.2),
            SplinePoint::new(1.0, 0.1),
        ]);

        spline.sample(erosion)
    }

    // Peaks and valleys spline
    fn peaks_valleys_spline(&self, peak_and_valley: f32) -> f32 {
        let spline = Spline::new(vec![
            SplinePoint::new(-1.0, -30.0),
            SplinePoint::new(-0.9, 0.0),
            SplinePoint::new(-0.2, 2.0),
            SplinePoint::new(0.2, 10.0),
            SplinePoint::new(0.6, 30.0),
            SplinePoint::new(0.9, 60.0),
            SplinePoint::new(1.0, 60.0),
        ]);

        spline.sample(peak_and_valley)
    }

    pub fn determine_biome(&self, values: &NoiseValues) -> BiomeType {
        #[rustfmt::skip]
        let temperature_level = match values.temperature {
            x if x >= -1.0  && x < -0.45 => 0,
            x if x >= -0.45 && x < -0.15 => 1,
            x if x >= -0.15 && x <  0.2  => 2,
            x if x >=  0.2  && x <  0.55 => 3,
            x if x >=  0.55 && x <  1.0  => 4,
            _ => 4,
        };

        #[rustfmt::skip]
        let humidity_level = match values.humidity {
            x if x >= -1.0  && x < -0.35 => 0,
            x if x >= -0.35 && x < -0.1  => 1,
            x if x >= -0.1  && x <  0.1  => 2,
            x if x >=  0.1  && x <  0.3  => 3,
            x if x >=  0.3  && x <  1.0  => 4,
            _ => 4,
        };

        #[rustfmt::skip]
        let continentalness_level = match values.continentalness {
            x if x >= -1.0  && x < -0.45 => 0,
            x if x >= -0.45 && x < -0.2  => 1,
            x if x >= -0.2  && x < -0.1  => 2,
            x if x >= -0.1  && x <  0.05 => 3,
            x if x >=  0.05 && x <  0.3  => 4,
            x if x >=  0.3  && x <  1.0  => 5,
            _ => 5,
        };

        #[rustfmt::skip]
        let erosion_level = match values.erosion {
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
        let pv_level = match values.pv {
            x if x >= -1.0  && x < -0.85 => 0,
            x if x >= -0.85 && x < -0.2  => 1,
            x if x >= -0.2  && x <  0.2  => 2,
            x if x >=  0.2  && x <  0.7  => 3,
            x if x >=  0.7  && x <  1.0  => 4,
            _ => 6,
        };

        let weirdness_level = (values.weirdness >= 0.0) as i32;

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
            (0..=1, 1) => BiomeType::ErodedBadlands,
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
            (4, 0..=1, 1) => BiomeType::ErodedBadlands,
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

    fn get_noise_values(&self, world_x: i32, world_y: i32) -> NoiseValues {
        let continentalness = self
            .continentalness_noise
            .noise2d(world_x as f32, world_y as f32);
        let erosion = self.erosion_noise.noise2d(world_x as f32, world_y as f32);
        let weirdness = self.weirdness_noise.noise2d(world_x as f32, world_y as f32);
        let temperature = self
            .temperature_noise
            .noise2d(world_x as f32, world_y as f32);
        let humidity = self.humidity_noise.noise2d(world_x as f32, world_y as f32);
        let pv = 1.0 - (3.0 * weirdness.abs() - 2.0).abs();

        NoiseValues {
            temperature,
            humidity,
            continentalness,
            erosion,
            weirdness,
            pv,
        }
    }

    fn generate_chunk_blocks(
        &self,
        (chunk_x, chunk_y): ChunkCoords,
    ) -> [[[Option<BlockType>; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH] {
        let mut blocks = [[[None; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH];

        let workers = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .min(CHUNK_WIDTH);
        let chunk_size = ceil_div(CHUNK_WIDTH, workers);

        thread::scope(|s| {
            let mut remainder: &mut [[[Option<BlockType>; CHUNK_HEIGHT]; CHUNK_WIDTH]] =
                &mut blocks;
            let mut start_x = 0;

            for _ in 0..workers {
                let len = (CHUNK_WIDTH - start_x).min(chunk_size);
                let (head, tail) = remainder.split_at_mut(len);
                let start_x_this = start_x;

                s.spawn(move || {
                    for (dx, plane) in head.iter_mut().enumerate() {
                        let x = start_x_this + dx;
                        let world_x = (chunk_x * CHUNK_WIDTH as i32) + x as i32;

                        for (y, column) in plane.iter_mut().enumerate() {
                            let world_y = (chunk_y * CHUNK_WIDTH as i32) + y as i32;
                            let noise_values = self.get_noise_values(world_x, world_y);
                            let height = self.generate_height_at(&noise_values) as usize;
                            let biome = self.determine_biome(&noise_values);

                            // DON'T CHANGE UNTIL FT_VOX PUSH
                            let cave_low =
                                self.cave_low_noise.noise2d(world_x as f32, world_y as f32) * 20.0
                                    + 110.0
                                    - height as f32 * 0.6;
                            let cave_high =
                                self.cave_high_noise.noise2d(world_x as f32, world_y as f32) * 25.0
                                    + lerp(56.0, height as f32, 0.3);

                            for z in 0..CHUNK_HEIGHT {
                                if z <= MAGMA_CORE {
                                    column[z] = Some(BlockType::RedSand); // TODO: Magma
                                } else if !biome.is_ocean()
                                    && cave_low < z as f32
                                    && (z as f32) < cave_high
                                {
                                    column[z] = None;
                                } else if z <= height {
                                    let depth_from_surface = height.saturating_sub(z);
                                    column[z] = Some(match depth_from_surface {
                                        0..5 => biome.get_surface_block(),
                                        _ => biome.get_deep_block(),
                                    });
                                } else if z <= SEA {
                                    column[z] = Some(BlockType::Water);
                                }
                            }
                        }
                    }
                });

                remainder = tail;
                start_x += len;
            }
        });

        blocks
    }

    pub fn generate_chunk_mesh(
        &mut self,
        (chunk_x, chunk_y): ChunkCoords,
    ) -> (Vec<Vertex>, Vec<u16>) {
        self.load_chunk((chunk_x, chunk_y));
        self.load_chunk((chunk_x, chunk_y + 1));
        self.load_chunk((chunk_x, chunk_y - 1));
        self.load_chunk((chunk_x + 1, chunk_y));
        self.load_chunk((chunk_x - 1, chunk_y));

        let chunk = self.get_chunk_if_loaded((chunk_x, chunk_y)).unwrap();

        let adjacent = AdjacentChunks {
            north: self.get_chunk_if_loaded((chunk_x, chunk_y + 1)),
            south: self.get_chunk_if_loaded((chunk_x, chunk_y - 1)),
            east: self.get_chunk_if_loaded((chunk_x + 1, chunk_y)),
            west: self.get_chunk_if_loaded((chunk_x - 1, chunk_y)),
        };

        chunk.generate_mesh(&adjacent)
    }

    pub fn delete_center_block(&mut self, camera: &Camera) -> Option<(WorldCoords, BlockType)> {
        let (_, world_coords, block) =
            self.find_block_in_dir(camera.position(), camera.direction(), MAX_DELETE_DISTANCE)?;
        self.delete_block(world_coords);
        Some((world_coords, block))
    }

    // TODO: update DDA to use the tree structure of Chunk
    pub fn find_block_in_dir(
        &self,
        pos: Vec3,
        dir: Vec3,
        max_distance: f32,
    ) -> Option<(f32, WorldCoords, BlockType)> {
        let start = pos;

        let (mut ix, mut iy, mut iz) = camera_to_world_coords(start);

        if self.get_block((ix, iy, iz)).is_some() {
            return None;
        }

        let step_x = sign(dir.x);
        let step_y = sign(dir.y);
        let step_z = sign(dir.z);

        let next_boundary = |i: i32, d: f32| -> f32 { (i + (d > 0.0) as i32) as f32 };

        let init_t_max = |i: i32, s: f32, d: f32| -> f32 {
            if step_x != 0 {
                (next_boundary(i, d) - s) / d
            } else {
                f32::INFINITY
            }
        };

        let mut t_max_x = init_t_max(ix, start.x, dir.x);
        let mut t_max_y = init_t_max(iy, start.y, dir.y);
        let mut t_max_z = init_t_max(iz, start.z, dir.z);

        let init_t_delta = |step: i32, d: f32| -> f32 {
            if step != 0 {
                (1.0 / d).abs()
            } else {
                f32::INFINITY
            }
        };

        let t_delta_x = init_t_delta(step_x, dir.x);
        let t_delta_y = init_t_delta(step_y, dir.y);
        let t_delta_z = init_t_delta(step_z, dir.z);

        let mut t = 0.0;

        while t <= max_distance {
            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    ix += step_x;
                    t = t_max_x;
                    t_max_x += t_delta_x;
                } else {
                    iz += step_z;
                    t = t_max_z;
                    t_max_z += t_delta_z;
                }
            } else {
                if t_max_y < t_max_z {
                    iy += step_y;
                    t = t_max_y;
                    t_max_y += t_delta_y;
                } else {
                    iz += step_z;
                    t = t_max_z;
                    t_max_z += t_delta_z;
                }
            }

            if iz < 0 || iz >= CHUNK_HEIGHT as i32 {
                return None;
            }

            let world_coords = (ix, iy, iz);
            if let Some(block) = self.get_block(world_coords) {
                return Some((t, world_coords, block));
            }
        }

        None
    }

    pub fn get_block(&self, world_coords: WorldCoords) -> Option<BlockType> {
        let (chunk_coords, block_coords) = split_coords(world_coords)?;
        let chunk = self.get_chunk_if_loaded(chunk_coords)?;
        chunk.get_block(block_coords)
    }

    fn delete_block(&mut self, world_coords: WorldCoords) {
        let Some((chunk_coords, block_coords)) = split_coords(world_coords) else {
            return;
        };

        let Some(chunk) = self.get_mut_chunk_if_loaded(chunk_coords) else {
            return;
        };

        chunk.delete_block(block_coords);
        self.deleted_blocks
            .entry(chunk_coords)
            .or_insert_with(HashSet::new)
            .insert(block_coords);
    }
}
