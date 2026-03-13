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
            Self::Swamp => BlockType::Dirt,
            Self::Taiga
            | Self::FrozenRiver
            | Self::SnowyPlains
            | Self::SnowyTaiga
            | Self::SnowySlopes => BlockType::Snow,
            Self::Desert
            | Self::Beach
            | Self::River
            | Self::SnowyBeach
            | Self::Ocean
            | Self::ColdOcean
            | Self::FrozenOcean
            | Self::WarmOcean
            | Self::DeepOcean
            | Self::DeepColdOcean
            | Self::DeepFrozenOcean => BlockType::Sand,
            Self::IceSpikes => BlockType::Ice,
            Self::ErodedBadlands | Self::Badlands => BlockType::RedSand,
            Self::StonyPeaks | Self::StonyShore => BlockType::Stone,
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
