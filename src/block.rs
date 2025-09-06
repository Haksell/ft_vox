#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BlockType {
    Grass,
    Snow,
}

impl BlockType {
    pub fn atlas_offset_top(&self) -> [u32; 2] {
        match self {
            BlockType::Grass => [29, 18],
            BlockType::Snow => [19, 24],
        }
    }

    pub fn atlas_offset_side(&self) -> [u32; 2] {
        match self {
            BlockType::Grass => [30, 15],
            BlockType::Snow => [31, 1],
        }
    }

    pub fn atlas_offset_bottom(&self) -> [u32; 2] {
        match self {
            BlockType::Grass => [25, 2],
            BlockType::Snow => [25, 2],
        }
    }
}
