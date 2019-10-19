mod options;

use std::path::PathBuf;

use engine::{Engine, RendererType};
use log::info;
use structopt::StructOpt;

use crate::options::Options;

const SCENE_DIR: &str = "assets/viewer/scenes";

fn main() -> anyhow::Result<()> {
    let options = Options::from_args();

    flexi_logger::Logger::with_env_or_str("debug")
        .start()
        .unwrap();

    let mut engine = Engine::new("viewer", RendererType::Vulkan, &options.get_window_config())
        .unwrap_or_else(|e| panic!("Error initializing engine: {}", e));

    let mut scene = PathBuf::from(SCENE_DIR);
    scene.push(options.get_scene());

    engine
        .load_scene(scene)
        .unwrap_or_else(|e| panic!("Error loading scene: {}", e));

    engine
        .run()
        .unwrap_or_else(|e| panic!("Error running viewer: {}", e));

    info!("Done!");

    Ok(())
}
