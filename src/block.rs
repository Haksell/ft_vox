#[derive(Debug, Clone, Copy)]
pub enum BlockType {
    Grass,
    Snow,
}

impl BlockType {
    pub fn atlas_offset(&self) -> [f32; 2] {
        match self {
            BlockType::Grass => [0.0, 0.625],
            BlockType::Snow => [2. / 32., 0.625],
        }
    }
}
