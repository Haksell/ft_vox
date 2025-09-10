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

type Blocks = [[[Option<BlockType>; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH];

pub type ChunkCoords = (i32, i32);

pub struct AdjacentChunks<'a> {
    pub north: Option<&'a Chunk>,
    pub south: Option<&'a Chunk>,
    pub east: Option<&'a Chunk>,
    pub west: Option<&'a Chunk>,
}

pub struct Chunk {
    coords: ChunkCoords,
    root: ChunkNode,
}
impl Chunk {
    pub fn new(coords: ChunkCoords, blocks: Blocks) -> Self {
        let root = ChunkNode::from_region(
            &blocks,
            ChunkNodePos::new(0, CHUNK_WIDTH, 0, CHUNK_WIDTH, 0, CHUNK_HEIGHT),
        );
        log::info!(
            "Chunk {:?} : {}/{} leaves",
            coords,
            root.count_leaves(),
            CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT
        );
        Self { coords, root }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<BlockType> {
        debug_assert!(x < CHUNK_WIDTH);
        debug_assert!(y < CHUNK_WIDTH);
        debug_assert!(z < CHUNK_HEIGHT);
        self.root.get_at(x, y, z, 0, 0, 0)
    }

    pub fn delete_block(&mut self, x: usize, y: usize, z: usize) {
        debug_assert!(x < CHUNK_WIDTH);
        debug_assert!(y < CHUNK_WIDTH);
        debug_assert!(z < CHUNK_HEIGHT);
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

    pub fn generate_mesh(&self, adjacent: &AdjacentChunks) -> (Vec<Vertex>, Vec<u16>) {
        let (vertices, indices, _) = self.root.generate_mesh(self, adjacent);
        return (vertices, indices);
    }

    fn is_face_visible(&self, pos: &ChunkNodePos, face: Face, adjacent: &AdjacentChunks) -> bool {
        // build the 1-voxel-thick neighbor "slab" touching `pos` on `face`.
        // if the slab is inside this chunk, query `self`. If it lies outside, query the
        // corresponding adjacent chunk (or treat as empty if missing).

        match face {
            Face::Left => {
                if pos.x0 > 0 {
                    self.root.any_empty_in_region(&ChunkNodePos::new(
                        pos.x0 - 1,
                        pos.x0,
                        pos.y0,
                        pos.y1,
                        pos.z0,
                        pos.z1,
                    ))
                } else {
                    adjacent.west.is_none_or(|west| {
                        west.root.any_empty_in_region(&ChunkNodePos::new(
                            CHUNK_WIDTH - 1,
                            CHUNK_WIDTH,
                            pos.y0,
                            pos.y1,
                            pos.z0,
                            pos.z1,
                        ))
                    })
                }
            }
            Face::Right => {
                if pos.x1 < CHUNK_WIDTH {
                    self.root.any_empty_in_region(&ChunkNodePos::new(
                        pos.x1,
                        pos.x1 + 1,
                        pos.y0,
                        pos.y1,
                        pos.z0,
                        pos.z1,
                    ))
                } else {
                    adjacent.east.is_none_or(|east| {
                        east.root.any_empty_in_region(&ChunkNodePos::new(
                            0, 1, pos.y0, pos.y1, pos.z0, pos.z1,
                        ))
                    })
                }
            }
            Face::Back => {
                if pos.y1 < CHUNK_WIDTH {
                    self.root.any_empty_in_region(&ChunkNodePos::new(
                        pos.x0,
                        pos.x1,
                        pos.y1,
                        pos.y1 + 1,
                        pos.z0,
                        pos.z1,
                    ))
                } else {
                    adjacent.north.is_none_or(|north| {
                        north.root.any_empty_in_region(&ChunkNodePos::new(
                            pos.x0, pos.x1, 0, 1, pos.z0, pos.z1,
                        ))
                    })
                }
            }
            Face::Front => {
                if pos.y0 > 0 {
                    self.root.any_empty_in_region(&ChunkNodePos::new(
                        pos.x0,
                        pos.x1,
                        pos.y0 - 1,
                        pos.y0,
                        pos.z0,
                        pos.z1,
                    ))
                } else {
                    adjacent.south.is_none_or(|south| {
                        south.root.any_empty_in_region(&ChunkNodePos::new(
                            pos.x0,
                            pos.x1,
                            CHUNK_WIDTH - 1,
                            CHUNK_WIDTH,
                            pos.z0,
                            pos.z1,
                        ))
                    })
                }
            }
            Face::Top => {
                pos.z1 >= CHUNK_HEIGHT
                    || self.root.any_empty_in_region(&ChunkNodePos::new(
                        pos.x0,
                        pos.x1,
                        pos.y0,
                        pos.y1,
                        pos.z1,
                        pos.z1 + 1,
                    ))
            }
            Face::Bottom => {
                pos.z0 > 0 && {
                    self.root.any_empty_in_region(&ChunkNodePos::new(
                        pos.x0,
                        pos.x1,
                        pos.y0,
                        pos.y1,
                        pos.z0 - 1,
                        pos.z0,
                    ))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum SplitDir {
    LeftRight,
    FrontBack,
    TopBottom,
}

enum ChunkNode {
    Leaf(Option<BlockType>, ChunkNodePos),
    Inner(Box<ChunkNode>, Box<ChunkNode>, SplitDir, ChunkNodePos),
}
impl ChunkNode {
    fn generate_mesh(
        &self,
        chunk: &Chunk,
        adjacent: &AdjacentChunks,
    ) -> (Vec<Vertex>, Vec<u16>, u16) {
        match self {
            Self::Leaf(None, _) => (vec![], vec![], 0),
            Self::Leaf(Some(block_type), pos) => {
                let mut vertices = Vec::new();
                let mut indices = Vec::new();
                let mut index_offset = 0;

                for face in FACES {
                    if chunk.is_face_visible(pos, face, adjacent) {
                        vertices.extend(create_face_vertices(face, *block_type, &pos));
                        indices.extend([
                            index_offset,
                            index_offset + 1,
                            index_offset + 2,
                            index_offset + 2,
                            index_offset + 3,
                            index_offset,
                        ]);
                        index_offset += 4;
                    }
                }

                (vertices, indices, index_offset)
            }
            Self::Inner(a, b, _, _) => {
                let (mut vertices_a, mut indices_a, index_offset_a) =
                    a.generate_mesh(chunk, adjacent);
                let (vertices_b, indices_b, index_offset_b) = b.generate_mesh(chunk, adjacent);
                vertices_a.extend(vertices_b);
                indices_a.extend(indices_b.iter().map(|i| i + index_offset_a));
                (vertices_a, indices_a, index_offset_a + index_offset_b)
            }
        }
    }

    fn from_region(blocks: &Blocks, pos: ChunkNodePos) -> Self {
        let (sx, sy, sz) = pos.size();
        debug_assert!(sx > 0 && sy > 0 && sz > 0);

        if let Some(u) = uniform(blocks, &pos) {
            return Self::Leaf(u, pos);
        }

        // choose the longest axis to split, with a preference for z
        if sz >= sx && sz >= sy && sz > 1 {
            let mid = pos.z0 + sz / 2;
            let a = Box::new(Self::from_region(blocks, ChunkNodePos { z1: mid, ..pos }));
            let b = Box::new(Self::from_region(blocks, ChunkNodePos { z0: mid, ..pos }));
            return merge_if_same(a, b, SplitDir::TopBottom, pos);
        } else if sy >= sx {
            let mid = pos.y0 + sy / 2;
            let a = Box::new(Self::from_region(blocks, ChunkNodePos { y1: mid, ..pos }));
            let b = Box::new(Self::from_region(blocks, ChunkNodePos { y0: mid, ..pos }));
            return merge_if_same(a, b, SplitDir::FrontBack, pos);
        } else {
            let mid = pos.x0 + sx / 2;
            let a = Box::new(Self::from_region(blocks, ChunkNodePos { x1: mid, ..pos }));
            let b = Box::new(Self::from_region(blocks, ChunkNodePos { x0: mid, ..pos }));
            return merge_if_same(a, b, SplitDir::LeftRight, pos);
        }
    }

    #[allow(unused)] // useful for debugging
    fn count_leaves(&self) -> u32 {
        match self {
            Self::Leaf(..) => 1,
            Self::Inner(a, b, ..) => a.count_leaves() + b.count_leaves(),
        }
    }

    fn any_empty_in_region(&self, region: &ChunkNodePos) -> bool {
        match self {
            Self::Leaf(val, pos) => intersects(pos, region) && val.is_none(),
            Self::Inner(a, b, _, pos) => {
                intersects(pos, region) && a.any_empty_in_region(region)
                    || b.any_empty_in_region(region)
            }
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
            ChunkNode::Inner(a, b, dir, pos) => match dir {
                SplitDir::LeftRight => {
                    let midx = ox + pos.size_x() / 2;
                    if x < midx {
                        a.get_at(x, y, z, ox, oy, oz)
                    } else {
                        b.get_at(x, y, z, midx, oy, oz)
                    }
                }
                SplitDir::FrontBack => {
                    let midy = oy + pos.size_y() / 2;
                    if y < midy {
                        a.get_at(x, y, z, ox, oy, oz)
                    } else {
                        b.get_at(x, y, z, ox, midy, oz)
                    }
                }
                SplitDir::TopBottom => {
                    let midz = oz + pos.size_z() / 2;
                    if z < midz {
                        a.get_at(x, y, z, ox, oy, oz)
                    } else {
                        b.get_at(x, y, z, ox, oy, midz)
                    }
                }
            },
        }
    }
}

// [start, end)
struct ChunkNodePos {
    x0: usize,
    x1: usize,
    y0: usize,
    y1: usize,
    z0: usize,
    z1: usize,
}
impl ChunkNodePos {
    fn new(x0: usize, x1: usize, y0: usize, y1: usize, z0: usize, z1: usize) -> Self {
        Self {
            x0,
            x1,
            y0,
            y1,
            z0,
            z1,
        }
    }

    #[inline]
    fn size_x(&self) -> usize {
        self.x1 - self.x0
    }

    #[inline]
    fn size_y(&self) -> usize {
        self.y1 - self.y0
    }

    #[inline]
    fn size_z(&self) -> usize {
        self.z1 - self.z0
    }

    #[inline]
    fn size(&self) -> (usize, usize, usize) {
        (self.size_x(), self.size_y(), self.size_z())
    }
}

fn merge_if_same(
    a: Box<ChunkNode>,
    b: Box<ChunkNode>,
    dir: SplitDir,
    pos: ChunkNodePos,
) -> ChunkNode {
    match (&*a, &*b) {
        (ChunkNode::Leaf(va, _), ChunkNode::Leaf(vb, _)) if va == vb => ChunkNode::Leaf(*va, pos),
        _ => ChunkNode::Inner(a, b, dir, pos),
    }
}

fn uniform(
    blocks: &Blocks,
    &ChunkNodePos {
        x0,
        x1,
        y0,
        y1,
        z0,
        z1,
    }: &ChunkNodePos,
) -> Option<Option<BlockType>> {
    let first = blocks[x0][y0][z0];
    (x0..x1)
        .all(|x| (y0..y1).all(|y| (z0..z1).all(|z| blocks[x][y][z] == first)))
        .then(|| first)
}

fn create_face_vertices(face: Face, block: BlockType, pos: &ChunkNodePos) -> [Vertex; 4] {
    let size = pos.size();
    let (sx, sy, sz) = size;

    let face_uvs = face.uvs(size);
    let face_positions = face.positions();

    std::array::from_fn(|i| Vertex {
        position: [
            pos.x0 as f32 + face_positions[i][0] * sx as f32,
            pos.y0 as f32 + face_positions[i][1] * sy as f32,
            pos.z0 as f32 + face_positions[i][2] * sz as f32,
        ],
        tex_coords: face_uvs[i],
        atlas_offset: match face {
            Face::Top => block.atlas_offset_top(),
            Face::Bottom => block.atlas_offset_bottom(),
            Face::Left | Face::Right | Face::Front | Face::Back => block.atlas_offset_side(),
        },
    })
}

#[inline]
fn intersects(a: &ChunkNodePos, b: &ChunkNodePos) -> bool {
    a.x0 < b.x1 && a.x1 > b.x0 && a.y0 < b.y1 && a.y1 > b.y0 && a.z0 < b.z1 && a.z1 > b.z0
}
