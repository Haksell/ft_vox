use image::{ImageBuffer, Rgb, RgbImage};

pub struct TerrainGenerator {
    noise: PerlinNoise,
    chunk_size: usize,
}

impl TerrainGenerator {
    pub fn new(seed: u64, chunk_size: usize) -> Self {
        TerrainGenerator {
            noise: PerlinNoise::new(seed),
            chunk_size,
        }
    }

    pub fn generate(&self, chunk_x: usize, chunk_z: usize) -> Vec<Vec<f64>> {
        let mut grid = vec![vec![0.0; self.chunk_size]; self.chunk_size];

        for y in 0..self.chunk_size {
            for x in 0..self.chunk_size {
                let world_x = chunk_x as f64 * self.chunk_size as f64 + x as f64;
                let world_y = chunk_z as f64 * self.chunk_size as f64 + y as f64;

                let nx = world_x / 64.0;
                let ny = world_y / 64.0;

                grid[y][x] = self.noise.perlin(nx, ny).clamp(0.0, 1.0);
            }
        }

        grid
    }

    pub fn save_as_image(
        &self,
        chunk_x: usize,
        chunk_z: usize,
        path: &str,
    ) -> Result<(), image::ImageError> {
        let grid = self.generate(chunk_x, chunk_z);

        // Create a new image buffer
        let mut img: RgbImage = ImageBuffer::new(self.chunk_size as u32, self.chunk_size as u32);

        // Fill the image with the noise values
        for y in 0..self.chunk_size {
            for x in 0..self.chunk_size {
                let noise_value = grid[y][x];
                let pixel_value = (noise_value * 255.0) as u8;
                img.put_pixel(
                    x as u32,
                    y as u32,
                    Rgb([pixel_value, pixel_value, pixel_value]),
                );
            }
        }

        // Save the image
        img.save(path)
    }
}

pub struct PerlinNoise {
    permutations: [u8; 512],
}

impl PerlinNoise {
    pub fn new(seed: u64) -> Self {
        let mut permutations = [0u8; 512];
        let mut temp = (0..=255).collect::<Vec<u8>>();

        // Pseudo Random Number Generator
        let mut hash = seed;
        for i in (0..=255).rev() {
            hash = (hash ^ hash.overflowing_shl(13).0) & 0xFFFFFFFFFFFFFFFF;
            hash = (hash ^ hash.overflowing_shr(7).0) & 0xFFFFFFFFFFFFFFFF;
            hash = (hash ^ hash.overflowing_shl(17).0) & 0xFFFFFFFFFFFFFFFF;

            let j = (hash % (i + 1)) as usize;
            temp.swap(i as usize, j);
        }

        // Duplicate for overflow prevention
        for i in 0..512 {
            permutations[i] = temp[i % 256];
        }

        PerlinNoise { permutations }
    }

    fn perlin(&self, x: f64, y: f64) -> f64 {
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;

        let xf = x - xi as f64;
        let yf = y - yi as f64;

        let u = Self::fade(xf);
        let v = Self::fade(yf);

        // Hash coordinates
        let a = self.permutations[(xi & 255) as usize] as i32 + yi;
        let aa = self.permutations[(a & 255) as usize] as usize;
        let ab = self.permutations[((a + 1) & 255) as usize] as usize;

        let b = self.permutations[((xi + 1) & 255) as usize] as i32 + yi;
        let ba = self.permutations[(b & 255) as usize] as usize;
        let bb = self.permutations[((b + 1) & 255) as usize] as usize;

        // Interpolate
        let value = Self::lerp(
            Self::lerp(self.grad(aa, xf, yf), self.grad(ba, xf - 1.0, yf), u),
            Self::lerp(
                self.grad(ab, xf, yf - 1.0),
                self.grad(bb, xf - 1.0, yf - 1.0),
                u,
            ),
            v,
        );

        // Normalization [-1..1] -> [0..1]
        value * 0.5 + 0.5
    }

    fn grad(&self, hash: usize, x: f64, y: f64) -> f64 {
        let (gx, gy) = match hash % 8 {
            0 => (1.0, 1.0),    // 45°
            1 => (-1.0, 1.0),   // 135°
            2 => (1.0, -1.0),   // 315°
            3 => (-1.0, -1.0),  // 225°
            4 => (1.0, 0.0),    // 0°
            5 => (-1.0, 0.0),   // 180°
            6 => (0.0, 1.0),    // 90°
            7 => (0.0, -1.0),   // 270°
            _ => unreachable!(),
        };
        
        (gx * x + gy * y) * 0.7071 // 1/√2 for diagonal vectors
    }

    fn lerp(a: f64, b: f64, t: f64) -> f64 {
        (1.0 - t) * a + t * b
    }

    fn fade(t: f64) -> f64 {
        ((t * 6.0 - 15.0) * t + 10.0) * t * t * t
    }
}
