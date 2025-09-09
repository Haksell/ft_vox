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

enum ChunkNode {
    Leaf(Option<BlockType>, ChunkNodePos),
    Inner(Box<ChunkNode>, Box<ChunkNode>, ChunkNodePos),
}

impl ChunkNode {
    fn from_region(blocks: &Blocks, chunk_node_pos: ChunkNodePos) -> Self {
        let sx = chunk_node_pos.size_x();
        let sy = chunk_node_pos.size_y();
        let sz = chunk_node_pos.size_z();
        debug_assert!(sx > 0 && sy > 0 && sz > 0);

        if let Some(u) = uniform(blocks, &chunk_node_pos) {
            return ChunkNode::Leaf(u, chunk_node_pos);
        }

        // choose the longest axis to split, with a preference for z
        if sz >= sx && sz >= sy && sz > 1 {
            let mid = chunk_node_pos.z0 + sz / 2;
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
            return merge_if_same(a, b, chunk_node_pos);
        } else if sy >= sx {
            let mid = chunk_node_pos.y0 + sy / 2;
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
            return merge_if_same(a, b, chunk_node_pos);
        } else {
            let mid = chunk_node_pos.x0 + sx / 2;
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
            return merge_if_same(a, b, chunk_node_pos);
        }
    }

    #[allow(unused)] // useful for debugging
    fn count_leaves(&self) -> u32 {
        match self {
            ChunkNode::Leaf(..) => 1,
            ChunkNode::Inner(a, b, ..) => a.count_leaves() + b.count_leaves(),
        }
    }
}

