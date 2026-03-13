use crate::block::BlockType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiomeType {
    Desert,
    Plains,
    Forest,
    Taiga,
    Swamp,
    Beach,
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
    ErodedBadlands,
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
    pub const fn get_surface_block(&self) -> BlockType {
        match self {
            Self::Desert => BlockType::Sand,
            Self::Plains => BlockType::Grass,
            Self::Forest => BlockType::Grass,
            Self::Taiga => BlockType::Snow,
            Self::Swamp => BlockType::Dirt,
            Self::Beach => BlockType::Sand,
            Self::River => BlockType::Sand,
            Self::FrozenRiver => BlockType::Snow,
            Self::SnowyBeach => BlockType::Sand,
            Self::SnowyPlains => BlockType::Snow,
            Self::SnowyTaiga => BlockType::Snow,
            Self::SnowySlopes => BlockType::Snow,
            Self::SunflowerForest => BlockType::Grass,
            Self::Ocean => BlockType::Sand,
            Self::ColdOcean => BlockType::Sand,
            Self::FrozenOcean => BlockType::Sand,
            Self::WarmOcean => BlockType::Sand,
            Self::DeepOcean => BlockType::Sand,
            Self::DeepColdOcean => BlockType::Sand,
            Self::DeepFrozenOcean => BlockType::Sand,
            Self::IceSpikes => BlockType::Ice,
            Self::ErodedBadlands => BlockType::RedSand,
            Self::Badlands => BlockType::RedSand,
            Self::StonyPeaks => BlockType::Stone,
            Self::StonyShore => BlockType::Stone,
            Self::Mangrove => BlockType::WarpedNylium,
            _ => BlockType::Grass,
        }
    }

    pub const fn is_ocean(&self) -> bool {
        matches!(
            self,
            Self::FrozenOcean
                | Self::ColdOcean
                | Self::Ocean
                | Self::DeepFrozenOcean
                | Self::DeepColdOcean
                | Self::DeepOcean
                | Self::WarmOcean
        )
    }
}
