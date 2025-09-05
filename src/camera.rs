use {
    crate::frustum::Frustum,
    winit::{
        event::{ElementState, KeyEvent, WindowEvent},
        keyboard::{KeyCode, PhysicalKey},
    },
};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        let view = camera.look_at();
        let proj = camera.projection();

        self.view_proj = (proj * view).to_cols_array_2d();
    }
}

pub struct Camera {
    eye: glam::Vec3,
    up: glam::Vec3,
    yaw: f32,
    pitch: f32,
    aspect: f32,
    fovy: f32,
    near: f32,
    far: f32,
}

impl Camera {
    pub fn new(
        eye: glam::Vec3,
        up: glam::Vec3,
        aspect: f32,
        fovy: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            eye,
            up,
            aspect,
            yaw: -90.0,
            pitch: 0.0,
            fovy,
            near,
            far,
        }
    }

    pub fn look_at(&self) -> glam::Mat4 {
        let forward = self.direction();
        let target = self.eye + forward;

        glam::Mat4::look_at_rh(self.eye, target, self.up)
    }

    pub fn projection(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(self.fovy.to_radians(), self.aspect, self.near, self.far)
    }

    pub fn direction(&self) -> glam::Vec3 {
        glam::Vec3::new(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        )
        .normalize()
    }

    pub fn position(&self) -> glam::Vec3 {
        self.eye
    }

    pub fn get_frustum(&self) -> Frustum {
        let view_proj = self.projection() * self.look_at();
        Frustum::from_matrix(view_proj)
    }

    pub fn set_aspect_ratio(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}

pub struct CameraController {
    speed: f32,
    sensitivity: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    mouse_delta: (f32, f32),
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            mouse_delta: (0.0, 0.0),
        }
    }

    pub fn process_mouse(&mut self, delta_x: f32, delta_y: f32) -> bool {
        self.mouse_delta.0 += delta_x;
        self.mouse_delta.1 += delta_y;

        true
    }

    pub fn process_keyboard(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update(&mut self, camera: &mut Camera, dt: f32) {
        // Apply rotation from mouse movement
        if self.mouse_delta.0 != 0.0 || self.mouse_delta.1 != 0.0 {
            // Adjust yaw and pitch based on mouse movement
            camera.yaw += self.mouse_delta.0 * self.sensitivity;
            camera.pitch -= self.mouse_delta.1 * self.sensitivity; // Invert Y for natural feel

            // Clamp pitch to avoid gimbal lock
            camera.pitch = camera.pitch.clamp(-89.0, 89.0);

            // Reset mouse delta
            self.mouse_delta = (0.0, 0.0);
        }

        // Calculate movement directions
        let forward = camera.direction();
        let right = forward.cross(camera.up);

        // Calculate movement
        let mut movement = glam::Vec3::ZERO;

        if self.is_forward_pressed {
            movement += forward;
        }
        if self.is_backward_pressed {
            movement -= forward;
        }
        if self.is_right_pressed {
            movement += right;
        }
        if self.is_left_pressed {
            movement -= right;
        }

        // Normalize movement vector if not zero and apply speed
        if movement != glam::Vec3::ZERO {
            movement = movement.normalize() * self.speed * dt;
            camera.eye += movement;
        }
    }
}
