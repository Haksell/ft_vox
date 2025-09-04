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
}
