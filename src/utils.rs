pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

pub fn ceil_div(numer: usize, denom: usize) -> usize {
    (numer + denom - 1) / denom
}
