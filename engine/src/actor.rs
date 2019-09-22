use ecs::Entity;

pub struct Actor {
    entity: Entity,
}

impl Actor {
    pub fn new() -> Self {
        Self {
            entity: Entity::new(),
        }
    }
}
