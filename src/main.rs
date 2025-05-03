mod assets;
mod camera;
mod instance;
mod model;
mod state;
mod texture;
mod uniforms;

use {
    state::State,
    std::time::Instant,
    winit::{
        event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent},
        event_loop::EventLoop,
        keyboard::{KeyCode, PhysicalKey},
        window::WindowBuilder,
    },
};

async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("ft_vox")
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(&window).await;
    let mut last_render_time = Instant::now();

    event_loop
        .run(move |event, control_flow| match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } if state.mouse_pressed => state.camera_controller.process_mouse(delta.0, delta.1),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() && !state.input(event) => match event {
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
                    state.resize(*physical_size);
                }
                WindowEvent::RedrawRequested => {
                    state.window().request_redraw();
                    let now = Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;
                    state.update(dt);
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            state.resize(state.size)
                        }
                        Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                            control_flow.exit()
                        }
                        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                    }
                }
                _ => {}
            },
            _ => {}
        })
        .unwrap();
}

fn main() {
    pollster::block_on(run());
}
