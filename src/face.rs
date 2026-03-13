#[derive(Debug, Clone, Copy)]
pub enum Face {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

impl Face {
    pub const ALL: [Self; 6] = [
        Self::Top,
        Self::Bottom,
        Self::Left,
        Self::Right,
        Self::Front,
        Self::Back,
    ];

    pub const fn normal(&self) -> [f32; 3] {
        match self {
            Self::Top => [0., 0., 1.],
            Self::Bottom => [0., 0., -1.],
            Self::Left => [-1., 0., 0.],
            Self::Right => [1., 0., 0.],
            Self::Front => [0., -1., 0.],
            Self::Back => [0., 1., 0.],
        }
    }

    pub const fn positions(&self) -> [[f32; 3]; 4] {
        match self {
            Self::Right => [
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
                [1.0, 0.0, 1.0],
            ],
            Self::Left => [
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            Self::Back => [
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
            ],
            Self::Front => [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ],
            Self::Top => [
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            Self::Bottom => [
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
        }
    }

    pub const fn uvs(&self, (sx, sy, sz): (usize, usize, usize)) -> [[f32; 2]; 4] {
        let (sx, sy, sz) = (sx as f32, sy as f32, sz as f32);
        match self {
            Self::Top | Self::Bottom => [[0.0, sy], [sx, sy], [sx, 0.0], [0.0, 0.0]],
            Self::Left | Self::Right => [[0.0, sz], [sy, sz], [sy, 0.0], [0.0, 0.0]],
            Self::Front | Self::Back => [[0.0, sz], [sx, sz], [sx, 0.0], [0.0, 0.0]],
        }
    }
}
