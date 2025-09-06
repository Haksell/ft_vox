#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BlockType {
    Grass,
    Snow,
    Sand,
    Stone,
    Dirt,
}

impl BlockType {
    pub fn atlas_offset_top(&self) -> [u32; 2] {
        match self {
            BlockType::Grass => [1, 16],
            BlockType::Snow => [19, 24],
            BlockType::Stone => [30, 29],
            BlockType::Sand => [6, 27],
            BlockType::Dirt => [25, 2],
        }
    }

    pub fn atlas_offset_side(&self) -> [u32; 2] {
        match self {
            BlockType::Grass => [30, 15],
            BlockType::Snow => [31, 1],
            BlockType::Stone => [30, 29],
            BlockType::Sand => [6, 27],
            BlockType::Dirt => [25, 2],
        }
    }

    pub fn atlas_offset_bottom(&self) -> [u32; 2] {
        match self {
            BlockType::Grass => [25, 2],
            BlockType::Snow => [25, 2],
            BlockType::Stone => [30, 29],
            BlockType::Sand => [6, 27],
            BlockType::Dirt => [25, 2],
        }
    }
}
