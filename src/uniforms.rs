use {
    crate::camera::{Camera, Projection},
    bytemuck::{Pod, Zeroable},
    cgmath::{Matrix4, Quaternion, Rad, Rotation3 as _, SquareMatrix as _, Vector3},
    std::{f32::consts::TAU, time::Duration},
};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    view_pos: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_pos: [0.0; 4],
            view_proj: Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_pos = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    _padding1: u32,
    color: [f32; 3],
    _padding2: u32,
}

impl LightUniform {
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position,
            _padding1: 0,
            color,
            _padding2: 0,
        }
    }

    pub fn update_position(&mut self, dt: Duration) {
        let old_position: Vector3<_> = self.position.into();
        self.position =
            (Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), Rad(TAU * dt.as_secs_f32()))
                * old_position)
                .into();
    }
}
