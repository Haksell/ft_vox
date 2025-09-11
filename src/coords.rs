use crate::chunk::{CHUNK_HEIGHT, CHUNK_WIDTH};

pub type WorldCoords = (i32, i32, i32);
pub type ChunkCoords = (i32, i32);
pub type BlockCoords = (usize, usize, usize);

pub fn split_coords((x, y, z): WorldCoords) -> Option<(ChunkCoords, BlockCoords)> {
    let block_z = (z >= 0 && z < CHUNK_HEIGHT as i32).then(|| z as usize)?;

    let chunk_x = x.div_euclid(CHUNK_WIDTH as i32);
    let block_x = x.rem_euclid(CHUNK_WIDTH as i32) as usize;
    let chunk_y = y.div_euclid(CHUNK_WIDTH as i32);
    let block_y = y.rem_euclid(CHUNK_WIDTH as i32) as usize;

    Some(((chunk_x, chunk_y), (block_x, block_y, block_z)))
}
