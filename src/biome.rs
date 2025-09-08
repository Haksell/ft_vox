use crate::block::BlockType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiomeType {
    Desert,
    Plains,
    Forest,
    Taiga,
    Tundra,
    Swamp,
    Mountains,
    Beach,
    Peaks,
    FrozenPeaks,
    Ocean,
    ColdOcean,
    FrozenOcean,
    WarmOcean,
    DeepOcean,
    DeepColdOcean,
    DeepFrozenOcean,
    River,
    FrozenRiver,
    Grove,
    Mangrove,
    StonyShore,
    StonyPeaks,
    WindsweptSavanna,
    SnowySlopes,
    SnowyBeach,
    JaggedPeaks,
    Badlands,
    ErrodedBadlands,
    WoodedBadlands,
    Jungle,
    BambooJungle,
    SparseJungle,
    Savanna,
    DarkForest,
    OldGrowthBirchForest,
    BirchForest,
    SunflowerForest,
    FlowerForest,
    OldGrowthPineTaiga,
    OldGrowthSpruceTaiga,
    SnowyTaiga,
    SnowyPlains,
    IceSpikes,
    WindsweptHills,
    WindsweptForest,
    WindsweptGravellyHills,
    PaleGarden,
    Meadow,
    CherryGrove,
    SavannaPlateau,
}

impl BiomeType {
    pub fn get_surface_block(&self) -> BlockType {
        match self {
            BiomeType::Desert => BlockType::Sand,
            BiomeType::Plains => BlockType::Grass,
            BiomeType::Forest => BlockType::Gravel,
            BiomeType::Taiga => BlockType::Snow,
            BiomeType::Tundra => BlockType::Ice,
            BiomeType::Swamp => BlockType::Dirt,
            BiomeType::Mountains => BlockType::Stone,
            BiomeType::Beach => BlockType::Sand,
            BiomeType::SnowyBeach => BlockType::Sand,
            BiomeType::SnowyPlains => BlockType::Snow,
            BiomeType::SunflowerForest => BlockType::Grass,
            BiomeType::Ocean => BlockType::Sand,
            BiomeType::IceSpikes => BlockType::Ice,
            BiomeType::Badlands => BlockType::RedSand,
            _ => BlockType::Grass,
        }
    }

    pub fn get_subsurface_block(&self) -> BlockType {
        match self {
            BiomeType::Desert => BlockType::Sand,
            BiomeType::Plains => BlockType::Dirt,
            BiomeType::Forest => BlockType::Dirt,
            BiomeType::Taiga => BlockType::Dirt,
            BiomeType::Tundra => BlockType::Stone,
            BiomeType::Swamp => BlockType::Dirt,
            BiomeType::Mountains => BlockType::Stone,
            BiomeType::Beach => BlockType::Sand,
            BiomeType::Ocean => BlockType::Sand,
            BiomeType::Badlands => BlockType::RedSand,
            _ => BlockType::Grass,
        }
    }

    pub fn get_deep_block(&self) -> BlockType {
        BlockType::Stone
    }
}
