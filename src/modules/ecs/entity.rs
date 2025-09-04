use crate::modules::ecs::components::*;
use crate::modules::ecs::world::*;

pub struct Entity {
    pub transform: Option<Transform>,
    pub mesh_handle: Option<MeshHandle>,
    pub material: Option<Material>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            transform: None,
            mesh_handle: None,
            material: None,
        }
    }
    
    pub fn add_transform(&mut self, t: Transform) { 
        self.transform = Some(t); 
    }
    
    pub fn add_mesh_handle(&mut self, m: MeshHandle) { 
        self.mesh_handle = Some(m); 
    }
    
    pub fn add_material(&mut self, mat: Material) { 
        self.material = Some(mat); 
    }
}

pub enum MeshType {
    Triangle,
    Cube,
    Custom(u32), // let the user pass a GPU mesh ID directly
}

pub fn spawn_triangle(world: &mut World) -> usize {
    let mut entity = Entity::new();
    entity.add_transform(Transform {
        position: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    });
    entity.add_mesh_handle(MeshHandle(0)); // 0 = triangle mesh id
    entity.add_material(Material {
        color: [1.0, 0.0, 0.0, 1.0],
    });
    world.entities.push(entity);
    world.entities.len() - 1
}

pub fn spawn_entity(
    world: &mut World,
    position: [f32; 3],
    scale: [f32; 3],
    color: [f32; 4],
    mesh: MeshType,
) -> usize {
    let mut entity = Entity::new();
    entity.add_transform(Transform {
        position,
        rotation: [0.0, 0.0, 0.0],
        scale,
    });
    
    // Choose mesh ID based on enum
    let mesh_id = match mesh {
        MeshType::Triangle => 0,
        MeshType::Cube => 1,
        MeshType::Custom(id) => id,
    };
    
    entity.add_mesh_handle(MeshHandle(mesh_id));
    entity.add_material(Material { color });
    world.entities.push(entity);
    world.entities.len() - 1
}

// Helper functions for common entity creation
pub fn spawn_cube(world: &mut World, position: [f32; 3], color: [f32; 4]) -> usize {
    spawn_entity(
        world, 
        position, 
        [1.0, 1.0, 1.0], // default scale
        color, 
        MeshType::Cube
    )
}

pub fn spawn_triangle_at(world: &mut World, position: [f32; 3], color: [f32; 4]) -> usize {
    spawn_entity(
        world,
        position,
        [1.0, 1.0, 1.0], // default scale
        color,
        MeshType::Triangle
    )
}
