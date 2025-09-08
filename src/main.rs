mod aabb;
mod app;
mod biome;
mod block;
mod camera;
mod chunk;
mod face;
mod frustum;
mod noise;
mod state;
mod texture;
mod utils;
mod vertex;
mod world;

use {crate::app::Application, winit::event_loop::EventLoop};

async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = Application::new();
    event_loop.run_app(&mut app).expect("Failed to run app");
}

fn main() {
    let _ = pollster::block_on(run());
}
