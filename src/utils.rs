pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

pub fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
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
