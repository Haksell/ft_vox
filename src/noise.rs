pub struct SimplexNoiseInfo {
    pub frequency: f32,
    pub octaves: usize,
    pub persistence: f32,
    pub lacunarity: f32,
}

impl Default for SimplexNoiseInfo {
    fn default() -> Self {
        Self {
            frequency: 0.005,
            octaves: 4,
            persistence: 0.4,
            lacunarity: 1.8,
        }
    }
}

pub struct SimplexNoise {
    permutations: [u8; 512],
    frequency: f32,
    octaves: usize,
    persistence: f32,
    lacunarity: f32,
}

impl SimplexNoise {
    // Constants for 2D simplex noise
    const F2: f32 = 0.3660254037844387; // (sqrt(3) - 1) / 2
    const G2: f32 = 0.21132486540518713; // (3 - sqrt(3)) / 6

    pub fn new(seed: u64, info: SimplexNoiseInfo) -> Self {
        let mut permutations = [0u8; 512];
        let mut temp = (0i32..256).map(|x| x as u8).collect::<Vec<u8>>();

        let mut hash = seed;
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

        SimplexNoise {
            permutations,
            frequency: info.frequency,
            octaves: info.octaves,
            persistence: info.persistence,
            lacunarity: info.lacunarity,
        }
    }

    pub fn noise2d(&self, x: f32, y: f32) -> f32 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = self.frequency;
        let mut max_value = 0.0;

        for _ in 0..self.octaves {
            let sample_x = x * frequency;
            let sample_y = y * frequency;

            let noise_value = self.simplex2d(sample_x, sample_y);
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

    fn simplex2d(&self, x: f32, y: f32) -> f32 {
        let s = (x + y) * Self::F2;
        let i = (x + s).floor();
        let j = (y + s).floor();

        let t = (i + j) * Self::G2;
        let x0 = x - (i - t);
        let y0 = y - (j - t);

        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };

        let x1 = x0 - i1 as f32 + Self::G2;
        let y1 = y0 - j1 as f32 + Self::G2;

        let x2 = x0 - 1.0 + 2.0 * Self::G2;
        let y2 = y0 - 1.0 + 2.0 * Self::G2;

        let ii = (i as i32) & 255;
        let jj = (j as i32) & 255;

        let gi0 =
            self.permutations[ii as usize + self.permutations[jj as usize] as usize] as usize % 12;
        let gi1 = self.permutations
            [(ii + i1) as usize + self.permutations[(jj + j1) as usize] as usize]
            as usize
            % 12;
        let gi2 = self.permutations
            [(ii + 1) as usize + self.permutations[(jj + 1) as usize] as usize]
            as usize
            % 12;

        let mut n0 = 0.0;
        let t0 = 0.5 - x0 * x0 - y0 * y0;
        if t0 >= 0.0 {
            let t0_sq = t0 * t0;
            n0 = t0_sq * t0_sq * self.dot2d(gi0, x0, y0);
        }

        let mut n1 = 0.0;
        let t1 = 0.5 - x1 * x1 - y1 * y1;
        if t1 >= 0.0 {
            let t1_sq = t1 * t1;
            n1 = t1_sq * t1_sq * self.dot2d(gi1, x1, y1);
        }

        let mut n2 = 0.0;
        let t2 = 0.5 - x2 * x2 - y2 * y2;
        if t2 >= 0.0 {
            let t2_sq = t2 * t2;
            n2 = t2_sq * t2_sq * self.dot2d(gi2, x2, y2);
        }

        70.0 * (n0 + n1 + n2)
    }

    const GRADIENT_2D: [(f32, f32); 12] = [
        (1.0, 1.0),
        (-1.0, 1.0),
        (1.0, -1.0),
        (-1.0, -1.0),
        (1.0, 0.0),
        (-1.0, 0.0),
        (1.0, 0.0),
        (-1.0, 0.0),
        (0.0, 1.0),
        (0.0, -1.0),
        (0.0, 1.0),
        (0.0, -1.0),
    ];

    fn dot2d(&self, gi: usize, x: f32, y: f32) -> f32 {
        let grad = Self::GRADIENT_2D[gi];
        grad.0 * x + grad.1 * y
    }
}
