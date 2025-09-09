use {
    crate::{
        aabb::AABB,
        block::BlockType,
        face::{Face, FACES},
        vertex::Vertex,
    },
    glam::Vec3,
};

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;

pub type ChunkCoords = (i32, i32);
pub type ChunkNodeSize = (usize, usize, usize);

pub struct Chunk {
    coords: ChunkCoords,
    root: ChunkNode,
}

#[derive(Debug, Clone, Copy)]
enum SplitDir {
    LeftRight,
    FrontBack,
    TopBottom,
}

enum ChunkNode {
    Leaf(Option<BlockType>, ChunkNodeSize),
    Inner(Box<ChunkNode>, Box<ChunkNode>, SplitDir, ChunkNodeSize),
}

impl ChunkNode {
    fn from_region(
        blocks: &Blocks,
        x0: usize,
        x1: usize,
        y0: usize,
        y1: usize,
        z0: usize,
        z1: usize,
    ) -> Self {
        let sx = x1 - x0;
        let sy = y1 - y0;
        let sz = z1 - z0;
        debug_assert!(sx > 0 && sy > 0 && sz > 0);
        let size = (sx, sy, sz);

        if let Some(u) = uniform(blocks, x0, x1, y0, y1, z0, z1) {
            // Whole region is the same block (including "all air")
            return ChunkNode::Leaf(u, size);
        }

        // Choose the longest axis to split
        if sz >= sx && sz >= sy && sz > 1 {
            // Split along Z (TopBottom)
            let mid = z0 + sz / 2;
            let a = Box::new(ChunkNode::from_region(blocks, x0, x1, y0, y1, z0, mid));
            let b = Box::new(ChunkNode::from_region(blocks, x0, x1, y0, y1, mid, z1));
            return ChunkNode::merge_if_same(a, b, SplitDir::TopBottom, size);
        } else if sx >= sy && sx > 1 {
            // Split along X (LeftRight)
            let mid = x0 + sx / 2;
            let a = Box::new(ChunkNode::from_region(blocks, x0, mid, y0, y1, z0, z1));
            let b = Box::new(ChunkNode::from_region(blocks, mid, x1, y0, y1, z0, z1));
            return ChunkNode::merge_if_same(a, b, SplitDir::LeftRight, size);
        } else if sy > 1 {
            // Split along Y (FrontBack)
            let mid = y0 + sy / 2;
            let a = Box::new(ChunkNode::from_region(blocks, x0, x1, y0, mid, z0, z1));
            let b = Box::new(ChunkNode::from_region(blocks, x0, x1, mid, y1, z0, z1));
            return ChunkNode::merge_if_same(a, b, SplitDir::FrontBack, size);
        }

        // Fallback: region isn't uniform but we cannot split further (a 1×1×1 non-uniform is impossible,
        // so this covers degenerate ranges). Treat as a single mixed voxel (pick one) — or panic.
        // Safer is to make a leaf from the actual single cell value.
        let v = blocks[x0][y0][z0];
        ChunkNode::Leaf(v, size)
    }

    fn merge_if_same(
        a: Box<ChunkNode>,
        b: Box<ChunkNode>,
        dir: SplitDir,
        size: ChunkNodeSize,
    ) -> Self {
        match (&*a, &*b) {
            (ChunkNode::Leaf(va, _), ChunkNode::Leaf(vb, _)) if va == vb => {
                ChunkNode::Leaf(*va, size)
            }
            _ => ChunkNode::Inner(a, b, dir, size),
        }
    }

