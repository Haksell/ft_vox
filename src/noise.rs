use image::{ImageBuffer, Rgb, RgbImage};

pub struct PerlinNoise {
    permutations: [u8; 512],
    frequency: f64,
    octaves: usize,
    persistence: f64,
    lacunarity: f64,
}

impl PerlinNoise {
    pub fn new(seed: u64) -> Self {
        let mut permutations = [0u8; 512];
        let mut temp = (0..256).map(|x| x as u8).collect::<Vec<u8>>();

        // Pseudo Random Number Generator
        let mut hash = seed;
        for i in (0..256).rev() {
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

        PerlinNoise {
            permutations,
            frequency: 0.01,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        }
    }

    pub fn generate(&self, width: u32, height: u32, path: &str) -> Result<(), image::ImageError> {
        // Create a new image buffer
        let mut img: RgbImage = ImageBuffer::new(width, height);

        // Fill the image with the noise values
        for y in 0..height {
            for x in 0..width {
                let nx = x as f64;
                let ny = y as f64;

                let noise_value = self.noise2d(nx, ny);
                let pixel_value = (noise_value * 255.0) as u8;

                img.put_pixel(x, y, Rgb([pixel_value, pixel_value, pixel_value]));
            }
        }

        // Save the image
        img.save(path)
    }

    pub fn noise2d(&self, x: f64, y: f64) -> f64 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = self.frequency;

        for _ in 0..self.octaves {
            let sample_x = x * frequency;
            let sample_y = y * frequency;

            let noise_value = self.perlin2d(sample_x, sample_y);
            value += noise_value * amplitude;

            amplitude *= self.persistence;
            frequency *= self.lacunarity;
        }

        value.clamp(-1.0, 1.0) * 0.5 + 0.5
    }

    fn perlin2d(&self, x: f64, z: f64) -> f64 {
        let xi = x.floor() as i32;
        let zi = z.floor() as i32;

        let xf = x - xi as f64;
        let zf = z - zi as f64;

        let u = Self::fade(xf);
        let v = Self::fade(zf);

        // Hash coordinates
        let a = self.permutations[(xi & 255) as usize] as i32 + zi;
        let aa = self.permutations[(a & 255) as usize] as usize;
        let ab = self.permutations[((a + 1) & 255) as usize] as usize;

        let b = self.permutations[((xi + 1) & 255) as usize] as i32 + zi;
        let ba = self.permutations[(b & 255) as usize] as usize;
        let bb = self.permutations[((b + 1) & 255) as usize] as usize;

        // Interpolate
        #[rustfmt::skip]
        let value = Self::lerp(
            Self::lerp(
                self.grad2d(aa, xf, zf),
                self.grad2d(ba, xf - 1.0, zf),
                u
            ),
            Self::lerp(
                self.grad2d(ab, xf, zf - 1.0),
                self.grad2d(bb, xf - 1.0, zf - 1.0),
                u,
            ),
            v,
        );

        value
    }

    #[rustfmt::skip]
    const GRADIENT_2D: [glam::Vec2; 8] = [
        glam::Vec2::new( 0.70710677,  0.70710677), // ( 1,  1) normalized
        glam::Vec2::new(-0.70710677,  0.70710677), // (-1,  1) normalized
        glam::Vec2::new( 0.70710677, -0.70710677), // ( 1, -1) normalized
        glam::Vec2::new(-0.70710677, -0.70710677), // (-1, -1) normalized
        glam::Vec2::new( 1.0,         0.0),
        glam::Vec2::new(-1.0,         0.0),
        glam::Vec2::new( 0.0,         1.0),
        glam::Vec2::new( 0.0,        -1.0),
    ];

    fn grad2d(&self, hash: usize, x: f64, y: f64) -> f64 {
        let gradient = Self::GRADIENT_2D[hash % 8];
        let position = glam::Vec2::new(x as f32, y as f32);

        gradient.dot(position) as f64
    }

    pub fn noise3d(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = self.frequency;

        for _ in 0..self.octaves {
            let sample_x = x * frequency;
            let sample_y = y * frequency;
            let sample_z = z * frequency;

            let noise_value = self.perlin3d(sample_x, sample_y, sample_z);
            value += noise_value * amplitude;

            amplitude *= self.persistence;
            frequency *= self.lacunarity;
        }

        value.clamp(-1.0, 1.0) * 0.5 + 0.5
    }

