use crate::modules::ecs::components::*;
use crate::modules::ecs::world::*;

// ============================================================================
// CORE TYPES AND STRUCTS
// ============================================================================

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

pub struct Camera {
    pub fov: f32,        // Field of view in degrees
    pub near: f32,       // Near clipping plane
    pub far: f32,        // Far clipping plane
    pub is_active: bool, // Only one camera can be active at a time
}

pub struct Entity {
    pub id: EntityId,
    pub name: String,
    pub transform: Option<Transform>,
    pub mesh_handle: Option<MeshHandle>,
    pub material: Option<Material>,
    pub camera: Option<Camera>, // new optional component
    pub tags: Vec<String>,
}

pub enum MeshType {
    Triangle,
    Cube,
    Custom(u32), // let the user pass a GPU mesh ID directly
}

// ============================================================================
// ENTITY IMPLEMENTATION
// ============================================================================

impl Entity {
    pub fn new(id: EntityId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            transform: None,
            mesh_handle: None,
            material: None,
            camera: None,
            tags: Vec::new(),
        }
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn add_camera(&mut self, camera: Camera) {
        self.camera = Some(camera);
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

// ============================================================================
// ENTITY SPAWNING FUNCTIONS
// ============================================================================

pub fn spawn_entity(
    world: &mut World,
    name: impl Into<String>,
    position: [f32; 3],
    scale: [f32; 3],
    mesh: MeshType,
    color: [f32; 4],
) -> EntityId {
    // Create entity through World to get unique ID
    let entity = world.create_entity(name);

    // Add transform
    entity.add_transform(Transform {
        position,
        rotation: [0.0, 0.0, 0.0],
        scale,
    });

    // Pick mesh ID from enum
    let mesh_id = match mesh {
        MeshType::Triangle => 0,
        MeshType::Cube => 1,
        MeshType::Custom(id) => id,
    };
    entity.add_mesh_handle(MeshHandle(mesh_id));

    // Add material
    entity.add_material(Material { color });

    // Return the entity's ID
    entity.id
}

pub fn spawn_triangle(world: &mut World, name: impl Into<String>) -> EntityId {
    // Create a new entity through the World (assigns unique ID)
    let entity = world.create_entity(name); // returns &mut Entity

    // Add components
    entity.add_transform(Transform {
        position: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    });
    entity.add_mesh_handle(MeshHandle(0)); // triangle mesh
    entity.add_material(Material {
        color: [1.0, 0.0, 0.0, 1.0],
    });

    // Return a copy of the entity's ID
    entity.id.clone()
}

pub fn spawn_camera(
    world: &mut World,
    name: impl Into<String>,
    position: [f32; 3],
    rotation: [f32; 3],
    fov_y: f32,
    near: f32,
    far: f32,
) -> EntityId {
    let entity = world.create_entity(name);

    entity.add_transform(Transform {
        position,
        rotation,
        scale: [1.0, 1.0, 1.0],
    });

   
entity.add_camera(Camera {
    fov: fov_y,  // <-- match struct field name
    near,
    far,
    is_active: true, // maybe default the first one to active
});


    entity.id
}

// ============================================================================
// CAMERA MANAGEMENT FUNCTIONS
// ============================================================================

pub fn set_active_camera(world: &mut World, id: EntityId) {
    for entity in &mut world.entities {
        if let Some(cam) = &mut entity.camera {
            cam.is_active = entity.id == id;
        }
    }
}

pub fn camera_view_proj(camera: &Camera, transform: &Transform, aspect_ratio: f32) -> glam::Mat4 {
    // View
    let eye = glam::Vec3::from_array(transform.position);
    let forward = rotation_to_forward(transform.rotation); // you'd write this helper
    let target = eye + forward;
    let up = glam::Vec3::Y; // [0, 1, 0]
    let view = glam::Mat4::look_at_rh(eye, target, up);

    // Projection
    let projection = glam::Mat4::perspective_rh_gl(
        camera.fov.to_radians(),
        aspect_ratio,
        camera.near,
        camera.far,
    );

    projection * view
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn rotation_to_forward(euler: [f32; 3]) -> glam::Vec3 {
    // Convert Euler (pitch, yaw, roll) into a quaternion
    let rot = glam::Quat::from_euler(
        glam::EulerRot::XYZ,
        euler[0].to_radians(), // pitch
        euler[1].to_radians(), // yaw
        euler[2].to_radians(), // roll
    );

    // Apply quaternion to "forward" (Z axis in most engines)
    rot * glam::Vec3::Z
}
