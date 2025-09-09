use {
    crate::aabb::AABB,
    glam::{Mat4, Vec3},
};

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }
}

#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    pub fn from_matrix(view_proj: Mat4) -> Self {
        // use transpose to get row-major access
        let m = view_proj.transpose().to_cols_array_2d();

        // extract planes using Gribb/Hartmann method
        let planes = [
            // left plane (w + x = 0)
            Plane::new(
                Vec3::new(m[3][0] + m[0][0], m[3][1] + m[0][1], m[3][2] + m[0][2]),
                m[3][3] + m[0][3],
            ),
            // right plane (w - x = 0)
            Plane::new(
                Vec3::new(m[3][0] - m[0][0], m[3][1] - m[0][1], m[3][2] - m[0][2]),
                m[3][3] - m[0][3],
            ),
            // bottom plane (w + y = 0)
            Plane::new(
                Vec3::new(m[3][0] + m[1][0], m[3][1] + m[1][1], m[3][2] + m[1][2]),
                m[3][3] + m[1][3],
            ),
            // top plane (w - y = 0)
            Plane::new(
                Vec3::new(m[3][0] - m[1][0], m[3][1] - m[1][1], m[3][2] - m[1][2]),
                m[3][3] - m[1][3],
            ),
            // near plane (w + z = 0)
            Plane::new(
                Vec3::new(m[3][0] + m[2][0], m[3][1] + m[2][1], m[3][2] + m[2][2]),
                m[3][3] + m[2][3],
            ),
            // far plane (w - z = 0)
            Plane::new(
                Vec3::new(m[3][0] - m[2][0], m[3][1] - m[2][1], m[3][2] - m[2][2]),
                m[3][3] - m[2][3],
            ),
        ];

        // normalize planes
        let mut normalized_planes = [Plane::new(Vec3::ZERO, 0.0); 6];
        for (i, plane) in planes.iter().enumerate() {
            let length = plane.normal.length();
            normalized_planes[i] = if length > 0.0 {
                Plane::new(plane.normal / length, plane.distance / length)
            } else {
                *plane
            };
        }

        Self {
            planes: normalized_planes,
        }
    }

    pub fn intersects_aabb(&self, aabb: &AABB) -> bool {
        let center = aabb.center();
        let extents = aabb.extents();

        for plane in &self.planes {
            let positive_vertex = center
                + Vec3::new(
                    if plane.normal.x >= 0.0 {
                        extents.x
                    } else {
                        -extents.x
                    },
                    if plane.normal.y >= 0.0 {
                        extents.y
                    } else {
                        -extents.y
                    },
                    if plane.normal.z >= 0.0 {
                        extents.z
                    } else {
                        -extents.z
                    },
                );

            if plane.distance_to_point(positive_vertex) < 0.0 {
                return false;
            }
        }

        true
    }
}
