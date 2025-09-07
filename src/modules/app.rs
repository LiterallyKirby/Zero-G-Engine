use crate::modules::ecs::entity::*;
use crate::modules::ecs::scripts::ScriptRegistry;
use crate::modules::ecs::systems::*;
use crate::modules::ecs::world::World;
use crate::modules::state::State;
use glam::{Vec3, Vec4};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

pub struct App {
    state: Option<State>,
    world: Option<World>,
    script_registry: ScriptRegistry,
    last_frame_time: Option<Instant>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: None,
            world: None,
            script_registry: ScriptRegistry::new(),
            last_frame_time: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        // Create the GPU state
        let state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);
        // Create world and spawn a triangle entity
        let mut world = World::new();

        // Cube
        
let cube_id = Entity::builder_with_world(
    &mut world,
    "Cube1",
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::ONE,
    MeshType::Cube,
    Vec4::new(0.0, 1.0, 0.0, 1.0),
    Some("build/debug.wasm"), // no script
)
.with_tag("player")
.build();




        // Camera
        let cam_id = spawn_camera(
            &mut world,
            "MainCamera",
            Vec3::new(0.0, 0.0, 10.0), // Changed from -5.0 to 5.0
            Vec3::ZERO,               // rotation
            45.0,                     // fov
            0.1,                      // near
            100.0,                    // far
        );
        set_active_camera(&mut world, cam_id);

        // Initialize scripts
        if let Err(e) = init_scripts(&mut world, &mut self.script_registry) {
            eprintln!("Failed to initialize scripts: {}", e);
        }

        // Initialize the frame time
        self.last_frame_time = Some(Instant::now());

        // Now store the world
        self.world = Some(world);
        // Request initial redraw
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(state) = self.state.as_mut() {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::RedrawRequested => {
                    // Calculate delta time
                    let current_time = Instant::now();
                    let delta_time = if let Some(last_time) = self.last_frame_time {
                        let dt = current_time.duration_since(last_time);
                        // Cap delta time to prevent large jumps (e.g., when debugging or minimized)
                        let max_delta = Duration::from_millis(50); // Cap at 50ms (20 FPS minimum)
                        dt.min(max_delta).as_secs_f32()
                    } else {
                        // First frame, use a reasonable default
                        0.016 // ~60 FPS
                    };
                    self.last_frame_time = Some(current_time);

                    if let Some(world) = &mut self.world {
                        // Fixed function call with all required parameters
                        if let Err(e) =
                            update_and_render(world, state, &mut self.script_registry, delta_time)
                        {
                            eprintln!("Update/render error: {}", e);
                        }
                    }

                    // Request the next frame
                    state.get_window().request_redraw();
                }
                WindowEvent::Resized(size) => state.resize(size),
                _ => (),
            }
        }
    }
}
