use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

// ============================================================================
// CORE DATA STRUCTURES
// ============================================================================
pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
    pub vertex_count: u32,
    pub index_count: Option<u32>,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EntityUniformData {
    view_proj: [[f32; 4]; 4],
    transform: [[f32; 4]; 4],
    color: [f32; 4],
}

impl EntityUniformData {
    pub fn new(view_proj: glam::Mat4, transform: glam::Mat4, color: glam::Vec4) -> Self {
        Self {
            view_proj: view_proj.to_cols_array_2d(),
            transform: transform.to_cols_array_2d(),
            color: color.to_array(),
        }
    }
}
unsafe impl bytemuck::Pod for EntityUniformData {}
unsafe impl bytemuck::Zeroable for EntityUniformData {}

// ============================================================================
// OPTIMIZED STATE STRUCTURE
// ============================================================================
pub struct State {
    window: Arc<Window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    pub entity_pipeline: wgpu::RenderPipeline,
    pub uniform_bind_group_layout: wgpu::BindGroupLayout,
    pub meshes: HashMap<u32, Mesh>,

    // Optimization: Pre-allocated resources
    uniform_buffer: wgpu::Buffer,
    uniform_buffer_size: u64,
    staging_belt: wgpu::util::StagingBelt,
}

// ============================================================================
// STATE IMPLEMENTATION
// ============================================================================
impl State {
    pub async fn new(window: Arc<Window>) -> State {
        // GPU setup
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();
        let size = window.inner_size();
        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        // Bind group layout
        let uniform_bind_group_layout = Self::create_uniform_bind_group_layout(&device);

        // Pipeline creation
        let entity_pipeline =
            Self::create_entity_pipeline(&device, surface_format, &uniform_bind_group_layout);

        // Initialize staging belt and uniform buffer
        let uniform_buffer_size = std::mem::size_of::<EntityUniformData>() as u64;
        let staging_belt = wgpu::util::StagingBelt::new(1024);
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Uniform Buffer"),
            size: uniform_buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut state = State {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            entity_pipeline,
            uniform_bind_group_layout,
            meshes: HashMap::new(),
            uniform_buffer,
            uniform_buffer_size,
            staging_belt,
        };

        state.configure_surface();
        state.load_default_meshes();
        state
    }

