use crate::modules::ecs::entity::*;
use crate::modules::ecs::scripts::ScriptRegistry;
use crate::modules::ecs::systems::*;
use crate::modules::ecs::world::World;
use crate::modules::state::State;
use crate::modules::ui::gui::*;
use glam::{Vec3, Vec4};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

pub type ProjectId = u32;

#[derive(Clone, Debug)]
pub enum HubAction {
    NewProject,
    LoadProject(ProjectId),
    None,
}

#[derive(Clone, Debug)]
enum AppState {
    Hub,
    Editor { project_id: ProjectId },
    PlayTest { project_id: ProjectId },
}

pub struct App {
    app_state: AppState,
    egui_enabled: bool,
    gpu_state: Option<State>,
    world: Option<World>,
    script_registry: ScriptRegistry,
    last_frame_time: Option<Instant>,
    gui: Option<Gui>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            app_state: AppState::Hub,
            egui_enabled: true,
            gpu_state: None,
            world: None,
            script_registry: ScriptRegistry::new(),
            last_frame_time: None,
            gui: None,
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
        
        // Initialize GUI with proper parameters
        let gui = Gui::new(
            gpu_state.get_device(),
            gpu_state.get_surface_format(),
            &window
        );
        
        self.gpu_state = Some(gpu_state);
        self.gui = Some(gui);

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
        // First, let egui handle it
        if let (Some(gui), Some(gpu_state)) = (&mut self.gui, &self.gpu_state) {
            let consumed = gui.on_event(&gpu_state.get_window(), &event); 
            if consumed {
                // egui handled it, don't pass down
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                self.update_and_render();
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
        self.world = None; // Clear world for hub
        println!("Initialized Hub");
    }

    fn initialize_editor(&mut self, project_id: ProjectId) {
        self.egui_enabled = true;
        self.create_default_scene();
        println!("Initialized Editor for project {}", project_id);
    }

    fn initialize_playtest(&mut self, project_id: ProjectId) {
        self.egui_enabled = false;
        println!("Started playtest for project {}", project_id);
    }

    fn create_default_scene(&mut self) {
        let mut world = World::new();

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

    fn show_hub_menu(&mut self, ctx: &egui::Context) -> HubAction {
        let mut action = HubAction::None;
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ZeroGPU Engine Hub");
            ui.separator();
            
            ui.vertical_centered(|ui| {
                if ui.add_sized([200.0, 50.0], egui::Button::new("New Project")).clicked() {
                    action = HubAction::NewProject;
                }
                
                ui.add_space(10.0);
                
                if ui.add_sized([200.0, 50.0], egui::Button::new("Load Project")).clicked() {
                    action = HubAction::LoadProject(1); // For now, hardcode project ID
                }
                
                ui.add_space(10.0);
                
                ui.label("Recent Projects:");
                // Add list of recent projects here
            });
        });
        
        action
    }

