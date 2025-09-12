#[allow(unused)] // TODO: remove
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockType {
    Dirt,
    EmeraldOre,
    GoldOre,
    Grass,
    Gravel,
    Ice,
    Magma,
    RedSand,
    RedStone,
    Sand,
    Snow,
    Stone,
    Water,
}

impl BlockType {
    pub fn atlas_offset_top(&self) -> [u32; 2] {
        match self {
            BlockType::Grass => [31, 2],
            BlockType::Snow => [19, 24],
            BlockType::Stone => [30, 29],
            BlockType::Sand => [6, 27],
            BlockType::Dirt => [25, 2],
            BlockType::Ice => [4, 22],
            BlockType::Water => [6, 4],
            BlockType::Gravel => [31, 3],
            BlockType::RedSand => [27, 25],
            BlockType::Magma => [24, 26],
            BlockType::RedStone => [24, 0],
            BlockType::GoldOre => [23, 13],
            BlockType::EmeraldOre => [23, 12],
        }
    }

    pub fn atlas_offset_side(&self) -> [u32; 2] {
        match self {
            BlockType::Grass => [30, 15],
            BlockType::Snow => [31, 1],
            BlockType::Stone => [30, 29],
            BlockType::Sand => [6, 27],
            BlockType::Dirt => [25, 2],
            BlockType::Ice => [4, 22],
            BlockType::Water => [6, 4],
            BlockType::Gravel => [31, 3],
            BlockType::RedSand => [27, 25],
            BlockType::Magma => [24, 26],
            BlockType::RedStone => [24, 0],
            BlockType::GoldOre => [23, 13],
            BlockType::EmeraldOre => [23, 12],
        }
    }

    pub fn atlas_offset_bottom(&self) -> [u32; 2] {
        match self {
            BlockType::Grass => [25, 2],
            BlockType::Snow => [25, 2],
            BlockType::Stone => [30, 29],
            BlockType::Sand => [6, 27],
            BlockType::Dirt => [25, 2],
            BlockType::Ice => [4, 22],
            BlockType::Water => [6, 4],
            BlockType::Gravel => [31, 3],
            BlockType::RedSand => [27, 25],
            BlockType::Magma => [24, 26],
            BlockType::RedStone => [24, 0],
            BlockType::GoldOre => [23, 13],
            BlockType::EmeraldOre => [23, 12],
        }
    }
}