    fn perlin3d(&self, x: f64, y: f64, z: f64) -> f64 {
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;
        let zi = z.floor() as i32;

        let xf = x - xi as f64;
        let yf = y - yi as f64;
        let zf = z - zi as f64;

        let u = Self::fade(xf);
        let v = Self::fade(yf);
        let w = Self::fade(zf);

        // Hash coordinates for 3D
        let a = self.permutations[xi as usize] as i32 + yi;
        let aa = self.permutations[a as usize] as i32 + zi;
        let ab = self.permutations[(a + 1) as usize] as i32 + zi;

        let b = self.permutations[(xi + 1) as usize] as i32 + yi;
        let ba = self.permutations[b as usize] as i32 + zi;
        let bb = self.permutations[(b + 1) as usize] as i32 + zi;

        #[rustfmt::skip]
        // Interpolate between 8 corners
        let value = Self::lerp(
            Self::lerp(
                Self::lerp(
                    self.grad3d(
                        self.permutations[aa as usize] as usize,
                        xf,
                        yf,
                        zf
                    ),
                    self.grad3d(
                        self.permutations[ba as usize] as usize,
                        xf - 1.0,
                        yf,
                        zf
                    ),
                    u,
                ),
                Self::lerp(
                    self.grad3d(
                        self.permutations[ab as usize] as usize,
                        xf,
                        yf - 1.0,
                        zf
                    ),
                    self.grad3d(
                        self.permutations[bb as usize] as usize,
                        xf - 1.0,
                        yf - 1.0,
                        zf,
                    ),
                    u,
                ),
                v,
            ),
            Self::lerp(
                Self::lerp(
                    self.grad3d(
                        self.permutations[(aa + 1) as usize] as usize,
                        xf,
                        yf,
                        zf - 1.0,
                    ),
                    self.grad3d(
                        self.permutations[(ba + 1) as usize] as usize,
                        xf - 1.0,
                        yf,
                        zf - 1.0,
                    ),
                    u,
                ),
                Self::lerp(
                    self.grad3d(
                        self.permutations[(ab + 1) as usize] as usize,
                        xf,
                        yf - 1.0,
                        zf - 1.0,
                    ),
                    self.grad3d(
                        self.permutations[(bb + 1) as usize] as usize,
                        xf - 1.0,
                        yf - 1.0,
                        zf - 1.0,
                    ),
                    u,
                ),
                v,
            ),
            w,
        );

        value
    }

    #[rustfmt::skip]
    const GRADIENT_3D: [glam::Vec3; 12] = [
        glam::Vec3::new( 0.70710677,  0.70710677,  0.0),         // ( 1,  1,  0) normalized
        glam::Vec3::new(-0.70710677,  0.70710677,  0.0),         // (-1,  1,  0) normalized
        glam::Vec3::new( 0.70710677, -0.70710677,  0.0),         // ( 1, -1,  0) normalized
        glam::Vec3::new(-0.70710677, -0.70710677,  0.0),         // (-1, -1,  0) normalized
        glam::Vec3::new( 0.70710677,  0.0,         0.70710677),  // ( 1,  0,  1) normalized
        glam::Vec3::new(-0.70710677,  0.0,         0.70710677),  // (-1,  0,  1) normalized
        glam::Vec3::new( 0.70710677,  0.0,        -0.70710677),  // ( 1,  0, -1) normalized
        glam::Vec3::new(-0.70710677,  0.0,        -0.70710677),  // (-1,  0, -1) normalized
        glam::Vec3::new( 0.0,         0.70710677,  0.70710677),  // ( 0,  1,  1) normalized
        glam::Vec3::new( 0.0,        -0.70710677,  0.70710677),  // ( 0, -1,  1) normalized
        glam::Vec3::new( 0.0,         0.70710677, -0.70710677),  // ( 0,  1, -1) normalized
        glam::Vec3::new( 0.0,        -0.70710677, -0.70710677),  // ( 0, -1, -1) normalized
    ];

    fn grad3d(&self, hash: usize, x: f64, y: f64, z: f64) -> f64 {
        let gradient = Self::GRADIENT_3D[hash % 12];
        let position = glam::Vec3::new(x as f32, y as f32, z as f32);

        gradient.dot(position) as f64
    }

    fn lerp(a: f64, b: f64, t: f64) -> f64 {
        (1.0 - t) * a + t * b
    }

    fn fade(t: f64) -> f64 {
        ((t * 6.0 - 15.0) * t + 10.0) * t * t * t
    }
}
