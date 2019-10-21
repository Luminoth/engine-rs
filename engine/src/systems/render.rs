use log::warn;
use specs::prelude::*;

use crate::components::TransformComponent;

pub(crate) struct RenderSystem;

impl<'a> System<'a> for RenderSystem {
    type SystemData = (ReadStorage<'a, TransformComponent>);

    fn run(&mut self, (transform): Self::SystemData) {
        for (transform) in (&transform).join() {
            warn!("render");
        }
    }
}
