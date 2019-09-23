use ecs::Entity;

pub struct Actor {
    entity: Entity,
}

impl Actor {
    pub fn new<S>(id: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            entity: Entity::new(id),
        }
    }

    pub fn get_entity(&self) -> &Entity {
        &self.entity
    }
}