    // ============================================================================
    // INITIALIZATION HELPERS
    // ============================================================================
    fn create_uniform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("entity_uniform_bind_group_layout"),
        })
    }

    fn create_entity_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ECS Entity Shader"),
            source: wgpu::ShaderSource::Wgsl(Self::get_shader_source()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ECS Entity Pipeline Layout"),
            bind_group_layouts: &[uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ECS Entity Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Self::get_vertex_buffer_layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        })
    }

    fn get_shader_source() -> std::borrow::Cow<'static, str> {
        r#"
            struct Uniforms {
                view_proj: mat4x4<f32>,
                transform: mat4x4<f32>,
                color: vec4<f32>,
            }
            @group(0) @binding(0)
            var<uniform> uniforms: Uniforms;
            struct VertexInput {
                @location(0) position: vec3<f32>,
            }
            struct VertexOutput {
                @builtin(position) clip_position: vec4<f32>,
            }
            @vertex
            fn vs_main(vertex: VertexInput) -> VertexOutput {
                var out: VertexOutput;
                let world_pos = uniforms.transform * vec4<f32>(vertex.position, 1.0);
                out.clip_position = uniforms.view_proj * world_pos;
                return out;
            }
            @fragment
            fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                return uniforms.color;
            }
        "#
        .into()
    }

    fn get_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: 3 * 4, // 3 floats * 4 bytes each
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }

    // ============================================================================
    // MESH LOADING (OPTIMIZED)
    // ============================================================================
    pub fn load_default_meshes(&mut self) {
        self.load_triangle_mesh();
        self.load_cube_mesh();
    }

    fn load_triangle_mesh(&mut self) {
        const TRIANGLE_VERTICES: &[f32] = &[
            0.0, 0.5, 0.0, // top
            -0.5, -0.5, 0.0, // left
            0.5, -0.5, 0.0, // right
        ];

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Triangle VB"),
                contents: bytemuck::cast_slice(TRIANGLE_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        self.meshes.insert(
            0,
            Mesh {
                vertex_buffer,
                index_buffer: None,
                vertex_count: 3,
                index_count: None,
            },
        );
    }

    fn load_cube_mesh(&mut self) {
        const CUBE_VERTICES: &[f32] = &[
            // Front face
            -0.5, -0.5, 0.5, // 0
            0.5, -0.5, 0.5, // 1
            0.5, 0.5, 0.5, // 2
            -0.5, 0.5, 0.5, // 3
            // Back face
            -0.5, -0.5, -0.5, // 4
            0.5, -0.5, -0.5, // 5
            0.5, 0.5, -0.5, // 6
            -0.5, 0.5, -0.5, // 7
        ];

        const CUBE_INDICES: &[u16] = &[
            // Front face
            0, 1, 2, 2, 3, 0, // Back face
            4, 6, 5, 6, 4, 7, // Left face
            4, 0, 3, 3, 7, 4, // Right face
            1, 5, 6, 6, 2, 1, // Top face
            3, 2, 6, 6, 7, 3, // Bottom face
            4, 5, 1, 1, 0, 4,
        ];

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cube VB"),
                contents: bytemuck::cast_slice(CUBE_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cube IB"),
                contents: bytemuck::cast_slice(CUBE_INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });

        self.meshes.insert(
            1,
            Mesh {
                vertex_buffer,
                index_buffer: Some(index_buffer),
                vertex_count: 8,
                index_count: Some(CUBE_INDICES.len() as u32),
            },
        );
    }

    // ============================================================================
    // UTILITY FUNCTIONS
    // ============================================================================
    pub fn get_window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.configure_surface();
        }
    }

    // ============================================================================
    // OPTIMIZED RENDERING
    // ============================================================================
    pub fn render(&mut self, world: &crate::modules::ecs::world::World) {
        let surface_texture = match self.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(_) => return, // Skip frame if surface is unavailable
        };

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Calculate view projection matrix once
        let aspect = self.size.width as f32 / self.size.height as f32;
        let view_proj = world
            .active_camera_matrix(aspect)
            .unwrap_or(glam::Mat4::IDENTITY);

        // Collect renderable entities with mesh IDs and copy their data
        let renderable_entities: Vec<_> = world.get_renderable_entities();

        if renderable_entities.is_empty() {
            // Early exit if nothing to render
            surface_texture.present();
            return;
        }

        // Begin render pass in a separate scope to avoid borrowing conflicts
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ECS Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,

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

            render_pass.set_pipeline(&self.entity_pipeline);

            // Render all entities - look up meshes during rendering to avoid borrowing conflicts
            for (mesh_id, transform, material) in &renderable_entities {
                if let Some(mesh) = self.meshes.get(mesh_id) {
                    Self::render_entity_in_pass(
                        &self.queue,
                        &self.device,
                        &self.uniform_buffer,
                        &self.uniform_bind_group_layout,
                        &mut render_pass,
                        mesh,
                        transform,
                        material,
                        view_proj,
                    );
                }
            }
        } // render_pass is dropped here, freeing the encoder borrow

        // Submit and present
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }

    fn render_entity_in_pass(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        uniform_buffer: &wgpu::Buffer,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        render_pass: &mut wgpu::RenderPass,
        mesh: &Mesh,
        transform: &crate::modules::ecs::components::Transform,
        material: &crate::modules::ecs::components::Material,
        view_proj: glam::Mat4,
    ) {
        // Create transform matrix using glam
        let transform_matrix = glam::Mat4::from_scale_rotation_translation(
            transform.scale,
            glam::Quat::IDENTITY, // or calculate from rotation if you implement it
            transform.position,
        );

        // Pack into uniform struct
        let uniform = EntityUniformData {
            view_proj: view_proj.to_cols_array_2d(),
            transform: transform_matrix.to_cols_array_2d(),
            color: material.color.into(),
        };

        // Write to GPU
        queue.write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("entity_uniform_bind_group"),
        });

        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));

        if let (Some(index_buffer), Some(index_count)) = (&mesh.index_buffer, mesh.index_count) {
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..index_count, 0, 0..1);
        } else {
            render_pass.draw(0..mesh.vertex_count, 0..1);
        }
    }
}
