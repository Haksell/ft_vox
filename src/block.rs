#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BlockType {
    Grass,
    Snow,
}

impl BlockType {
    pub fn atlas_offset(&self) -> [u32; 2] {
        match self {
            BlockType::Grass => [0, 10],
            BlockType::Snow => [2, 10],
        }
    }
}
