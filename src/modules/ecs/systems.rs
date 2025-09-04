use crate::modules::state::State;
use crate::modules::ecs::world::World;
/*
pub fn render_system(state: &State, world: &World) {
    // 1. Acquire swapchain frame
    let frame = state.surface.get_current_texture().unwrap();
    let view = frame.texture.create_view(&Default::default());
    let mut encoder = state.device.create_command_encoder(&Default::default());
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            depth_slice: None,  // Added missing field
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,  // Changed from bool to StoreOp
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,  // Added missing field
        timestamp_writes: None,     // Added missing field
    });
    
    // 2. Iterate over ECS entities
    for entity in &world.entities {
        if let (Some(_mesh), Some(_material), Some(_transform)) =
            (&entity.mesh_handle, &entity.material, &entity.transform)
        {
            // For now, just pretend draw:
            // You'd use mesh.id to get GPU buffers, material.color for shader uniforms,
            // and transform for positioning
            render_pass.set_pipeline(&state.triangle_pipeline); // Now accessible as public field
            render_pass.set_vertex_buffer(0, state.triangle_vertex_buffer.slice(..));
            render_pass.draw(0..3, 0..1);
        }
    }
    
    drop(render_pass);
    state.queue.submit(Some(encoder.finish()));
    frame.present();
}
*/
