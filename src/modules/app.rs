use crate::modules::ecs::components::Transform;
use crate::modules::ecs::components::*;
use crate::modules::ecs::entity::*;
use crate::modules::ecs::world::World;
use crate::modules::state::State;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

#[derive(Default)]
pub struct App {
    state: Option<State>,
    world: Option<World>,
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

        // Spawn a triangle
        spawn_entity(
            &mut world,
            "Triangle1",          // name
            [0.0, 0.0, 0.0],      // position
            [1.0, 1.0, 1.0],      // scale
            MeshType::Triangle,   // mesh
            [1.0, 0.0, 0.0, 1.0], // color
        );

        // Spawn a cube
        spawn_entity(
            &mut world,
            "Cube1",
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            MeshType::Cube,
            [0.0, 1.0, 0.0, 1.0],
        );

        // Spawn a custom mesh with GPU ID 42
        spawn_entity(
            &mut world,
            "CustomMesh1",
            [2.0, 0.0, 0.0],
            [0.5, 0.5, 0.5],
            MeshType::Custom(42),
            [0.0, 0.0, 1.0, 1.0],
        );
        let cam_id = spawn_camera(
            &mut world,
            "MainCamera",
            [0.0, 0.0, -5.0], // position
            [0.0, 0.0, 0.0], // rotation
            45.0,            // fov
            0.1,             // near
            100.0,           // far
        );

        set_active_camera(&mut world, cam_id);

        // Now store the world
        self.world = Some(world);

        // Request initial redraw
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Get state reference once
        if let Some(state) = self.state.as_mut() {
            match event {
                WindowEvent::CloseRequested => {
                    println!("The close button was pressed; stopping");
                    event_loop.exit();
                }

                WindowEvent::RedrawRequested => {
                    // Render the frame with ECS world
                    if let Some(world) = &self.world {
                        state.render(world);
                    }
                    // Request the next frame
                    state.get_window().request_redraw();
                }

                WindowEvent::Resized(size) => {
                    // Reconfigure the surface size
                    state.resize(size);
                }

                _ => (),
            }
        }
    }
}