fn merge_if_same(a: Box<ChunkNode>, b: Box<ChunkNode>, chunk_node_pos: ChunkNodePos) -> ChunkNode {
    match (&*a, &*b) {
        (ChunkNode::Leaf(va, _), ChunkNode::Leaf(vb, _)) if va == vb => {
            ChunkNode::Leaf(*va, chunk_node_pos)
        }
        _ => ChunkNode::Inner(a, b, chunk_node_pos),
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

    pub fn generate_mesh(&self, adjacent: &AdjacentChunks) -> (Vec<Vertex>, Vec<u16>) {
        let (vertices, indices, _) = self.root.generate_mesh(self, adjacent);
        return (vertices, indices);
    }
}

impl ChunkNode {
    fn generate_mesh(
        &self,
        chunk: &Chunk,
        adjacent: &AdjacentChunks,
    ) -> (Vec<Vertex>, Vec<u16>, u16) {
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
                    if chunk.is_face_visible(chunk_node_pos, face, adjacent) {
                        let (face_verts, face_indices) =
                            create_face(face, *block_type, &chunk_node_pos);

                        vertices.extend(face_verts);
                        indices.extend(face_indices.iter().map(|i| *i + index_offset));
                        index_offset += 4;
                    }
                }

                (vertices, indices, index_offset)
            }
            ChunkNode::Inner(a, b, _) => {
                let (mut vertices_a, mut indices_a, index_offset_a) =
                    a.generate_mesh(chunk, adjacent);
                let (vertices_b, indices_b, index_offset_b) = b.generate_mesh(chunk, adjacent);
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

#[inline]
fn intersects(a: &ChunkNodePos, b: &ChunkNodePos) -> bool {
    a.x0 < b.x1 && a.x1 > b.x0 && a.y0 < b.y1 && a.y1 > b.y0 && a.z0 < b.z1 && a.z1 > b.z0
}

impl ChunkNode {
    fn any_empty_in_region(&self, region: &ChunkNodePos) -> bool {
        match self {
            ChunkNode::Leaf(val, pos) => {
                if !intersects(pos, region) {
                    false
                } else {
                    val.is_none()
                }
            }
            ChunkNode::Inner(a, b, pos) => {
                if !intersects(pos, region) {
                    false
                } else {
                    a.any_empty_in_region(region) || b.any_empty_in_region(region)
                }
            }
        }
    }
}

impl Chunk {
    fn is_face_visible(
        &self,
        chunk_node_pos: &ChunkNodePos,
        face: Face,
        adjacent: &AdjacentChunks,
    ) -> bool {
        // build the 1-voxel-thick neighbor "slab" touching `chunk_node_pos` on `face`.
        // if the slab is inside this chunk, query `self`. If it lies outside, query the
        // corresponding adjacent chunk (or treat as empty if missing).

        let make_region =
            |x0: usize, x1: usize, y0: usize, y1: usize, z0: usize, z1: usize| ChunkNodePos {
                x0,
                x1,
                y0,
                y1,
                z0,
                z1,
            };

        match face {
            Face::Left => {
                if chunk_node_pos.x0 > 0 {
                    let region = make_region(
                        chunk_node_pos.x0 - 1,
                        chunk_node_pos.x0,
                        chunk_node_pos.y0,
                        chunk_node_pos.y1,
                        chunk_node_pos.z0,
                        chunk_node_pos.z1,
                    );
                    self.root.any_empty_in_region(&region)
                } else {
                    // Need west neighbor; if none, it's air.
                    if let Some(west) = adjacent.west {
                        let region = make_region(
                            CHUNK_WIDTH - 1,
                            CHUNK_WIDTH,
                            chunk_node_pos.y0,
                            chunk_node_pos.y1,
                            chunk_node_pos.z0,
                            chunk_node_pos.z1,
                        );
                        west.root.any_empty_in_region(&region)
                    } else {
                        true
                    }
                }
            }
            Face::Right => {
                if chunk_node_pos.x1 < CHUNK_WIDTH {
                    let region = make_region(
                        chunk_node_pos.x1,
                        chunk_node_pos.x1 + 1,
                        chunk_node_pos.y0,
                        chunk_node_pos.y1,
                        chunk_node_pos.z0,
                        chunk_node_pos.z1,
                    );
                    self.root.any_empty_in_region(&region)
                } else {
                    if let Some(east) = adjacent.east {
                        let region = make_region(
                            0,
                            1,
                            chunk_node_pos.y0,
                            chunk_node_pos.y1,
                            chunk_node_pos.z0,
                            chunk_node_pos.z1,
                        );
                        east.root.any_empty_in_region(&region)
                    } else {
                        true
                    }
                }
            }
            Face::Back => {
                if chunk_node_pos.y1 < CHUNK_WIDTH {
                    let region = make_region(
                        chunk_node_pos.x0,
                        chunk_node_pos.x1,
                        chunk_node_pos.y1,
                        chunk_node_pos.y1 + 1,
                        chunk_node_pos.z0,
                        chunk_node_pos.z1,
                    );
                    self.root.any_empty_in_region(&region)
                } else {
                    if let Some(north) = adjacent.north {
                        let region = make_region(
                            chunk_node_pos.x0,
                            chunk_node_pos.x1,
                            0,
                            1,
                            chunk_node_pos.z0,
                            chunk_node_pos.z1,
                        );
                        north.root.any_empty_in_region(&region)
                    } else {
                        true
                    }
                }
            }
            Face::Front => {
                if chunk_node_pos.y0 > 0 {
                    let region = make_region(
                        chunk_node_pos.x0,
                        chunk_node_pos.x1,
                        chunk_node_pos.y0 - 1,
                        chunk_node_pos.y0,
                        chunk_node_pos.z0,
                        chunk_node_pos.z1,
                    );
                    self.root.any_empty_in_region(&region)
                } else {
                    if let Some(south) = adjacent.south {
                        let region = make_region(
                            chunk_node_pos.x0,
                            chunk_node_pos.x1,
                            CHUNK_WIDTH - 1,
                            CHUNK_WIDTH,
                            chunk_node_pos.z0,
                            chunk_node_pos.z1,
                        );
                        south.root.any_empty_in_region(&region)
                    } else {
                        true
                    }
                }
            }
            Face::Top => {
                if chunk_node_pos.z1 < CHUNK_HEIGHT {
                    let region = make_region(
                        chunk_node_pos.x0,
                        chunk_node_pos.x1,
                        chunk_node_pos.y0,
                        chunk_node_pos.y1,
                        chunk_node_pos.z1,
                        chunk_node_pos.z1 + 1,
                    );
                    self.root.any_empty_in_region(&region)
                } else {
                    true
                }
            }
            Face::Bottom => {
                if chunk_node_pos.z0 > 0 {
                    let region = make_region(
                        chunk_node_pos.x0,
                        chunk_node_pos.x1,
                        chunk_node_pos.y0,
                        chunk_node_pos.y1,
                        chunk_node_pos.z0 - 1,
                        chunk_node_pos.z0,
                    );
                    self.root.any_empty_in_region(&region)
                } else {
                    true
                }
            }
        }
    }
}
