use crate::modules::ecs::components::*;
use crate::modules::ecs::scripts::*;
use crate::modules::ecs::world::*;
use glam::*;

// ============================================================================
// CORE TYPES AND STRUCTS
// ============================================================================

pub struct Camera {
    pub fov: f32,        // Field of view in degrees
    pub near: f32,       // Near clipping plane
    pub far: f32,        // Far clipping plane
    pub is_active: bool, // Only one camera can be active at a time
}

pub struct Entity {
    pub name: String,
    pub transform: Option<Transform>,
    pub mesh_handle: Option<MeshHandle>,
    pub material: Option<Material>,
    pub camera: Option<Camera>,
    pub scripts: Option<Vec<Script>>,
    pub children: Option<Vec<EntityId>>,
    pub parent: Option<EntityId>,
    pub tags: Vec<String>,
}

pub struct EntityBuilder {
    world_ref: *mut World, // Store a raw pointer to avoid lifetime issues
    entity_id: Option<EntityId>,
    name: String,
    transform: Option<Transform>,
    mesh_handle: Option<MeshHandle>,
    material: Option<Material>,
    camera: Option<Camera>,
    scripts: Option<Vec<Script>>,
    children: Option<Vec<EntityId>>,
    parent: Option<EntityId>,
    tags: Vec<String>,
}

pub enum MeshType {
    Triangle,
    Cube,
    Custom(u32), // Let the user pass a GPU mesh ID directly
}

// ============================================================================
// ENTITY BUILDER IMPLEMENTATION
// ============================================================================

impl EntityBuilder {
    /// Create a new EntityBuilder with all the basic components
    pub fn new(
        world: &mut World,
        name: impl Into<String>,
        position: Vec3,
        scale: Vec3,
        mesh: MeshType,
        color: Vec4,
        script_path: Option<impl Into<String>>,
    ) -> Self {
        let name_string = name.into();
        let entity_id = world.create_entity(&name_string);

        let scripts = script_path.map(|s| vec![Script::new(s.into())]);

        Self {
            world_ref: world as *mut World,
            entity_id: Some(entity_id),
            name: name_string,
            transform: Some(Transform {
                position,
                rotation: Vec3::ZERO,
                scale,
            }),
            mesh_handle: Some(MeshHandle(Self::mesh_type_to_id(mesh))),
            material: Some(Material { color }),
            camera: None,
            scripts,
            children: None,
            parent: None,
            tags: vec![],
        }
    }

    /// Add additional script to the entity
    pub fn with_script(mut self, script_path: impl Into<String>) -> Self {
        let script = Script::new(script_path.into());
        match &mut self.scripts {
            Some(scripts) => scripts.push(script),
            None => self.scripts = Some(vec![script]),
        }
        self
    }

    /// Add multiple scripts at once
    pub fn with_scripts(mut self, script_paths: Vec<String>) -> Self {
        let scripts: Vec<Script> = script_paths.into_iter().map(Script::new).collect();
        match &mut self.scripts {
            Some(existing) => existing.extend(scripts),
            None => self.scripts = Some(scripts),
        }
        self
    }

    /// Add a camera component
    pub fn with_camera(mut self, fov: f32, near: f32, far: f32) -> Self {
        self.camera = Some(Camera {
            fov,
            near,
            far,
            is_active: false, // Set to false by default, use set_active_camera later
        });
        self
    }

    /// Add tags to the entity
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags.extend(tags);
        self
    }

    /// Add a single tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Build the entity and insert it into the world
    pub fn build(self) -> EntityId {
        let entity_id = self
            .entity_id
            .expect("EntityBuilder should have an entity_id");

        // Safety: We know the world pointer is valid because we got it from a mutable reference
        let world = unsafe { &mut *self.world_ref };

        if let Some(entity) = world.get_entity_mut(entity_id) {
            // Update the entity with all the builder data
            entity.name = self.name;
            entity.transform = self.transform;
            entity.mesh_handle = self.mesh_handle;
            entity.material = self.material;
            entity.camera = self.camera;
            entity.scripts = self.scripts;
            entity.children = self.children;
            entity.parent = self.parent;
            entity.tags = self.tags;
        }

        entity_id
    }

    /// Convert MeshType enum to mesh ID
    fn mesh_type_to_id(mesh: MeshType) -> u32 {
        match mesh {
            MeshType::Triangle => 0,
            MeshType::Cube => 1,
            MeshType::Custom(id) => id,
        }
    }
}

// ============================================================================
// ENTITY IMPLEMENTATION
// ============================================================================

