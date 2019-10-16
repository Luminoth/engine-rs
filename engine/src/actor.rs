//use specs::prelude::*;

pub struct Actor {
    //entity: Entity,
}

impl Actor {
    pub fn new<S>(id: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            //entity: Entity::new(),
        }
    }

    /*pub fn get_entity(&self) -> &Entity {
        &self.entity
    }*/
}
