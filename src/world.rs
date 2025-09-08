use {
    crate::{
        utils::lerp,
        biome::BiomeType,
        block::BlockType,
        chunk::{AdjacentChunks, Chunk, CHUNK_HEIGHT, CHUNK_WIDTH},
        noise::{PerlinNoise, PerlinNoiseBuilder},
        vertex::Vertex,
    },
    std::collections::HashMap,
};

pub const RENDER_DISTANCE: usize = 16;

pub struct World {
    temperature_noise: PerlinNoise,
    humidity_noise: PerlinNoise,
    continentalness_noise: PerlinNoise,
    erosion_noise: PerlinNoise,
    weirdness_noise: PerlinNoise,

    chunks: HashMap<(i32, i32), Chunk>,

    surface: f32,
    render_distance: usize,
}

impl World {
    pub fn new(seed: u64) -> Self {
        let chunks = HashMap::new();

        // Temperature: affects hot vs cold biomes
        let temperature_noise = PerlinNoiseBuilder::new(seed.wrapping_add(0xFF446677))
            .frequency(0.0002)
            .octaves(4)
            .build();

        // Humidity: affects dry vs wet biomes
        let humidity_noise = PerlinNoiseBuilder::new(seed.wrapping_add(0xAABB33CC))
            .frequency(0.0003)
            .octaves(4)
            .build();

        // Continentalness: determines land vs ocean
        let continentalness_noise = PerlinNoiseBuilder::new(seed.wrapping_add(0xFF000055))
            .frequency(0.0002)
            .octaves(12)
            .build();

        // Erosion: affects terrain ruggedness
        let erosion_noise = PerlinNoiseBuilder::new(seed.wrapping_add(0x44336699))
            .frequency(0.0004)
            .octaves(6)
            .build();

        // Weirdness: creates unusual terrain features
        let weirdness_noise = PerlinNoiseBuilder::new(seed.wrapping_add(0xFF110077))
            .frequency(0.001)
            .octaves(6)
            .build();

        Self {
            temperature_noise,
            humidity_noise,
            continentalness_noise,
            erosion_noise,
            weirdness_noise,
            chunks,
            surface: CHUNK_HEIGHT as f32 * 0.25,
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

    fn generate_height_at(&self, world_x: f32, world_y: f32) -> f32 {
        let continentalness = self.continentalness_noise.noise2d(world_x, world_y);
        let erosion = self.erosion_noise.noise2d(world_x, world_y);
        let weirdness = self.weirdness_noise.noise2d(world_x, world_y);

        let peak_and_valley = 1.0 - (3.0 * weirdness.abs() - 2.0).abs();

        let continentalness_offset = self.continentalness_spline(continentalness);

        let erosion_offset = self.erosion_spline(erosion);

        let pv_offset = self.peaks_valleys_spline(peak_and_valley, erosion);

        let stretch_factor =
            self.calculate_stretch_factor(continentalness, erosion, peak_and_valley);

        let base_height = self.surface;

        let height_offset = continentalness_offset + erosion_offset + pv_offset;
        let final_height = base_height + (height_offset * stretch_factor);

        final_height.max(0.0)
    }

    // Continentalness spline: higher continentalness = higher terrain
    fn continentalness_spline(&self, continentalness: f32) -> f32 {
        match continentalness {
            x if x < -0.45 => -20.0, // Deep ocean
            x if x < -0.2 => lerp(-20.0, -10.0, (x + 0.45) / 0.25), // Ocean to coast
            x if x < -0.1 => lerp(-10.0, 5.0, (x + 0.2) / 0.1), // Coast to low land
            x if x < 0.05 => lerp(5.0, 15.0, (x + 0.1) / 0.15), // Low to mid land
            x if x < 0.3 => lerp(15.0, 35.0, (x - 0.05) / 0.25), // Mid to high land
            x => lerp(35.0, 50.0, (x - 0.02) / 0.2), // High mountains
        }
    }

    // Erosion spline: higher erosion = lower, flatter terrain
    fn erosion_spline(&self, erosion: f32) -> f32 {
        match erosion {
            x if x < -0.8 => 0.0, // No erosion effect
            x if x < -0.38 => lerp(0.0, -5.0, (x + 0.8) / 0.42),
            x if x < -0.22 => lerp(-5.0, -15.0, (x + 0.38) / 0.16),
            x if x < 0.05 => lerp(-15.0, -25.0, (x + 0.22) / 0.27),
            x if x < 0.45 => lerp(-25.0, -35.0, (x - 0.05) / 0.4),
            x => lerp(-35.0, -40.0, (x - 0.02) / 0.4), // High erosion = very flat
        }
    }

    // Peaks and valleys spline
    fn peaks_valleys_spline(&self, peak_and_valley: f32, erosion: f32) -> f32 {
        // PV effect is stronger in non-eroded areas
        let erosion_factor = 1.0 - erosion.max(-1.0).min(1.0) * 0.5;

        match peak_and_valley {
            x if x < -0.85 => -30.0 * erosion_factor,
            x if x < -0.2 => {
                let t = (x + 0.85) / 0.65;
                lerp(-30.0, -10.0, t) * erosion_factor
            }
            x if x < 0.2 => {
                let t = (x + 0.2) / 0.4;
                lerp(-10.0, 5.0, t) * erosion_factor
            }
            x if x < 0.7 => {
                let t = (x - 0.2) / 0.5;
                lerp(5.0, 25.0, t) * erosion_factor
            }
            x => {
                let t = (x - 0.2) / 0.5;
                lerp(25.0, 50.0, t) * erosion_factor
            }
        }
    }

    // Calculate vertical stretch factor
    fn calculate_stretch_factor(
        &self,
        continentalness: f32,
        erosion: f32,
        peak_and_valley: f32,
    ) -> f32 {
        // Base stretch factor
        let mut stretch = 1.0;

        // Continental areas have more dramatic height variations
        if continentalness > 0.0 {
            stretch *= 1.0 + continentalness * 0.8;
        }

        // Low erosion areas can have more dramatic height changes
        if erosion < 0.0 {
            stretch *= 1.0 + (-erosion) * 0.6;
        }

        // High PV values create more dramatic terrain
        stretch *= 1.0 + peak_and_valley.abs() * 0.4;

        stretch.max(0.3).min(3.0) // Clamp to reasonable range
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
                let height = self.generate_height_at(world_x as f32, world_y as f32) as usize;
                let biome = self.determine_biome(world_x as f32, world_y as f32);

                for z in 0..=height.min(CHUNK_HEIGHT - 1) {
                    let depth_from_surface = height.saturating_sub(z);

                    blocks[x][y][z] = Some(match depth_from_surface {
                        0 => biome.get_surface_block(),
                        1..=3 => biome.get_subsurface_block(),
                        _ => biome.get_deep_block(),
                    });
                }
            }
        }

        blocks
    }

    pub fn generate_chunk_mesh(&mut self, chunk_x: i32, chunk_y: i32) -> (Vec<Vertex>, Vec<u16>) {
        // Load the target chunk and its 4 cardinal neighbors
        self.get_chunk(chunk_x, chunk_y);
        self.get_chunk(chunk_x, chunk_y + 1);
        self.get_chunk(chunk_x, chunk_y - 1);
        self.get_chunk(chunk_x + 1, chunk_y);
        self.get_chunk(chunk_x - 1, chunk_y);

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
