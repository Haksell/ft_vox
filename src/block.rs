#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockType {
    Basalt,
    Dirt,
    EmeraldOre,
    GoldOre,
    Grass,
    Ice,
    Magma,
    RedSand,
    RedStone,
    Sand,
    Snow,
    Stone,
    WarpedNylium,
    Water,
}

impl BlockType {
    pub fn atlas_offset_top(&self) -> [u32; 2] {
        match self {
            BlockType::Basalt => [12, 6],
            BlockType::Dirt => [25, 2],
            BlockType::EmeraldOre => [23, 12],
            BlockType::GoldOre => [23, 13],
            BlockType::Grass => [31, 2],
            BlockType::Ice => [4, 22],
            BlockType::Magma => [24, 26],
            BlockType::RedSand => [29, 25],
            BlockType::RedStone => [24, 0],
            BlockType::Sand => [6, 27],
            BlockType::Snow => [19, 24],
            BlockType::Stone => [30, 29],
            BlockType::WarpedNylium => [33, 20],
            BlockType::Water => [6, 4],
        }
    }

    pub fn atlas_offset_side(&self) -> [u32; 2] {
        match self {
            BlockType::Basalt => [12, 5],
            BlockType::Dirt => [25, 2],
            BlockType::EmeraldOre => [23, 12],
            BlockType::GoldOre => [23, 13],
            BlockType::Grass => [30, 15],
            BlockType::Ice => [4, 22],
            BlockType::Magma => [24, 26],
            BlockType::Sand => [6, 27],
            BlockType::Snow => [31, 1],
            BlockType::Stone => [30, 29],
            BlockType::RedSand => [28, 25],
            BlockType::RedStone => [24, 0],
            BlockType::WarpedNylium => [33, 21],
            BlockType::Water => [6, 4],
        }
    }

    pub fn atlas_offset_bottom(&self) -> [u32; 2] {
        match self {
            BlockType::Basalt => [12, 6],
            BlockType::Dirt => [25, 2],
            BlockType::EmeraldOre => [23, 12],
            BlockType::GoldOre => [23, 13],
            BlockType::Grass => [25, 2],
            BlockType::Ice => [4, 22],
            BlockType::Magma => [24, 26],
            BlockType::RedSand => [27, 25],
            BlockType::RedStone => [24, 0],
            BlockType::Sand => [6, 27],
            BlockType::Snow => [25, 2],
            BlockType::Stone => [30, 29],
            BlockType::WarpedNylium => [25, 20],
            BlockType::Water => [6, 4],
        }
    }
}
