use crate::modules::ecs::components::*;
use crate::modules::ecs::entity::Entity;
use crate::modules::ecs::entity::*;
use slotmap::{SlotMap, new_key_type};
use std::collections::HashMap;
new_key_type! { pub struct EntityKey; }
pub type EntityId = EntityKey;

// Tag interned as u32
pub type TagId = u32;

pub struct TagRegistry {
    map: HashMap<String, TagId>,
    next: TagId,
}

impl TagRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            next: 0,
        }
    }

    fn intern(&mut self, tag: &str) -> TagId {
        *self.map.entry(tag.to_string()).or_insert_with(|| {
            let id = self.next;
            self.next += 1;
            id
        })
    }

    pub fn get_or_create(&mut self, name: &str) -> u32 {
        if let Some(&id) = self.map.get(name) {
            id
        } else {
            let id = self.next;
            self.map.insert(name.to_string(), id);
            self.next += 1;
            id
        }
    }
}

pub struct World {
    entities: SlotMap<EntityId, Entity>,
    tags: TagRegistry,
    tag_index: HashMap<TagId, Vec<EntityId>>, // speeds up queries
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: SlotMap::with_key(),
            tags: TagRegistry::new(),
            tag_index: HashMap::new(),
        }
    }

    pub fn active_camera_matrix(&self, aspect: f32) -> Option<glam::Mat4> {
        for (_, entity) in &self.entities {
            if let (Some(cam), Some(t)) = (&entity.camera, &entity.transform) {
                if cam.is_active {
                    return Some(camera_view_proj(cam, t, aspect));
                }
            }
        }
        None
    }

    pub fn create_entity(&mut self, name: impl Into<String>) -> EntityId {
        let entity_id = self.entities.insert(Entity::new(name));
        entity_id
    }

    pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(id)
    }

    pub fn get_renderable_entities(&self) -> Vec<(u32, Transform, Material)> {
        self.entities
            .values()
            .filter_map(|entity| {
                if let (Some(mesh_handle), Some(material), Some(transform)) =
                    (&entity.mesh_handle, &entity.material, &entity.transform)
                {
                    Some((mesh_handle.0, *transform, *material))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_renderable_entities_with_ids(&self) -> Vec<(EntityId, u32, Transform, Material)> {
        self.entities
            .iter()
            .filter_map(|(entity_id, entity)| {
                if let (Some(mesh_handle), Some(material), Some(transform)) =
                    (&entity.mesh_handle, &entity.material, &entity.transform)
                {
                    Some((entity_id, mesh_handle.0, *transform, *material))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(id)
    }

    pub fn get_entities_with_tag(&self, tag: &str) -> Vec<EntityId> {
        // Fast path using tag index if available
        if let Some(tag_id) = self.tags.map.get(tag) {
            if let Some(entity_ids) = self.tag_index.get(tag_id) {
                return entity_ids.clone();
            }
        }

        // Fallback: iterate through entities
        let mut entity_list = Vec::new();
        for (id, entity) in &self.entities {
            if entity.has_tag(tag) {
                entity_list.push(id);
            }
        }
        entity_list
    } 

    pub fn remove_tag_from_entity(&mut self, entity_id: EntityId, tag: &str) {
        if let Some(entity) = self.entities.get_mut(entity_id) {
            entity.remove_tag(tag);

            // Update tag index
            if let Some(tag_id) = self.tags.map.get(tag) {
                if let Some(entity_ids) = self.tag_index.get_mut(tag_id) {
                    entity_ids.retain(|&id| id != entity_id);
                }
            }
        }
    }

    pub fn remove_entity(&mut self, id: EntityId) -> Option<Entity> {
        if let Some(entity) = self.entities.remove(id) {
            // Clean up tag index
            for tag in &entity.tags {
                if let Some(tag_id) = self.tags.map.get(tag) {
                    if let Some(entity_ids) = self.tag_index.get_mut(tag_id) {
                        entity_ids.retain(|&eid| eid != id);
                    }
                }
            }
            Some(entity)
        } else {
            None
        }
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn iter_entities(&self) -> impl Iterator<Item = (EntityId, &Entity)> {
        self.entities.iter()
    }

    pub fn iter_entities_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut Entity)> {
        self.entities.iter_mut()
    }
}
