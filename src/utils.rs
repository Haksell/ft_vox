pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

pub fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}