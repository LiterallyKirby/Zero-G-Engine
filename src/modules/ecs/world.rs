
use crate::modules::ecs::entity::Entity;

pub struct World {
    pub entities: Vec<Entity>,
}

impl World {
    pub fn new() -> Self {
        Self { entities: vec![] }
    }
}

