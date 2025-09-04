use std::sync::Arc;
use winit::window::Window;
use std::collections::HashMap;
use wgpu::util::DeviceExt;
pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
    pub vertex_count: u32,
    pub index_count: Option<u32>,
}

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
}

impl State {
    pub async fn new(window: Arc<Window>) -> State {
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

        // Create uniform bind group layout for ECS entities
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        });

        // Create pipeline for ECS entities
        let entity_pipeline = Self::create_entity_pipeline(&device, surface_format, &uniform_bind_group_layout);

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
        };
        
        state.configure_surface();
        state.load_default_meshes();
        state
    }

    pub fn load_default_meshes(&mut self) {
        // Triangle vertices
        let triangle_vertices: &[f32] = &[
            0.0, 0.5, 0.0,   // top
            -0.5, -0.5, 0.0, // left
            0.5, -0.5, 0.0,  // right
        ];
        let triangle_vertex_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Triangle VB"),
                contents: bytemuck::cast_slice(triangle_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );
        self.meshes.insert(0, Mesh {
            vertex_buffer: triangle_vertex_buffer,
            index_buffer: None,
            vertex_count: 3,
            index_count: None,
        });

        // Cube vertices
        let cube_vertices: &[f32] = &[
            // Front face
            -0.5, -0.5,  0.5,  // 0
             0.5, -0.5,  0.5,  // 1
             0.5,  0.5,  0.5,  // 2
            -0.5,  0.5,  0.5,  // 3
            // Back face
            -0.5, -0.5, -0.5,  // 4
             0.5, -0.5, -0.5,  // 5
             0.5,  0.5, -0.5,  // 6
            -0.5,  0.5, -0.5,  // 7
        ];
        
        let cube_indices: &[u16] = &[
            // Front face
            0, 1, 2,  2, 3, 0,
            // Back face
            4, 6, 5,  6, 4, 7,
            // Left face
            4, 0, 3,  3, 7, 4,
            // Right face
            1, 5, 6,  6, 2, 1,
            // Top face
            3, 2, 6,  6, 7, 3,
            // Bottom face
            4, 5, 1,  1, 0, 4,
        ];
        
        let cube_vertex_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Cube VB"),
                contents: bytemuck::cast_slice(cube_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );
        
        let cube_index_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Cube IB"),
                contents: bytemuck::cast_slice(cube_indices),
                usage: wgpu::BufferUsages::INDEX,
            },
        );
        
        self.meshes.insert(1, Mesh {
            vertex_buffer: cube_vertex_buffer,
            index_buffer: Some(cube_index_buffer),
            vertex_count: 8,
            index_count: Some(cube_indices.len() as u32),
        });
    }

    fn create_entity_pipeline(
        device: &wgpu::Device, 
        format: wgpu::TextureFormat,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ECS Entity Shader"),
            source: wgpu::ShaderSource::Wgsl(r#"
                struct Uniforms {
                    transform: mat4x4<f32>,
                    color: vec4<f32>,
                }
                @group(0) @binding(0)
                var<uniform> uniforms: Uniforms;

                struct VertexInput {
                    @location(0) position: vec3<f32>,
                }

                @vertex
                fn vs_main(vertex: VertexInput) -> @builtin(position) vec4<f32> {
                    let world_pos = vec4<f32>(vertex.position, 1.0);
                    return uniforms.transform * world_pos;
                }

                @fragment
                fn fs_main() -> @location(0) vec4<f32> {
                    return uniforms.color;
                }
            "#.into()),
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ECS Entity Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ECS Entity Pipeline Layout"),
                bind_group_layouts: &[uniform_bind_group_layout],
                push_constant_ranges: &[],
            })),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 3 * 4, // 3 floats * 4 bytes each
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    }],
                }],
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
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        })
    }

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
        self.size = new_size;
        self.configure_surface();
    }

    pub fn render(&mut self, world: &crate::modules::ecs::world::World) {
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });
        
        let mut encoder = self.device.create_command_encoder(&Default::default());
        
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ECS Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
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
        
        // Only render ECS entities - nothing else
        for entity in &world.entities {
            if let (Some(mesh_handle), Some(material), Some(transform)) =
                (&entity.mesh_handle, &entity.material, &entity.transform)
            {
                if let Some(mesh) = self.meshes.get(&mesh_handle.0) {
                    self.render_entity(&mut render_pass, mesh, transform, material);
                }
            }
        }
        
        drop(render_pass);
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }
    
    fn render_entity(
        &self,
        render_pass: &mut wgpu::RenderPass,
        mesh: &Mesh,
        transform: &crate::modules::ecs::components::Transform,
        material: &crate::modules::ecs::components::Material,
    ) {
        // Create transform matrix from ECS transform component
        let transform_matrix = self.create_transform_matrix(transform);
        
        // Create uniforms with transform and material data
        let uniforms = EntityUniformData {
            transform: transform_matrix,
            color: material.color,
        };
        
        let uniform_array = [uniforms];
        let uniform_data = bytemuck::cast_slice(&uniform_array);
        
        let uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Uniform Buffer"),
            size: uniform_data.len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        self.queue.write_buffer(&uniform_buffer, 0, uniform_data);
        
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("entity_uniform_bind_group"),
        });
        
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        
        // Draw based on whether mesh has indices or not
        if let Some(index_buffer) = &mesh.index_buffer {
            if let Some(index_count) = mesh.index_count {
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..index_count, 0, 0..1);
            }
        } else {
            render_pass.draw(0..mesh.vertex_count, 0..1);
        }
    }
    
    fn create_transform_matrix(&self, transform: &crate::modules::ecs::components::Transform) -> [[f32; 4]; 4] {
        // Create scale matrix
        let scale = [
            [transform.scale[0], 0.0, 0.0, 0.0],
            [0.0, transform.scale[1], 0.0, 0.0],
            [0.0, 0.0, transform.scale[2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        
        // Create translation matrix
        let translation = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [transform.position[0], transform.position[1], transform.position[2], 1.0],
        ];
        
        // For now, just combine scale and translation (rotation can be added later)
        // This is a simplified matrix multiplication: translation * scale
        [
            [scale[0][0], 0.0, 0.0, 0.0],
            [0.0, scale[1][1], 0.0, 0.0],
            [0.0, 0.0, scale[2][2], 0.0],
            [translation[3][0], translation[3][1], translation[3][2], 1.0],
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EntityUniformData {
    transform: [[f32; 4]; 4],
    color: [f32; 4],
}

unsafe impl bytemuck::Pod for EntityUniformData {}
unsafe impl bytemuck::Zeroable for EntityUniformData {}
