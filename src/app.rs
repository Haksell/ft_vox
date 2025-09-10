use {
    crate::{chunk::ChunkCoords, state::State, world::World},
    std::{
        sync::Arc,
        time::{Duration, Instant},
    },
    winit::{
        application::ApplicationHandler,
        dpi::{PhysicalPosition, PhysicalSize},
        event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent},
        event_loop::ActiveEventLoop,
        keyboard::{KeyCode, PhysicalKey},
        window::{Fullscreen, Window, WindowAttributes, WindowId},
    },
};

pub struct Application<'a> {
    window_attributes: WindowAttributes,
    state: Option<State<'a>>,
    window: Option<Arc<Window>>,
    world: World,
    last_chunk: Option<ChunkCoords>,
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
}

impl<'a> ApplicationHandler for Application<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(self.window_attributes.clone())
                .unwrap(),
        );
        window.set_cursor_visible(false);
        let state = pollster::block_on(State::new(window.clone()));

        self.window = Some(window);
        self.state = Some(state);
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        let state = self.state.as_mut().unwrap();
        let camera_controller = &mut state.camera_controller;
        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                camera_controller.process_mouse_motion(dx as f32, dy as f32);
                state.update_crosshair(&self.world);
            }
            DeviceEvent::Button {
                button,
                state: button_state,
            } => match button {
                1 => camera_controller.process_boost(button_state.is_pressed()),
                3 => {
                    if button_state.is_pressed() {
                        state.is_right_clicking = true;
                        state.update_crosshair(&self.world);
                    } else {
                        state.is_right_clicking = false;
                        state.is_crosshair_active = false;
                        let block = self.world.delete_center_block(&state.camera);
                        println!("{:?}", block);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = self.state.as_mut().unwrap();
        let window = self.window.as_mut().unwrap();

        if window_id != window.id() {
            return;
        }

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
            } => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: element_state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                state
                    .camera_controller
                    .process_keyboard(element_state, keycode);
                state.update_crosshair(&self.world);
            }
            WindowEvent::Resized(physical_size) => {
                log::info!("physical_size: {physical_size:?}");
                state.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now - self.last_render;
                if dt > Duration::from_millis(100) {
                    log::warn!("frame took {}ms to generate", dt.as_millis());
                }
                self.last_render = now;

                let camera_pos = state.camera.position();
                let camera_chunk = self
                    .world
                    .get_chunk_index_from_position(camera_pos.x, camera_pos.y);

                if self.last_chunk != Some(camera_chunk) {
                    self.last_chunk = Some(camera_chunk);
                    state.update_chunks(&mut self.world);
                }

                state.update(dt);

                // reset cursor to center (TODO: only when not fullscreen)
                let size = window.inner_size();
                let center = PhysicalPosition::new(size.width / 2, size.height / 2);
                window.set_cursor_position(center).unwrap();

                match state.render() {
                    Ok(_) => {
                        self.frames_since_log += 1;
                        let elapsed = self.last_fps_log.elapsed();
                        if elapsed >= Duration::from_secs(1) {
                            let secs = elapsed.as_secs_f64();
                            state.fps = self.frames_since_log as f32 / secs as f32;
                            log::info!("FPS: {:.1} | CHUNK: {:?}", state.fps, camera_chunk);
                            self.frames_since_log = 0;
                            self.last_fps_log = Instant::now();
                        }
                    }
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                        log::error!("OutOfMemory");
                        event_loop.exit();
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
