use failure::Error;

use engine::Engine;

// http://vulkano.rs/guide/buffer-creation

fn main() -> Result<(), Error> {
    let mut engine =
        Engine::new("engine-rs").unwrap_or_else(|e| panic!("Error initializing engine: {}", e));

    println!("Loading shaders...");
    let (vs, fs) = engine
        .get_renderer()
        .load_simple_shader()
        .unwrap_or_else(|e| panic!("Error loading simple shader: {}", e));

    println!("Creating render pass...");
    let render_pass = engine
        .get_renderer()
        .create_render_pass()
        .unwrap_or_else(|e| panic!("Error creating render pass: {}", e));

    println!("Creating pipeline...");
    let pipeline = engine
        .get_renderer()
        .create_pipeline(render_pass, vs, fs)
        .unwrap_or_else(|e| panic!("Error creating pipeline: {}", e));

    engine.run();

    println!("Done!");

    Ok(())
}
