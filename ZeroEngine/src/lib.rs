use crate::modules::ecs::entity::Entity;
use crate::modules::ecs::entity::MeshType;
use crate::modules::ecs::scripts::ScriptRegistry;
use crate::modules::ecs::systems::init_scripts;
use crate::modules::ecs::systems::update_and_render;
use crate::modules::ecs::world::World;
use crate::modules::state::State;
use anyhow::Result;
use glam::Vec3;
use glam::Vec4;

// Expose your modules publicly

pub mod modules; // any additional helpers

/// The main engine struct that runtime/editor will use
pub struct Engine {
    pub world: World,
    pub scripts: ScriptRegistry,
}

impl Engine {
    /// Create a new engine instance
    pub fn new() -> Self {
        let mut world = World::new();
        let scripts = ScriptRegistry::new();

        // Example: spawn a cube (you can move this to a helper function)
        use glam::{Vec3, Vec4};
        use modules::ecs::entity::{Entity, MeshType};

        // Initialize scripts
        if let Err(e) = modules::ecs::systems::init_scripts(&mut world, &mut scripts.clone()) {
            eprintln!("Failed to initialize scripts: {}", e);
        }

        Engine { world, scripts }
    }

    /// Update world and render; runtime/editor will provide `State` and delta time

    pub fn update_and_render(&mut self, state: &mut State, dt: f32) -> Result<(), String> {
        modules::ecs::systems::update_and_render(&mut self.world, state, &mut self.scripts, dt)
            .map_err(|e| e.to_string())
    }

    pub fn init_with_state(&mut self, state: &mut State, path: String) {
        self.world = World::new();
        self.scripts = ScriptRegistry::new();

        if let Err(e) = self.load_scene(path) {
            eprintln!("failed to load scene: {}", e);
        }

        // initialize scripts
        if let Err(e) = init_scripts(&mut self.world, &mut self.scripts) {
            eprintln!("Failed to init scripts: {}", e);
        }

        // finally, let State know to resize GPU buffers if needed
        state.resize(state.get_window().inner_size());
    }

    /// Optional: helper to reset or initialize world
    pub fn init_world(&mut self) {
        self.world = World::new();
        self.scripts = ScriptRegistry::new();
    }
}
