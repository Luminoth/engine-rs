use failure::Error;

use engine::Engine;

// http://vulkano.rs/guide/buffer-creation

fn main() -> Result<(), Error> {
    let mut engine =
        Engine::new("engine-rs").unwrap_or_else(|e| panic!("Error initializing engine: {}", e));

    println!("Loading shaders...");
    engine
        .get_renderer()
        .load_simple_shader()
        .unwrap_or_else(|e| panic!("Error loading simple shader: {}", e));

    engine.run();

    println!("Done!");

    Ok(())
}
