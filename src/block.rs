use wgpu::naga::Block;

#[derive(Debug, Clone, Copy)]
pub enum BlockType {
    Grass,
    Snow,
}

impl BlockType {
    pub fn atlas_offset(&self) -> (u32, u32) {
        match self {
            BlockType::Grass => (0, 10),
            BlockType::Snow => (2, 10),
        }
    }
}
