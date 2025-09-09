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

// [start, end)
struct ChunkNodePos {
    pub x0: usize,
    pub x1: usize,
    pub y0: usize,
    pub y1: usize,
    pub z0: usize,
    pub z1: usize,
}
impl ChunkNodePos {
    fn from_dimensions(x: usize, y: usize, z: usize) -> Self {
        Self {
            x0: 0,
            x1: x,
            y0: 0,
            y1: y,
            z0: 0,
            z1: z,
        }
    }

    #[inline]
    pub fn size_x(&self) -> usize {
        self.x1 - self.x0
    }

    #[inline]
    pub fn size_y(&self) -> usize {
        self.y1 - self.y0
    }

    #[inline]
    pub fn size_z(&self) -> usize {
        self.z1 - self.z0
    }
}

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
    Leaf(Option<BlockType>, ChunkNodePos),
    Inner(Box<ChunkNode>, Box<ChunkNode>, SplitDir, ChunkNodePos),
}

impl ChunkNode {
    fn from_region(blocks: &Blocks, chunk_node_pos: ChunkNodePos) -> Self {
        let ChunkNodePos {
            x0,
            x1,
            y0,
            y1,
            z0,
            z1,
        } = chunk_node_pos;

        let sx = x1 - x0;
        let sy = y1 - y0;
        let sz = z1 - z0;
        debug_assert!(sx > 0 && sy > 0 && sz > 0);

        if let Some(u) = uniform(blocks, x0, x1, y0, y1, z0, z1) {
            return ChunkNode::Leaf(u, chunk_node_pos);
        }

        // choose the longest axis to split, with a preference for z
        if sz >= sx && sz >= sy && sz > 1 {
            let mid = z0 + sz / 2;
            let a = Box::new(ChunkNode::from_region(
                blocks,
                ChunkNodePos {
                    z1: mid,
                    ..chunk_node_pos
                },
            ));
            let b = Box::new(ChunkNode::from_region(
                blocks,
                ChunkNodePos {
                    z0: mid,
                    ..chunk_node_pos
                },
            ));
            return ChunkNode::merge_if_same(a, b, SplitDir::TopBottom, chunk_node_pos);
        } else if sx >= sy && sx > 1 {
            let mid = x0 + sx / 2;
            let a = Box::new(ChunkNode::from_region(
                blocks,
                ChunkNodePos {
                    x1: mid,
                    ..chunk_node_pos
                },
            ));
            let b = Box::new(ChunkNode::from_region(
                blocks,
                ChunkNodePos {
                    x0: mid,
                    ..chunk_node_pos
                },
            ));
            return ChunkNode::merge_if_same(a, b, SplitDir::LeftRight, chunk_node_pos);
        } else if sy > 1 {
            let mid = y0 + sy / 2;
            let a = Box::new(ChunkNode::from_region(
                blocks,
                ChunkNodePos {
                    y1: mid,
                    ..chunk_node_pos
                },
            ));
            let b = Box::new(ChunkNode::from_region(
                blocks,
                ChunkNodePos {
                    y0: mid,
                    ..chunk_node_pos
                },
            ));
            return ChunkNode::merge_if_same(a, b, SplitDir::FrontBack, chunk_node_pos);
        }

        // TODO?: unreachable!()

        // Fallback: region isn't uniform but we cannot split further (a 1×1×1 non-uniform is impossible,
        // so this covers degenerate ranges). Treat as a single mixed voxel (pick one) — or panic.
        // Safer is to make a leaf from the actual single cell value.
        let v = blocks[x0][y0][z0];
        ChunkNode::Leaf(v, chunk_node_pos)
    }

