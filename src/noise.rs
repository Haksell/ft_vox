use {
    crate::utils::{fade, lerp},
    glam::{Vec2, Vec3},
};

// TODO: remove PerlinNoiseBuilder in favor of PerlinNoise::new
pub struct PerlinNoiseBuilder {
    seed: u64,
    frequency: f32,
    octaves: usize,
    persistence: f32,
    lacunarity: f32,
}

impl PerlinNoiseBuilder {
    pub fn new(seed: u64) -> Self {
        PerlinNoiseBuilder {
            seed,
            frequency: 0.005,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        }
    }

    pub fn frequency(mut self, frequency: f32) -> Self {
        self.frequency = frequency.max(0.0001);
        self
    }

    pub fn octaves(mut self, octaves: usize) -> Self {
        self.octaves = octaves.clamp(1, 24);
        self
    }

    pub fn persistence(mut self, persistence: f32) -> Self {
        self.persistence = persistence.clamp(0.0, 1.0);
        self
    }

    pub fn lacunarity(mut self, lacunarity: f32) -> Self {
        self.lacunarity = lacunarity.max(1.0);
        self
    }

    pub fn build(self) -> PerlinNoise {
        let mut permutations = [0u8; 512];
        let mut temp = (0i32..256).map(|x| x as u8).collect::<Vec<u8>>();

        let mut hash = self.seed;
        for i in (0..256).rev() {
            hash ^= hash >> 12;
            hash ^= hash << 25;
            hash ^= hash >> 27;
            hash = hash.wrapping_mul(0x2545F4914F6CDD1D);

            let j = (hash % (i + 1)) as usize;
            temp.swap(i as usize, j);
        }

        for i in 0..512 {
            permutations[i] = temp[i % 256];
        }

        PerlinNoise {
            permutations,
            frequency: self.frequency,
            octaves: self.octaves,
            persistence: self.persistence,
            lacunarity: self.lacunarity,
        }
    }
}

pub struct PerlinNoise {
    permutations: [u8; 512],
    frequency: f32,
    octaves: usize,
    persistence: f32,
    lacunarity: f32,
}

impl PerlinNoise {
    pub fn noise2d(&self, x: f32, y: f32) -> f32 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = self.frequency;
        let mut max_value = 0.0;

        for _ in 0..self.octaves {
            let sample_x = x * frequency;
            let sample_y = y * frequency;

            let noise_value = self.perlin2d(sample_x, sample_y);
            value += noise_value * amplitude;
            max_value += amplitude;

            amplitude *= self.persistence;
            frequency *= self.lacunarity;
        }

        if max_value > 0.0 {
            value = value / max_value;
        }