impl Entity {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transform: None,
            mesh_handle: None,
            material: None,
            children: None,
            parent: None,
            camera: None,
            scripts: None,
            tags: Vec::new(),
        }
    }

    /// Create a basic EntityBuilder (without world integration)
    pub fn builder(name: impl Into<String>) -> EntityBuilder {
        EntityBuilder {
            world_ref: std::ptr::null_mut(),
            entity_id: None,
            name: name.into(),
            transform: None,
            mesh_handle: None,
            material: None,
            camera: None,
            scripts: None,
            children: None,
            parent: None,
            tags: vec![],
        }
    }

    /// Create an EntityBuilder that integrates with the world (this is what you want)
    pub fn builder_with_world(
        world: &mut World,
        name: impl Into<String>,
        position: Vec3,
        scale: Vec3,
        mesh: MeshType,
        color: Vec4,
        script_path: Option<impl Into<String>>,
    ) -> EntityBuilder {
        let name_string = name.into();
        let entity_id = world.create_entity(&name_string);

        let mut builder = EntityBuilder {
            world_ref: world as *mut World,
            entity_id: Some(entity_id),
            name: name_string,
            transform: Some(Transform {
                position,
                rotation: Vec3::ZERO,
                scale,
            }),
            mesh_handle: Some(MeshHandle(EntityBuilder::mesh_type_to_id(mesh))),
            material: Some(Material { color }),
            camera: None,
            scripts: None,
            children: None,
            parent: None,
            tags: vec![],
        };

        if let Some(path) = script_path {
            builder.scripts = Some(vec![Script::new(path.into())]);
        }

        builder
    }

    // Component setters
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn add_camera(&mut self, camera: Camera) {
        self.camera = Some(camera);
    }

    pub fn add_parent(&mut self, parent: EntityId) {
        self.parent = Some(parent);
    }

    pub fn add_child(&mut self, child: EntityId) {
        self.children.get_or_insert_with(Vec::new).push(child);
    }

    pub fn add_transform(&mut self, transform: Transform) {
        self.transform = Some(transform);
    }

    pub fn add_mesh_handle(&mut self, mesh_handle: MeshHandle) {
        self.mesh_handle = Some(mesh_handle);
    }

    pub fn add_material(&mut self, material: Material) {
        self.material = Some(material);
    }

    // Tag management
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

   
}

// ============================================================================
// ENTITY SPAWNING FUNCTIONS
// ============================================================================

pub fn spawn_entity(
    world: &mut World,
    name: impl Into<String>,
    position: Vec3,
    scale: Vec3,
    mesh: MeshType,
    color: Vec4,
) -> EntityId {
    let entity_id = world.create_entity(name);

    if let Some(entity) = world.get_entity_mut(entity_id) {
        entity.add_transform(Transform {
            position,
            rotation: Vec3::ZERO,
            scale,
        });

        let mesh_id = EntityBuilder::mesh_type_to_id(mesh);
        entity.add_mesh_handle(MeshHandle(mesh_id));
        entity.add_material(Material { color });
    }

    entity_id
}

pub fn spawn_scripted_entity(
    world: &mut World,
    name: impl Into<String>,
    position: Vec3,
    scale: Vec3,
    mesh: MeshType,
    color: Vec4,
    script_paths: Vec<String>,
) -> EntityId {
    let entity_id = spawn_entity(world, name, position, scale, mesh, color);

    if let Some(entity) = world.get_entity_mut(entity_id) {
        let scripts: Vec<Script> = script_paths
            .into_iter()
            .map(Script::new)
            .collect();
        entity.add_scripts(scripts);
    }

    entity_id
}

pub fn spawn_single_script_entity(
    world: &mut World,
    name: impl Into<String>,
    position: Vec3,
    scale: Vec3,
    mesh: MeshType,
    color: Vec4,
    script_path: impl Into<String>,
) -> EntityId {
    spawn_scripted_entity(
        world,
        name,
        position,
        scale,
        mesh,
        color,
        vec![script_path.into()],
    )
}

pub fn spawn_camera(
    world: &mut World,
    name: impl Into<String>,
    position: Vec3,
    rotation: Vec3,
    fov_y: f32,
    near: f32,
    far: f32,
) -> EntityId {
    let entity_id = world.create_entity(name);

    if let Some(entity) = world.get_entity_mut(entity_id) {
        entity.add_transform(Transform {
            position,
            rotation,
            scale: Vec3::ZERO,
        });

        entity.add_camera(Camera {
            fov: fov_y,
            near,
            far,
            is_active: true, // Default the first one to active
        });
    }

    entity_id
}

// ============================================================================
// CAMERA MANAGEMENT FUNCTIONS
// ============================================================================

pub fn set_active_camera(world: &mut World, id: EntityId) {
    for (entity_id, entity) in world.iter_entities_mut() {
        if let Some(cam) = &mut entity.camera {
            cam.is_active = entity_id == id;
        }
    }
}

pub fn camera_view_proj(camera: &Camera, transform: &Transform, aspect_ratio: f32) -> Mat4 {
    let eye = transform.position;
    let rotation = Quat::from_euler(
        EulerRot::XYZ,
        transform.rotation.x,
        transform.rotation.y,
        transform.rotation.z,
    );

    // Forward direction (-Z is forward in right-handed system)
    let forward = rotation * -Vec3::Z;
    let target = eye + forward;
    let up = rotation * Vec3::Y;

    // View matrix
    let view = Mat4::look_at_rh(eye, target, up);

    // Projection matrix (perspective)
    let proj = Mat4::perspective_rh_gl(
        camera.fov.to_radians(),
        aspect_ratio,
        camera.near,
        camera.far,
    );

    proj * view
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn rotation_to_forward(euler: [f32; 3]) -> Vec3 {
    // Convert Euler (pitch, yaw, roll) into a quaternion
    let rot = Quat::from_euler(
        EulerRot::XYZ,
        euler[0].to_radians(), // pitch
        euler[1].to_radians(), // yaw
        euler[2].to_radians(), // roll
    );

    // Apply quaternion to "forward" (Z axis)
    rot * Vec3::Z
}