    fn show_editor_interface(&mut self, ctx: &egui::Context, _world: &mut World) {
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Scene").clicked() {
                        // Handle new scene
                    }
                    if ui.button("Save").clicked() {
                        // Handle save
                    }
                    if ui.button("Load").clicked() {
                        // Handle load
                    }
                });
                
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        // Handle undo
                    }
                    if ui.button("Redo").clicked() {
                        // Handle redo
                    }
                });
                
                ui.separator();
                ui.label("Editor Mode");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("▶ Play (F5)").clicked() {
                        // Will be handled by the return value
                    }
                });
            });
        });

        // Left panel - Scene hierarchy
        egui::SidePanel::left("hierarchy").show(ctx, |ui| {
            ui.heading("Scene Hierarchy");
            ui.separator();
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.collapsing("Scene", |ui| {
                    ui.selectable_label(false, "Main Camera");
                    ui.selectable_label(false, "Cube1");
                });
            });
        });

        // Right panel - Inspector
        egui::SidePanel::right("inspector").show(ctx, |ui| {
            ui.heading("Inspector");
            ui.separator();
            
            ui.label("Transform");
            ui.horizontal(|ui| {
                ui.label("Position:");
                ui.add(egui::DragValue::new(&mut 0.0f32).prefix("X: "));
                ui.add(egui::DragValue::new(&mut 0.0f32).prefix("Y: "));
                ui.add(egui::DragValue::new(&mut 0.0f32).prefix("Z: "));
            });
        });

        // Bottom panel - Console/logs
        egui::TopBottomPanel::bottom("console").show(ctx, |ui| {
            ui.heading("Console");
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label("Engine initialized successfully");
                ui.label("Scene loaded");
            });
        });
    }

    fn render_hub_ui(&mut self) {
        if let (Some(gpu_state), Some(gui)) = (&mut self.gpu_state, &mut self.gui) {
            let window = gpu_state.get_window();
            let size = window.inner_size();
            
            // Prepare GUI frame
            let mut encoder = gpu_state.get_device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("GUI Command Encoder"),
                }
            );
            
            let paint_jobs = gui.prepare(
                gpu_state.get_device(),
                gpu_state.get_queue(),
                &window,
                &mut encoder,
                (size.width, size.height),
            );
            
            // Show hub menu
            let hub_result = self.show_hub_menu(&gui.egui_ctx);
            
            match hub_result {
                HubAction::NewProject => {
                    self.create_new_project();
                }
                HubAction::LoadProject(project_id) => {
                    self.load_project(project_id);
                }
                HubAction::None => {}
            }
            
            // Render everything
            if let Ok(surface_texture) = gpu_state.get_surface().get_current_texture() {
                let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
                
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("GUI Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    
                    let screen_descriptor = egui_wgpu::ScreenDescriptor {
                        size_in_pixels: [size.width, size.height],
                        pixels_per_point: 1.0,
                    };
                    
                    gui.render(&mut rpass, &paint_jobs, &screen_descriptor);
                }
                
                gpu_state.get_queue().submit(std::iter::once(encoder.finish()));
                surface_texture.present();
            }
        }
    }

    fn render_editor_ui(&mut self) {
        if let (Some(gpu_state), Some(gui), Some(world)) = 
            (&mut self.gpu_state, &mut self.gui, &mut self.world) {
            
            let window = gpu_state.get_window();
            let size = window.inner_size();
            
            // Prepare GUI frame
            let mut encoder = gpu_state.get_device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Editor Command Encoder"),
                }
            );
            
            let paint_jobs = gui.prepare(
                gpu_state.get_device(),
                gpu_state.get_queue(),
                &window,
                &mut encoder,
                (size.width, size.height),
            );
            
            // Show editor interface
            self.show_editor_interface(&gui.egui_ctx, world);
            
            // Render 3D scene first, then GUI
            if let Err(e) = update_and_render(world, gpu_state, &mut self.script_registry, 0.0) {
                eprintln!("Editor 3D render error: {}", e);
            }
            
            // Then render GUI overlay
            if let Ok(surface_texture) = gpu_state.get_surface().get_current_texture() {
                let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
                
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Editor GUI Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load, // Don't clear, draw over 3D scene
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    
                    let screen_descriptor = egui_wgpu::ScreenDescriptor {
                        size_in_pixels: [size.width, size.height],
                        pixels_per_point: 1.0,
                    };
                    
                    gui.render(&mut rpass, &paint_jobs, &screen_descriptor);
                }
                
                gpu_state.get_queue().submit(std::iter::once(encoder.finish()));
                surface_texture.present();
            }
        }
    }

    fn render_game_only(&mut self) {
        if let Some(gpu_state) = &mut self.gpu_state {
            if let Some(world) = &mut self.world {
                if let Err(e) = update_and_render(world, gpu_state, &mut self.script_registry, 0.0) {
                    eprintln!("Game render error: {}", e);
                }
            }
        }
    }

    fn update_game_logic(&mut self, delta_time: f32) {
        if let Some(gpu_state) = &mut self.gpu_state {
            if let Some(world) = &mut self.world {
                if let Err(e) = update_and_render(world, gpu_state, &mut self.script_registry, delta_time) {
                    eprintln!("Game update error: {}", e);
                }
            }
        }
    }

    fn handle_game_input(&mut self, _event: &KeyEvent) {
        // Handle game-specific input when not in GUI mode
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
        let new_project_id = 1; // You might want to generate this properly
        self.transition_to_editor(new_project_id);
        new_project_id
    }

    // Helper method for loading existing projects from hub
    pub fn load_project(&mut self, project_id: ProjectId) {
        self.transition_to_editor(project_id);
    }
}
