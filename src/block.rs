#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BlockType {
    Grass,
    Snow,
    Stone,
    Wood,
}

impl BlockType {
    pub fn atlas_offset(&self) -> [f32; 2] {
        match self {
            BlockType::Grass => [0.0, 0.625],
            BlockType::Snow => [0.0625, 0.625],
            BlockType::Stone => [0.59375, 0.625],
            BlockType::Wood => [0.5, 0.0],
        }
    }
}
