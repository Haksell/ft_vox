use {
    crate::{
        chunk::{CHUNK_HEIGHT, CHUNK_WIDTH},
        frustum::Frustum,
        state::RENDER_DISTANCE,
        Args,
    },
    glam::{Mat4, Vec3, Vec4},
    std::f32::consts::{FRAC_PI_2, FRAC_PI_4, SQRT_2},
    winit::{event::ElementState, keyboard::KeyCode},
};

const CAMERA_MAX_OUT_OF_BOUNDS: f32 = 16.0;

const MAX_PITCH: f32 = FRAC_PI_2 * 0.99; // avoids gimbal lock

pub const CAMERA_NEAR: f32 = 0.1;

// not const because of f32::sqrt :(
pub fn camera_far() -> f32 {
    let camera_far_xy = (RENDER_DISTANCE + 1.0) * SQRT_2 * CHUNK_WIDTH as f32;
    (camera_far_xy.powi(2) + (CHUNK_HEIGHT as f32).powi(2)).sqrt()
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view_proj_skybox_inverse: [[f32; 4]; 4],
    pos: [f32; 3],
    _pad0: f32,
}
impl CameraUniform {
    pub fn new(camera: &Camera) -> Self {
        let view = camera.look_at();
        let view_skybox = camera.look_at_skybox();
        let proj = camera.projection();

        let view_proj = proj * view;
        let view_proj_skybox = proj * view_skybox;
        let view_proj_skybox_inverse = view_proj_skybox.inverse();

        Self {
            view_proj: view_proj.to_cols_array_2d(),
            view_proj_skybox_inverse: view_proj_skybox_inverse.to_cols_array_2d(),
            pos: camera.position().to_array(),
            _pad0: 0.0,
        }
    }
}

pub struct Camera {
    eye: Vec3,
    up: Vec3,
    yaw: f32,
    pitch: f32,
    aspect: f32,
    fov_y: f32,
    near: f32,
    far: f32,
    projection: Mat4,
}
impl Camera {
    pub fn new(eye: Vec3, up: Vec3, aspect: f32, fov_x: f32, near: f32, far: f32) -> Self {
        let fov_y = 2.0 * (fov_x / 2.0).tan().atan2(aspect);
        let projection = Mat4::perspective_rh(fov_y, aspect, near, far);

        Self {
            eye,
            up,
            aspect,
            yaw: 0.0,
            pitch: -FRAC_PI_4,
            fov_y,
            near,
            far,
            projection,
        }
    }

    pub fn look_at(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye, self.eye + self.direction(), self.up)
    }

    // same code as Mat4::look_to_rh but with the eye hardcoded at the origin
    pub fn look_at_skybox(&self) -> Mat4 {
        let f = self.direction();
        let s = f.cross(self.up).normalize();
        let u = s.cross(f);

        Mat4::from_cols(
            Vec4::new(s.x, u.x, -f.x, 0.0),
            Vec4::new(s.y, u.y, -f.y, 0.0),
            Vec4::new(s.z, u.z, -f.z, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn projection(&self) -> Mat4 {
        self.projection
    }

    pub fn direction(&self) -> Vec3 {
        Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
        )
        .normalize()
    }

    pub fn position(&self) -> Vec3 {
        self.eye
    }

    pub fn get_frustum(&self) -> Frustum {
        let view_proj = self.projection() * self.look_at();
        Frustum::from_matrix(view_proj)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
        self.projection = Mat4::perspective_rh(self.fov_y, self.aspect, self.near, self.far);
    }
}

pub struct CameraController {
    normal_speed: f32,
    boosted_speed: f32,
    sensitivity: f32,
    is_boosted: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    mouse_delta: (f32, f32),
}
impl CameraController {
    pub fn new(args: &Args) -> Self {
        Self {
            normal_speed: args.normal_speed,
            boosted_speed: args.boosted_speed,
            sensitivity: 0.004,
            is_boosted: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            mouse_delta: (0.0, 0.0),
        }
    }

    pub fn process_mouse_motion(&mut self, delta_x: f32, delta_y: f32) {
        self.mouse_delta.0 += delta_x;
        self.mouse_delta.1 += delta_y;
    }

    pub fn process_boost(&mut self, is_pressed: bool) {
        self.is_boosted = is_pressed;
    }

    pub fn process_keyboard(&mut self, state: ElementState, keycode: KeyCode) {
        let is_pressed = state == ElementState::Pressed;
        match keycode {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
            }
            KeyCode::Space => {
                self.is_up_pressed = is_pressed;
            }
            KeyCode::ShiftLeft | KeyCode::ControlLeft => {
                self.is_down_pressed = is_pressed;
            }
            _ => {}
        }
    }

    pub fn update(&mut self, camera: &mut Camera, dt: f32) {
        // === ROTATION ===
        let (dx, dy) = self.mouse_delta;
        camera.yaw += dx * self.sensitivity;
        camera.pitch = (camera.pitch - dy * self.sensitivity).clamp(-MAX_PITCH, MAX_PITCH);
        self.mouse_delta = (0.0, 0.0);

        // === MOVEMENT ===
        let forward = camera.direction();
        let right = forward.cross(camera.up);

        let mut movement = Vec3::ZERO;
        movement += forward * (self.is_forward_pressed as i32) as f32;
        movement -= forward * (self.is_backward_pressed as i32) as f32;
        movement += right * (self.is_right_pressed as i32) as f32;
        movement -= right * (self.is_left_pressed as i32) as f32;
        movement += camera.up * (self.is_up_pressed as i32) as f32;
        movement -= camera.up * (self.is_down_pressed as i32) as f32;

        if movement == Vec3::ZERO {
            return;
        }

        camera.eye += movement.normalize() * self.speed() * dt;
        camera.eye.z = camera.eye.z.clamp(
            -CAMERA_MAX_OUT_OF_BOUNDS,
            CHUNK_HEIGHT as f32 + CAMERA_MAX_OUT_OF_BOUNDS,
        );
    }

    #[inline]
    fn speed(&self) -> f32 {
        if self.is_boosted {
            self.boosted_speed
        } else {
            self.normal_speed
        }
    }
}
