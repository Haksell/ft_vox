use crate::aabb::AABB;

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: glam::Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: glam::Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    pub fn distance_to_point(&self, point: glam::Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }
}

#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    pub fn from_matrix(view_proj: glam::Mat4) -> Self {
        let m = view_proj.to_cols_array_2d();

        // Extract planes using Gribb/Hartmann method
        let planes = [
            // Left plane
            Plane::new(
                glam::Vec3::new(m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0]),
                m[3][3] + m[3][0],
            ),
            // Right plane
            Plane::new(
                glam::Vec3::new(m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0]),
                m[3][3] - m[3][0],
            ),
            // Top plane
            Plane::new(
                glam::Vec3::new(m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1]),
                m[3][3] - m[3][1],
            ),
            // Bottom plane
            Plane::new(
                glam::Vec3::new(m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1]),
                m[3][3] + m[3][1],
            ),
            // Near plane
            Plane::new(
                glam::Vec3::new(m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2]),
                m[3][3] + m[3][2],
            ),
            // Far plane
            Plane::new(
                glam::Vec3::new(m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2]),
                m[3][3] - m[3][2],
            ),
        ];

        let mut normalized_planes = [Plane::new(glam::Vec3::ZERO, 0.0); 6];
        for (i, plane) in planes.iter().enumerate() {
            let length = plane.normal.length();
            normalized_planes[i] = Plane::new(plane.normal / length, plane.distance / length);
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
                + glam::Vec3::new(
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
