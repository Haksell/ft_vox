use {
    crate::{world::World, State},
    std::{
        sync::Arc,
        time::{Duration, Instant},
    },
    winit::{
        application::ApplicationHandler,
        dpi::PhysicalSize,
        event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent},
        event_loop::ActiveEventLoop,
        keyboard::{KeyCode, PhysicalKey},
        window::{Fullscreen, Window, WindowAttributes},
    },
};

pub struct Application<'a> {
    window_attributes: WindowAttributes,
    state: Option<State<'a>>,
    window: Option<Arc<Window>>,
    world: World,
    last_chunk: Option<(i32, i32)>,
    last_render: Instant,
    last_fps_log: Instant,
    frames_since_log: u32,
}

impl<'a> Application<'a> {
    pub fn new() -> Self {
        Self {
            window_attributes: Window::default_attributes()
                .with_title("ft_vox")
                .with_resizable(true)
                .with_inner_size(PhysicalSize::new(1280.0, 720.0)),
            state: None,
            window: None,
            world: World::new(42),
            last_chunk: None,
            last_render: Instant::now(),
            last_fps_log: Instant::now(),
            frames_since_log: 0,
        }
    }

    pub fn state_from_window(window: Arc<Window>) -> State<'static> {
        pollster::block_on(State::new(window))
    }
}

impl<'a> ApplicationHandler for Application<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(self.window_attributes.clone())
                .unwrap(),
        );
        window.set_cursor_visible(false);
        let state = Application::state_from_window(window.clone());

        self.window = Some(window);
        self.state = Some(state);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            self.state
                .as_mut()
                .unwrap()
                .camera_controller
                .process_mouse(dx as f32, dy as f32);
        }
    }

    fn window_event(
        &mut self,
        control_flow: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = self.state.as_mut().unwrap();
        let window = self.window.as_mut().unwrap();

        if window_id == window.id() && !state.input(&event) {
            match event {
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::F11),
                            ..
                        },
                    ..
                } => {
                    let monitor = window.current_monitor().unwrap();
                    match window.fullscreen() {
                        Some(_) => window.set_fullscreen(None),
                        None => window.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor)))),
                    }
                }
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => control_flow.exit(),
                WindowEvent::Resized(physical_size) => {
                    log::info!("physical_size: {physical_size:?}");
                    state.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    let now = Instant::now();
                    let dt = now - self.last_render;
                    self.last_render = now;

                    let camera_pos = state.camera.position();
                    let current_chunk = State::world_to_chunk_coords(camera_pos.x, camera_pos.z);

                    if self.last_chunk != Some(current_chunk) {
                        self.last_chunk = Some(current_chunk);
                        state.update_chunks(&mut self.world);
                    }

                    state.update(dt);

                    match state.render() {
                        Ok(_) => {
                            self.frames_since_log += 1;
                            let elapsed = self.last_fps_log.elapsed();
                            if elapsed >= Duration::from_secs(1) {
                                let secs = elapsed.as_secs_f64();
                                let fps = self.frames_since_log as f64 / secs;
                                log::info!("FPS: {:.1}", fps);
                                self.frames_since_log = 0;
                                self.last_fps_log = Instant::now();
                            }
                        }
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            state.resize(state.size)
                        }
                        Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                            log::error!("OutOfMemory");
                            control_flow.exit();
                        }
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("Surface timeout")
                        }
                    }

                    window.request_redraw();
                }
                _ => {}
            }
        }
    }
}
