use failure::Error;

use engine::{Engine, RendererType};

fn main() -> Result<(), Error> {
    let window_config = engine::config::WindowConfig {
        width: 1024,
        height: 768,
        fullscreen: false,
    };

    let mut engine = Engine::new("engine-rs", RendererType::Vulkan, &window_config)
        .unwrap_or_else(|e| panic!("Error initializing engine: {}", e));

    engine
        .load_scene()
        .unwrap_or_else(|e| panic!("Error loading scene: {}", e));

    engine
        .run()
        .unwrap_or_else(|e| panic!("Error running game: {}", e));

    println!("Done!");

    Ok(())
}