        value.clamp(-1.0, 1.0)
    }

    pub fn noise3d(&self, x: f32, y: f32, z: f32) -> f32 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = self.frequency;
        let mut max_value = 0.0;

        for _ in 0..self.octaves {
            let sample_x = x * frequency;
            let sample_y = y * frequency;
            let sample_z = z * frequency;

            let noise_value = self.perlin3d(sample_x, sample_y, sample_z);
            value += noise_value * amplitude;
            max_value += amplitude;

            amplitude *= self.persistence;
            frequency *= self.lacunarity;
        }

        if max_value > 0.0 {
            value = value / max_value;
        }

        value.clamp(-1.0, 1.0)
    }

    fn perlin2d(&self, x: f32, y: f32) -> f32 {
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;

        let xf = x - xi as f32;
        let yf = y - yi as f32;

        let u = fade(xf);
        let v = fade(yf);

        let a = self.permutations[(xi & 255) as usize] as i32 + yi;
        let aa = self.permutations[(a & 255) as usize] as usize;
        let ab = self.permutations[((a + 1) & 255) as usize] as usize;

        let b = self.permutations[((xi + 1) & 255) as usize] as i32 + yi;
        let ba = self.permutations[(b & 255) as usize] as usize;
        let bb = self.permutations[((b + 1) & 255) as usize] as usize;

        let value = lerp(
            lerp(self.grad2d(aa, xf, yf), self.grad2d(ba, xf - 1.0, yf), u),
            lerp(
                self.grad2d(ab, xf, yf - 1.0),
                self.grad2d(bb, xf - 1.0, yf - 1.0),
                u,
            ),
            v,
        );

        value
    }

    fn perlin3d(&self, x: f32, y: f32, z: f32) -> f32 {
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;
        let zi = z.floor() as i32;

        let xf = x - xi as f32;
        let yf = y - yi as f32;
        let zf = z - zi as f32;

        let u = fade(xf);
        let v = fade(yf);
        let w = fade(zf);

        let a = self.permutations[(xi & 255) as usize] as i32 + yi;
        let aa = self.permutations[(a & 255) as usize] as i32 + zi;
        let aaa = self.permutations[(aa & 255) as usize] as usize;
        let aab = self.permutations[((aa + 1) & 255) as usize] as usize;

        let ab = self.permutations[((a + 1) & 255) as usize] as i32 + zi;
        let aba = self.permutations[(ab & 255) as usize] as usize;
        let abb = self.permutations[((ab + 1) & 255) as usize] as usize;

        let b = self.permutations[((xi + 1) & 255) as usize] as i32 + yi;
        let ba = self.permutations[(b & 255) as usize] as i32 + zi;
        let baa = self.permutations[(ba & 255) as usize] as usize;
        let bab = self.permutations[((ba + 1) & 255) as usize] as usize;

        let bb = self.permutations[((b + 1) & 255) as usize] as i32 + zi;
        let bba = self.permutations[(bb & 255) as usize] as usize;
        let bbb = self.permutations[((bb + 1) & 255) as usize] as usize;

        let value = lerp(
            lerp(
                lerp(
                    self.grad3d(aaa, xf, yf, zf),
                    self.grad3d(baa, xf - 1.0, yf, zf),
                    u,
                ),
                lerp(
                    self.grad3d(aba, xf, yf - 1.0, zf),
                    self.grad3d(bba, xf - 1.0, yf - 1.0, zf),
                    u,
                ),
                v,
            ),
            lerp(
                lerp(
                    self.grad3d(aab, xf, yf, zf - 1.0),
                    self.grad3d(bab, xf - 1.0, yf, zf - 1.0),
                    u,
                ),
                lerp(
                    self.grad3d(abb, xf, yf - 1.0, zf - 1.0),
                    self.grad3d(bbb, xf - 1.0, yf - 1.0, zf - 1.0),
                    u,
                ),
                v,
            ),
            w,
        );

        value
    }

    #[rustfmt::skip]
    const GRADIENT_2D: [Vec2; 16] = [
        Vec2::new( 1.0,         0.0),
        Vec2::new(-1.0,         0.0),
        Vec2::new( 0.0,         1.0),
        Vec2::new( 0.0,        -1.0),
        Vec2::new( 0.70710677,  0.70710677),
        Vec2::new(-0.70710677,  0.70710677),
        Vec2::new( 0.70710677, -0.70710677),
        Vec2::new(-0.70710677, -0.70710677),
        Vec2::new( 0.96592583,  0.25881905),
        Vec2::new(-0.96592583,  0.25881905),
        Vec2::new( 0.96592583, -0.25881905),
        Vec2::new(-0.96592583, -0.25881905),
        Vec2::new( 0.25881905,  0.96592583),
        Vec2::new(-0.25881905,  0.96592583),
        Vec2::new( 0.25881905, -0.96592583),
        Vec2::new(-0.25881905, -0.96592583),
    ];

    #[rustfmt::skip]
    const GRADIENT_3D: [Vec3; 12] = [
        Vec3::new( 1.0,  1.0,  0.0),
        Vec3::new(-1.0,  1.0,  0.0),
        Vec3::new( 1.0, -1.0,  0.0),
        Vec3::new(-1.0, -1.0,  0.0),
        Vec3::new( 1.0,  0.0,  1.0),
        Vec3::new(-1.0,  0.0,  1.0),
        Vec3::new( 1.0,  0.0, -1.0),
        Vec3::new(-1.0,  0.0, -1.0),
        Vec3::new( 0.0,  1.0,  1.0),
        Vec3::new( 0.0, -1.0,  1.0),
        Vec3::new( 0.0,  1.0, -1.0),
        Vec3::new( 0.0, -1.0, -1.0),
    ];

    fn grad2d(&self, hash: usize, x: f32, y: f32) -> f32 {
        let gradient = Self::GRADIENT_2D[hash & 15];
        let position = Vec2::new(x as f32, y as f32);
        gradient.dot(position) as f32
    }

    fn grad3d(&self, hash: usize, x: f32, y: f32, z: f32) -> f32 {
        let gradient = Self::GRADIENT_3D[hash % 12];
        let position = Vec3::new(x as f32, y as f32, z as f32);
        gradient.dot(position) as f32
    }
}
