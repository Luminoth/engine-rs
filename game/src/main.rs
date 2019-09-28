use failure::Error;

use engine::Engine;

// http://vulkano.rs/guide/buffer-creation

fn main() -> Result<(), Error> {
    let mut engine =
        Engine::new("engine-rs").unwrap_or_else(|e| panic!("Error initializing engine: {}", e));

    // TODO: this is probably not even necessary
    engine
        .load_scene()
        .unwrap_or_else(|e| panic!("Error loading scene: {}", e));

    engine
        .run()
        .unwrap_or_else(|e| panic!("Error running game: {}", e));

    println!("Done!");

    Ok(())
}
