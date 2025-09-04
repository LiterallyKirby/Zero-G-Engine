
use crate::modules::ecs::entity::Entity;
use crate::modules::ecs::entity::*;

pub struct World {
    pub entities: Vec<Entity>,
    next_id: u64,
}



impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            next_id: 0,
        }
    }
pub fn active_camera_matrix(&self, aspect: f32) -> Option<glam::Mat4> {
        for entity in &self.entities {
            if let (Some(cam), Some(t)) = (&entity.camera, &entity.transform) {
                if cam.is_active {
                    return Some(camera_view_proj(cam, t, aspect));
                }
            }
        }
        None
    }
    pub fn create_entity(&mut self, name: impl Into<String>) -> &mut Entity {
        let entity = Entity::new(EntityId(self.next_id), name);
        self.next_id += 1;
        self.entities.push(entity);
        self.entities.last_mut().unwrap()
    }

    pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id == id)
    }

    pub fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id == id)
    }
}