    fn merge_if_same(
        a: Box<ChunkNode>,
        b: Box<ChunkNode>,
        dir: SplitDir,
        chunk_node_pos: ChunkNodePos,
    ) -> Self {
        match (&*a, &*b) {
            (ChunkNode::Leaf(va, _), ChunkNode::Leaf(vb, _)) if va == vb => {
                ChunkNode::Leaf(*va, chunk_node_pos)
            }
            _ => ChunkNode::Inner(a, b, dir, chunk_node_pos),
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
            ChunkNode::Leaf(v, _) => *v,
            ChunkNode::Inner(a, b, dir, chunk_node_pos) => match dir {
                SplitDir::LeftRight => {
                    let midx = ox + chunk_node_pos.size_x() / 2;
                    if x < midx {
                        a.get_at(x, y, z, ox, oy, oz)
                    } else {
                        b.get_at(x, y, z, midx, oy, oz)
                    }
                }
                SplitDir::FrontBack => {
                    let midy = oy + chunk_node_pos.size_y() / 2;
                    if y < midy {
                        a.get_at(x, y, z, ox, oy, oz)
                    } else {
                        b.get_at(x, y, z, ox, midy, oz)
                    }
                }
                SplitDir::TopBottom => {
                    let midz = oz + chunk_node_pos.size_z() / 2;
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
    pub north: Option<&'a Chunk>, // +y direction
    pub south: Option<&'a Chunk>, // -y direction
    pub east: Option<&'a Chunk>,  // +x direction
    pub west: Option<&'a Chunk>,  // -x direction
}

type Blocks = [[[Option<BlockType>; CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH];

impl Chunk {
    pub fn new(coords: ChunkCoords, blocks: Blocks) -> Self {
        let root = ChunkNode::from_region(
            &blocks,
            ChunkNodePos::from_dimensions(CHUNK_WIDTH, CHUNK_WIDTH, CHUNK_HEIGHT),
        );
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

        // TODO: is_none_or for all 4

        match (neighbor_x, neighbor_y) {
            (x, y) if x >= CHUNK_WIDTH as i32 && y >= 0 && y < CHUNK_WIDTH as i32 => {
                let Some(adj_chunk) = adjacent.east else {
                    return true;
                };
                if adj_chunk
                    .get_block(0, y as usize + 0, neighbor_z as usize)
                    .is_none()
                {
                    return true;
                }
                false
            }
            (x, y) if x < 0 && y >= 0 && y < CHUNK_WIDTH as i32 => {
                let Some(adj_chunk) = adjacent.west else {
                    return true;
                };
                if adj_chunk
                    .get_block(CHUNK_WIDTH - 1, y as usize, neighbor_z as usize)
                    .is_none()
                {
                    return true;
                }
                false
            }
            (x, y) if y >= CHUNK_WIDTH as i32 && x >= 0 && x < CHUNK_WIDTH as i32 => {
                adjacent.north.is_none_or(|adj_chunk| {
                    adj_chunk
                        .get_block(x as usize, 0, neighbor_z as usize)
                        .is_none()
                })
            }
            (x, y) if y < 0 && x >= 0 && x < CHUNK_WIDTH as i32 => {
                adjacent.south.is_none_or(|adj_chunk| {
                    adj_chunk
                        .get_block(x as usize, CHUNK_WIDTH - 1, neighbor_z as usize)
                        .is_none()
                })
            }
            _ => unreachable!(),
        }
    }

    pub fn generate_mesh(&self, adjacent: &AdjacentChunks) -> (Vec<Vertex>, Vec<u16>) {
        let (v, i, _) = self.root.generate_mesh(adjacent);
        println!("{} {}", v.len(), i.len());
        return (v, i);
    }
}

impl ChunkNode {
    fn generate_mesh(&self, adjacent: &AdjacentChunks) -> (Vec<Vertex>, Vec<u16>, u16) {
        match self {
            ChunkNode::Leaf(None, _) => (vec![], vec![], 0),
            ChunkNode::Leaf(Some(block_type), chunk_node_pos) => {
                let mut vertices = Vec::new();
                let mut indices = Vec::new();
                let mut index_offset = 0;

                // TODO: use again to avoid computing it 6 times
                // let position = Vec3::new(
                //     chunk_node_pos.x0 as f32,
                //     chunk_node_pos.y0 as f32,
                //     chunk_node_pos.z0 as f32,
                // );

                for face in FACES {
                    // if chunk.is_face_visible() {
                    let (face_verts, face_indices) =
                        create_face(face, *block_type, &chunk_node_pos);

                    vertices.extend(face_verts);
                    indices.extend(face_indices.iter().map(|i| *i + index_offset));
                    index_offset += 4;
                    // }
                }

                (vertices, indices, index_offset)
            }
            ChunkNode::Inner(a, b, _, _) => {
                let (mut vertices_a, mut indices_a, index_offset_a) = a.generate_mesh(adjacent);
                let (vertices_b, indices_b, index_offset_b) = b.generate_mesh(adjacent);
                vertices_a.extend(vertices_b);
                indices_a.extend(indices_b.iter().map(|i| i + index_offset_a));
                (vertices_a, indices_a, index_offset_a + index_offset_b)
            }
        }
    }
}

fn create_face(
    face: Face,
    block: BlockType,
    chunk_node_pos: &ChunkNodePos,
) -> ([Vertex; 4], [u16; 6]) {
    let positions = face.positions();

    // TODO: stretch uv in x and y but not z direction
    let uvs = face.uvs();
    // for y in 0..4 {
    //     uvs[y][0] *= ??? as f32;
    //     if matches!(face, Face::Top | Face::Bottom) {
    //         uvs[y][1] *= ??? as f32;
    //     }
    // }

    // TODO: in calling function
    let position = Vec3::new(
        chunk_node_pos.x0 as f32,
        chunk_node_pos.y0 as f32,
        chunk_node_pos.z0 as f32,
    );

    let vertices = std::array::from_fn(|i| Vertex {
        position: [
            position.x + positions[i][0] * chunk_node_pos.size_x() as f32,
            position.y + positions[i][1] * chunk_node_pos.size_y() as f32,
            position.z + positions[i][2] * chunk_node_pos.size_z() as f32,
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
