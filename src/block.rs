#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BlockType {
    Grass,
    Snow,
}

impl BlockType {
    pub fn atlas_offset_top(&self) -> [u32; 2] {
        match self {
            BlockType::Grass => [22, 0],
            BlockType::Snow => [19, 7],
        }
    }

    pub fn atlas_offset_side(&self) -> [u32; 2] {
        match self {
            BlockType::Grass => [0, 10],
            BlockType::Snow => [2, 10],
        }
    }

    pub fn atlas_offset_bottom(&self) -> [u32; 2] {
        match self {
            BlockType::Grass => [8, 4],
            BlockType::Snow => [8, 4], // same as grass?
        }
    }
}
