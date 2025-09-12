use {
    crate::{
        chunk::CHUNK_WIDTH,
        coords::{camera_to_chunk_coords, split_coords, ChunkCoords},
        state::State,
        world::World,
        Args,
    },
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
    args: Args,
    window_attributes: WindowAttributes,
    window: Option<Arc<Window>>,
    state: Option<State<'a>>,
    world: World,
    last_chunk: Option<ChunkCoords>,
    last_render: Instant,
    last_fps_log: Instant,
    frames_since_log: u32,
}

impl<'a> Application<'a> {
    pub fn new(args: Args) -> Self {
        let mut window_attributes = Window::default_attributes()
            .with_title("ft_vox")
            .with_resizable(true)
            .with_inner_size(PhysicalSize::new(1280.0, 720.0));
        if args.fullscreen {
            window_attributes =
                window_attributes.with_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        Self {
            window_attributes,
            window: None,
            state: None,
            world: World::new(args.seed),
            last_chunk: None,
            last_render: Instant::now(),
            last_fps_log: Instant::now(),
            frames_since_log: 0,
            args,
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
        let state = pollster::block_on(State::new(window.clone(), &self.args));

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
                        if let Some((world_coords, block)) =
                            self.world.delete_center_block(&state.camera)
                        {
                            log::debug!("Deleted {:?}", block);

                            let ((cx, cy), (bx, by, _)) = split_coords(world_coords).unwrap();

                            state.chunks_to_rerender.insert((cx, cy));

                            // rerender neighbors if on the edge
                            if bx == 0 {
                                state.chunks_to_rerender.insert((cx - 1, cy));
                            } else if bx == CHUNK_WIDTH - 1 {
                                state.chunks_to_rerender.insert((cx + 1, cy));
                            }
                            if by == 0 {
                                state.chunks_to_rerender.insert((cx, cy - 1));
                            } else if by == CHUNK_WIDTH - 1 {
                                state.chunks_to_rerender.insert((cx, cy + 1));
                            }
                        }
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
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::KeyF),
                        ..
                    },
                ..
            } => {
                state.toggle_show_fps();
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

                state.update(dt);

                state.rerender_chunks(&mut self.world);

                let camera_chunk = camera_to_chunk_coords(state.camera.position());
                if self.last_chunk != Some(camera_chunk) {
                    self.last_chunk = Some(camera_chunk);
                    state.update_chunks(&mut self.world);
                }

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
