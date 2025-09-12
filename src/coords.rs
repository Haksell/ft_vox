use {
    crate::chunk::{CHUNK_HEIGHT, CHUNK_WIDTH},
    glam::Vec3,
};

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

pub fn camera_to_world_coords(camera_coords: Vec3) -> WorldCoords {
    (
        camera_coords.x.floor() as i32,
        camera_coords.y.floor() as i32,
        camera_coords.z.floor() as i32,
    )
}

pub fn camera_to_chunk_coords(camera_coords: Vec3) -> ChunkCoords {
    let chunk_x = (camera_coords.x / CHUNK_WIDTH as f32).floor() as i32;
    let chunk_y = (camera_coords.y / CHUNK_WIDTH as f32).floor() as i32;
    (chunk_x, chunk_y)
}

pub fn chunk_distance_squared((cx1, cy1): ChunkCoords, (cx2, cy2): ChunkCoords) -> i32 {
    (cx1 - cx2).pow(2) + (cy1 - cy2).pow(2)
}

pub fn chunk_distance(cc1: ChunkCoords, cc2: ChunkCoords) -> f32 {
    (chunk_distance_squared(cc1, cc2) as f32).sqrt()
}
