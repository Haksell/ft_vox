#[derive(Debug, Clone, Copy)]
pub enum Face {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

pub const FACES: [Face; 6] = [
    Face::Top,
    Face::Bottom,
    Face::Left,
    Face::Right,
    Face::Front,
    Face::Back,
];

impl Face {
    pub fn normal(&self) -> (i32, i32, i32) {
        match self {
            Face::Top => (0, 0, 1),
            Face::Bottom => (0, 0, -1),
            Face::Left => (-1, 0, 0),
            Face::Right => (1, 0, 0),
            Face::Front => (0, -1, 0),
            Face::Back => (0, 1, 0),
        }
    }

    pub fn positions(&self) -> [[f32; 3]; 4] {
        match self {
            Face::Right => [
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
                [1.0, 0.0, 1.0],
            ],
            Face::Left => [
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            Face::Back => [
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
            ],
            Face::Front => [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ],
            Face::Top => [
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            Face::Bottom => [
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
        }
    }

    pub fn uvs(&self) -> [[f32; 2]; 4] {
        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]
    }
}