    fn get_at(
        &self,
        x: usize,
        y: usize,
        z: usize,
        ox: usize,
        oy: usize,
        oz: usize,
    ) -> Option<BlockType> {
        match self {
            ChunkNode::Leaf(v, _size) => *v,
            ChunkNode::Inner(a, b, dir, (sx, sy, sz)) => match dir {
                SplitDir::LeftRight => {
                    let midx = ox + sx / 2;
                    if x < midx {
                        a.get_at(x, y, z, ox, oy, oz)
                    } else {
                        b.get_at(x, y, z, midx, oy, oz)
                    }
                }
                SplitDir::FrontBack => {
                    let midy = oy + sy / 2;
                    if y < midy {
                        a.get_at(x, y, z, ox, oy, oz)
                    } else {
                        b.get_at(x, y, z, ox, midy, oz)
                    }
                }
                SplitDir::TopBottom => {
                    let midz = oz + sz / 2;
                    if z < midz {
                        a.get_at(x, y, z, ox, oy, oz)
                    } else {
                        b.get_at(x, y, z, ox, oy, midz)
                    }
                }
            },
        }
    }

    fn count_leaves(&self) -> u32 {
        match self {
            ChunkNode::Leaf(..) => 1,
            ChunkNode::Inner(a, b, ..) => a.count_leaves() + b.count_leaves(),
        }
    }
}

fn uniform(
    blocks: &Blocks,
    x0: usize,
    x1: usize,
    y0: usize,
    y1: usize,
    z0: usize,
    z1: usize,
) -> Option<Option<BlockType>> {
    let first = blocks[x0][y0][z0];
    for x in x0..x1 {
        for y in y0..y1 {
            for z in z0..z1 {
                if blocks[x][y][z] != first {
                    return None;
                }
            }
        }
    }
    Some(first)
}

