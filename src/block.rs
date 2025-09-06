#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BlockType {
    Grass,
    Snow,
}

impl BlockType {
    pub fn atlas_offset(&self) -> [f32; 2] {
        match self {
            BlockType::Grass => [0.0, 0.625],
            BlockType::Snow => [0.0625, 0.625],
        }
    }
}
