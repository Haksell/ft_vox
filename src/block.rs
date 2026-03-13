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
    pub const fn atlas_offset_top(&self) -> [u32; 2] {
        match self {
            Self::Basalt => [12, 6],
            Self::Dirt => [25, 2],
            Self::EmeraldOre => [23, 12],
            Self::GoldOre => [23, 13],
            Self::Grass => [31, 2],
            Self::Ice => [4, 22],
            Self::Magma => [24, 26],
            Self::RedSand => [29, 25],
            Self::RedStone => [24, 0],
            Self::Sand => [6, 27],
            Self::Snow => [19, 24],
            Self::Stone => [30, 29],
            Self::WarpedNylium => [33, 20],
            Self::Water => [6, 4],
        }
    }

    pub const fn atlas_offset_side(&self) -> [u32; 2] {
        match self {
            Self::Basalt => [12, 5],
            Self::Dirt => [25, 2],
            Self::EmeraldOre => [23, 12],
            Self::GoldOre => [23, 13],
            Self::Grass => [30, 15],
            Self::Ice => [4, 22],
            Self::Magma => [24, 26],
            Self::Sand => [6, 27],
            Self::Snow => [31, 1],
            Self::Stone => [30, 29],
            Self::RedSand => [28, 25],
            Self::RedStone => [24, 0],
            Self::WarpedNylium => [33, 21],
            Self::Water => [6, 4],
        }
    }

    pub const fn atlas_offset_bottom(&self) -> [u32; 2] {
        match self {
            Self::Basalt => [12, 6],
            Self::Dirt | Self::Grass | Self::Snow => [25, 2],
            Self::EmeraldOre => [23, 12],
            Self::GoldOre => [23, 13],
            Self::Ice => [4, 22],
            Self::Magma => [24, 26],
            Self::RedSand => [27, 25],
            Self::RedStone => [24, 0],
            Self::Sand => [6, 27],
            Self::Stone => [30, 29],
            Self::WarpedNylium => [25, 20],
            Self::Water => [6, 4],
        }
    }
}