pub struct AdjacentChunks<'a> {
    pub north: Option<(&'a Chunk, usize)>, // +y direction
    pub south: Option<(&'a Chunk, usize)>, // -y direction
    pub east: Option<(&'a Chunk, usize)>,  // +x direction
    pub west: Option<(&'a Chunk, usize)>,  // -x direction
}

type Blocks = [[[Option<BlockType>; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH];

impl Chunk {
    pub fn new(coords: ChunkCoords, blocks: Blocks) -> Self {
        let root = ChunkNode::from_region(&blocks, 0, CHUNK_WIDTH, 0, CHUNK_WIDTH, 0, CHUNK_HEIGHT);
        log::info!(
            "Chunk {:?} : {}/{} leaves",
            coords,
            root.count_leaves(),
            CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT
        );
        Self { coords, root }
    }

    pub fn bounding_box(&self) -> AABB {
        let (x, y) = self.coords;
        let world_x = x as f32 * CHUNK_WIDTH as f32;
        let world_y = y as f32 * CHUNK_WIDTH as f32;

        AABB::new(
            Vec3::new(world_x, world_y, 0.0),
            Vec3::new(
                world_x + CHUNK_WIDTH as f32,
                world_y + CHUNK_WIDTH as f32,
                CHUNK_HEIGHT as f32,
            ),
        )
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<BlockType> {
        if x >= CHUNK_WIDTH || y >= CHUNK_WIDTH || z >= CHUNK_HEIGHT {
            None
        } else {
            self.root.get_at(x, y, z, 0, 0, 0)
        }
    }

    fn is_face_visible(
        &self,
        neighbor_x: i32,
        neighbor_y: i32,
        neighbor_z: i32,
        adjacent: &AdjacentChunks,
        lod_step: usize,
    ) -> bool {
        if neighbor_z < 0 {
            return false;
        }
        if neighbor_z >= CHUNK_HEIGHT as i32 {
            return true;
        }

        if neighbor_x >= 0
            && neighbor_x < CHUNK_WIDTH as i32
            && neighbor_y >= 0
            && neighbor_y < CHUNK_WIDTH as i32
        {
            return self
                .get_block(
                    neighbor_x as usize,
                    neighbor_y as usize,
                    neighbor_z as usize,
                )
                .is_none();
        }

        // TODO: no loops, just check if used neighbor is present

        match (neighbor_x, neighbor_y) {
            (x, y) if x >= CHUNK_WIDTH as i32 && y >= 0 && y < CHUNK_WIDTH as i32 => {
                let Some((adj_chunk, adj_lod_step)) = adjacent.east else {
                    return true;
                };
                for dx in 0..lod_step {
                    for dy in 0..lod_step {
                        if adj_chunk
                            .get_block(dx, y as usize + dy, neighbor_z as usize)
                            .is_none()
                        {
                            return true;
                        }
                    }
                }
                false
            }
            (x, y) if x < 0 && y >= 0 && y < CHUNK_WIDTH as i32 => {
                let Some((adj_chunk, adj_lod_step)) = adjacent.west else {
                    return true;
                };
                for dx in 0..lod_step {
                    for dy in 0..lod_step {
                        if adj_chunk
                            .get_block(
                                CHUNK_WIDTH - lod_step + dx,
                                y as usize + dy,
                                neighbor_z as usize,
                            )
                            .is_none()
                        {
                            return true;
                        }
                    }
                }
                false
            }
            (x, y) if y >= CHUNK_WIDTH as i32 && x >= 0 && x < CHUNK_WIDTH as i32 => {
                let Some((adj_chunk, adj_lod_step)) = adjacent.north else {
                    return true;
                };
                for dx in 0..lod_step {
                    for dy in 0..lod_step {
                        if adj_chunk
                            .get_block(x as usize + dx, dy, neighbor_z as usize)
                            .is_none()
                        {
                            return true;
                        }
                    }
                }
                false
            }
            (x, y) if y < 0 && x >= 0 && x < CHUNK_WIDTH as i32 => {
                let Some((adj_chunk, adj_lod_step)) = adjacent.south else {
                    return true;
                };
                for dx in 0..lod_step {
                    for dy in 0..lod_step {
                        if adj_chunk
                            .get_block(
                                x as usize + dx,
                                CHUNK_WIDTH - lod_step + dy,
                                neighbor_z as usize,
                            )
                            .is_none()
                        {
                            return true;
                        }
                    }
                }
                false
            }
            _ => unreachable!(),
        }
    }

    fn create_face(
        block: BlockType,
        position: Vec3,
        face: Face,
        lod_step: usize,
    ) -> ([Vertex; 4], [u16; 6]) {
        let positions = face.positions();

        // stretch uv in x and y but not z direction
        let mut uvs = face.uvs();
        for y in 0..4 {
            uvs[y][0] *= lod_step as f32;
            if matches!(face, Face::Top | Face::Bottom) {
                uvs[y][1] *= lod_step as f32;
            }
        }

        let vertices = std::array::from_fn(|i| Vertex {
            position: [
                position.x + positions[i][0] * lod_step as f32,
                position.y + positions[i][1] * lod_step as f32,
                position.z + positions[i][2] as f32,
            ],
            tex_coords: uvs[i],
            atlas_offset: match face {
                Face::Top => block.atlas_offset_top(),
                Face::Bottom => block.atlas_offset_bottom(),
                Face::Left | Face::Right | Face::Front | Face::Back => block.atlas_offset_side(),
            },
        });

        let indices = [0, 1, 2, 2, 3, 0];

        (vertices, indices)
    }

    pub fn generate_mesh(
        &self,
        lod_step: usize,
        adjacent: &AdjacentChunks,
    ) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0;

        for local_x in (0..CHUNK_WIDTH).step_by(lod_step) {
            for local_y in (0..CHUNK_WIDTH).step_by(lod_step) {
                for local_z in 0..CHUNK_HEIGHT {
                    let Some(block) = self.get_block(local_x, local_y, local_z) else {
                        continue;
                    };

                    let position = Vec3::new(local_x as f32, local_y as f32, local_z as f32);

                    for face in FACES {
                        let (dx, dy, dz) = face.normal();
                        let neighbor_x = local_x as i32 + dx * lod_step as i32;
                        let neighbor_y = local_y as i32 + dy * lod_step as i32;
                        let neighbor_z = local_z as i32 + dz as i32;

                        if self
                            .is_face_visible(neighbor_x, neighbor_y, neighbor_z, adjacent, lod_step)
                        {
                            let (face_verts, face_indices) =
                                Self::create_face(block, position, face, lod_step);

                            vertices.extend(face_verts);
                            indices.extend(face_indices.iter().map(|i| *i + index_offset));
                            index_offset += 4;
                        }
                    }
                }
            }
        }

        (vertices, indices)
    }
}
