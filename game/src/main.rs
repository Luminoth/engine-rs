use failure::Error;

use engine::{Engine, RendererType};

fn main() -> Result<(), Error> {
    let mut engine = Engine::new("engine-rs", RendererType::Vulkan)
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
