use crate::utils::lerp;

#[derive(Debug, Clone)]
pub struct SplinePoint {
    pub x: f32, // Input
    pub y: f32, // Output
}

impl SplinePoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone)]
pub struct Spline {
    points: Vec<SplinePoint>,
}

impl Spline {
    pub fn new(points: Vec<SplinePoint>) -> Self {
        let mut spline = Self { points };
        spline.sort_points();
        spline
    }

    fn sort_points(&mut self) {
        self.points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    }

    pub fn sample(&self, x: f32) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }

        if self.points.len() == 1 {
            return self.points[0].y;
        }

        if x <= self.points[0].x {
            return self.points[0].y;
        }

        if x >= self.points.last().unwrap().x {
            return self.points.last().unwrap().y;
        }

        for i in 0..self.points.len() - 1 {
            let p1 = &self.points[i];
            let p2 = &self.points[i + 1];

            if x >= p1.x && x <= p2.x {
                let t = (x - p1.x) / (p2.x - p1.x);
                return lerp(p1.y, p2.y, t);
            }
        }

        self.points.last().unwrap().y
    }
}
