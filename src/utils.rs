use crate::coords::WorldCoords;

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

pub fn ceil_div(numer: usize, denom: usize) -> usize {
    (numer + denom - 1) / denom
}

pub fn sign(x: f32) -> i32 {
    if x > 0.0 {
        1
    } else if x < 0.0 {
        -1
    } else {
        0
    }
}

pub fn prf_i32x3_mod((x, y, z): WorldCoords, m: u64) -> u64 {
    debug_assert!(m > 0);

    #[inline]
    fn mix64(mut x: u64) -> u64 {
        x ^= x >> 30;
        x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x ^= x >> 27;
        x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^ (x >> 31)
    }

    let mut seed = (x as u32 as u64).wrapping_mul(0x9E37_79B1_85EB_CA87)
        ^ (y as u32 as u64).rotate_left(21)
        ^ (z as u32 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ 0x9E37_79B9_7F4A_7C15;

    let threshold: u64 = m.wrapping_neg() % m; // == 2^64 mod m

    const STEP: u64 = 0x9E37_79B9_7F4A_7C15;
    loop {
        let r = mix64(seed);
        let prod = (r as u128) * (m as u128);
        let lo = prod as u64;
        if lo >= threshold {
            return (prod >> 64) as u64;
        }
        seed = seed.wrapping_add(STEP);
    }
}
