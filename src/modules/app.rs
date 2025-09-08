use crate::modules::ecs::entity::*;
use crate::modules::ecs::scripts::ScriptRegistry;
use crate::modules::ecs::systems::*;
use crate::modules::ecs::world::World;
use crate::modules::egui_renderer::*;
use crate::modules::state::State;
use crate::modules::ui::hub::*;
use glam::{Vec3, Vec4}; // Keep these imports - they're used in create_default_scene
use std::sync::Arc;
use std::time::{Duration, Instant};
use egui_winit::pixels_per_point;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent}, // Fixed import
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};
pub type ProjectId = u32;

use wgpu::util::StagingBelt;
#[derive(Clone, Debug)]
enum AppState {
    Hub,
    Editor { project_id: ProjectId },
    PlayTest { project_id: ProjectId },
}
#[derive()]
pub struct App {
    app_state: AppState,
    egui_enabled: bool,
    gpu_state: Option<State>,
    world: Option<World>,
    script_registry: ScriptRegistry, // ✅ FIXED: This was missing!
    last_frame_time: Option<Instant>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            app_state: AppState::Hub,
            egui_enabled: true,
            gpu_state: None,
            world: None,
            script_registry: ScriptRegistry::new(), // ✅ FIXED: Initialize it
            last_frame_time: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let gpu_state = pollster::block_on(State::new(window.clone()));
        self.gpu_state = Some(gpu_state);

        match &self.app_state {
            AppState::Hub => {
                self.initialize_hub();
            }
            AppState::Editor { project_id } => {
                self.initialize_editor(*project_id);
            }
            AppState::PlayTest { project_id } => {
                self.initialize_playtest(*project_id);
            }
        }

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Fixed: Separate the mutable borrow and method call to avoid double mutable borrow
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                self.update_and_render();
                // Fixed: Get window reference separately to avoid borrowing conflicts
                if let Some(gpu_state) = &self.gpu_state {
                    gpu_state.get_window().request_redraw();
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(gpu_state) = &mut self.gpu_state {
                    gpu_state.resize(size);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // ✅ FIXED: Proper winit event handling
                if event.state == winit::event::ElementState::Pressed {
                    match event.physical_key {
                        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::F5) => {
                            if let AppState::Editor { project_id } = &self.app_state {
                                self.transition_to_playtest(*project_id);
                            }
                        }
                        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) => {
                            if let AppState::PlayTest { project_id } = &self.app_state {
                                self.transition_to_editor(*project_id);
                            }
                        }
                        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::F1) => {
                            self.transition_to_hub();
                        }
                        _ => {}
                    }
                }

                if !matches!(self.app_state, AppState::Hub) && !self.egui_enabled {
                    self.handle_game_input(&event);
                }
            }
            _ => {}
        }
    }
}

impl App {
    fn initialize_hub(&mut self) {
        self.egui_enabled = true;
        // TODO: Initialize hub UI state
        println!("Initialized Hub");
    }

    fn initialize_editor(&mut self, project_id: ProjectId) {
        self.egui_enabled = true;
        self.create_default_scene();
        println!("Initialized Editor for project {}", project_id);
    }

    fn initialize_playtest(&mut self, project_id: ProjectId) {
        self.egui_enabled = false;
        // World should already exist from editor, just disable UI
        println!("Started playtest for project {}", project_id);
    }

    fn create_default_scene(&mut self) {
        // Create world and spawn default entities (your existing code)
        let mut world = World::new();

        // Cube - Fixed: Prefix with underscore to indicate intentionally unused
        let _cube_id = Entity::builder_with_world(
            &mut world,
            "Cube1",
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::ONE,
            MeshType::Cube,
            Vec4::new(0.0, 1.0, 0.0, 1.0),
            Some("build/debug.wasm"),
        )
        .with_tag("player")
        .build();

        // Camera
        let cam_id = spawn_camera(
            &mut world,
            "MainCamera",
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::ZERO,
            45.0,
            0.1,
            100.0,
        );
        set_active_camera(&mut world, cam_id);

        // Initialize scripts
        if let Err(e) = init_scripts(&mut world, &mut self.script_registry) {
            eprintln!("Failed to initialize scripts: {}", e);
        }

        self.world = Some(world);
        self.last_frame_time = Some(Instant::now());
    }

