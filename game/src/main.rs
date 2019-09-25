use failure::Error;

use engine::Engine;

// http://vulkano.rs/guide/buffer-creation

fn main() -> Result<(), Error> {
    let mut engine =
        Engine::new("engine-rs").unwrap_or_else(|e| panic!("Error initializing engine: {}", e));

    println!("Loading shaders...");
    let (vs, fs) = engine
        .get_renderer()
        .read()
        .load_simple_shader()
        .unwrap_or_else(|e| panic!("Error loading simple shader: {}", e));

    let render_pass = engine
        .get_renderer()
        .read()
        .create_simple_render_pass()
        .unwrap_or_else(|e| panic!("Error creating render pass: {}", e));

    engine
        .get_renderer()
        .write()
        .create_frame_buffers(render_pass.clone());

    engine
        .run()
        .unwrap_or_else(|e| panic!("Error running game: {}", e));

    println!("Done!");

    Ok(())
}
