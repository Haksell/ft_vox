mod aabb;
mod app;
mod biome;
mod block;
mod camera;
mod chunk;
mod face;
mod frustum;
mod noise;
mod spline;
mod state;
mod texture;
mod utils;
mod vertex;
mod world;

use {
    crate::app::Application,
    clap::{arg, command, Parser},
    winit::event_loop::EventLoop,
};

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(long = "no-fullscreen", action = clap::ArgAction::SetFalse)]
    fullscreen: bool,
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long, default_value_t = 1.0)]
    normal_speed: f32,
    #[arg(long, default_value_t = 20.0)]
    boosted_speed: f32,
}

async fn run(args: Args) {
    env_logger::init();
    log::info!("Running {} with {:?}", env!("CARGO_CRATE_NAME"), args);
    let event_loop = EventLoop::new().unwrap();
    let mut app = Application::new(args);
    event_loop.run_app(&mut app).expect("Failed to run app");
}

fn main() {
    let args = Args::parse();
    let _ = pollster::block_on(run(args));
}