    fn update_and_render(&mut self) {
        let current_time = Instant::now();
        let delta_time = if let Some(last_time) = self.last_frame_time {
            let dt = current_time.duration_since(last_time);
            let max_delta = Duration::from_millis(50);
            dt.min(max_delta).as_secs_f32()
        } else {
            0.016
        };
        self.last_frame_time = Some(current_time);

        match &self.app_state {
            AppState::Hub => {
                self.render_hub_ui();
            }
            AppState::Editor { project_id: _ } => {
                self.render_editor_ui();
                self.update_game_logic(delta_time);
            }
            AppState::PlayTest { project_id: _ } => {
                self.update_game_logic(delta_time);
                self.render_game_only();
            }
        }
    }

   
fn render_hub_ui(&mut self) {
    if let Some(gpu_state) = &mut self.gpu_state {
       
let window_ref = gpu_state.get_window(); // immutable borrow
gpu_state.egui_renderer.begin_frame(window_ref);


        // Draw UI
        render_hub(gpu_state.egui_renderer.context());

        // Get next frame from the swap chain
        let frame = gpu_state.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder = gpu_state.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("egui encoder") }
        );

       
let window_ref = gpu_state.get_window(); // separate immutable borrow
let mut staging_belt = gpu_state.staging_belt(); // mutable borrow stored locally

gpu_state.egui_renderer.end_frame_and_draw(
    &gpu_state.device,
    &gpu_state.queue,
    &mut encoder,
    window_ref,
    &target_view,
    pixels_per_point,
    size_in_pixels,
    &mut staging_belt,
);


        // Submit commands
        gpu_state.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}


    fn render_editor_ui(&mut self) {
        // TODO: Render editor UI with egui
        // This would include your inspector, hierarchy, viewport, etc.
        if let Some(gpu_state) = &mut self.gpu_state {
            if let Some(world) = &mut self.world {
                // Render the game in the editor viewport
                if let Err(e) = update_and_render(world, gpu_state, &mut self.script_registry, 0.0)
                {
                    eprintln!("Editor render error: {}", e);
                }
            }
        }
    }

    fn render_game_only(&mut self) {
        if let Some(gpu_state) = &mut self.gpu_state {
            if let Some(world) = &mut self.world {
                if let Err(e) = update_and_render(world, gpu_state, &mut self.script_registry, 0.0)
                {
                    eprintln!("Game render error: {}", e);
                }
            }
        }
    }

    fn update_game_logic(&mut self, delta_time: f32) {
        if let Some(gpu_state) = &mut self.gpu_state {
            if let Some(world) = &mut self.world {
                if let Err(e) =
                    update_and_render(world, gpu_state, &mut self.script_registry, delta_time)
                {
                    eprintln!("Game update error: {}", e);
                }
            }
        }
    }

    fn handle_game_input(&mut self, _event: &KeyEvent) {
        // Handle game-specific input
    }

    // State transition methods
    fn transition_to_hub(&mut self) {
        self.app_state = AppState::Hub;
        self.initialize_hub();
    }

    fn transition_to_editor(&mut self, project_id: ProjectId) {
        self.app_state = AppState::Editor { project_id };
        self.initialize_editor(project_id);
    }

    fn transition_to_playtest(&mut self, project_id: ProjectId) {
        self.app_state = AppState::PlayTest { project_id };
        self.initialize_playtest(project_id);
    }

    // Helper method for creating new projects from hub
    pub fn create_new_project(&mut self) -> ProjectId {
        let new_project_id = 1; // You'd generate this properly
        self.transition_to_editor(new_project_id);
        new_project_id
    }

    // Helper method for loading existing projects from hub
    pub fn load_project(&mut self, project_id: ProjectId) {
        // Load project data from file
        self.transition_to_editor(project_id);
    }
}
