
use egui::{Context, FullOutput};
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, RenderPass, TextureView};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use winit::window::Window;

pub struct EguiRenderer {
    state: State,
    renderer: Renderer,
    frame_started: bool,
}

impl EguiRenderer {
    pub fn new(
        device: &Device,
        output_color_format: egui_wgpu::wgpu::TextureFormat,
        output_depth_format: Option<egui_wgpu::wgpu::TextureFormat>,
        msaa_samples: u32,
        window: &Window,
    ) -> Self {
        
let egui_ctx = Context::default();
let egui_state = State::new(
    egui_ctx,                   // egui context
    egui::ViewportId::ROOT,      // viewport ID
    window,                       // &Window
    None,                         // Option<f32> for pixels_per_point
    None,                         // Option<winit::window::Theme>
    None,                         // Option<usize> for max_texture_side
);


        let renderer = Renderer::new(device, output_color_format, output_depth_format, msaa_samples, true);

        Self {
            state: egui_state,
            renderer,
            frame_started: false,
        }
    }

    pub fn context(&self) -> &Context {
        self.state.egui_ctx()
    }

    pub fn begin_frame(&mut self, window: &Window) {
        let raw_input = self.state.take_egui_input(window);
        self.state.egui_ctx().begin_pass(raw_input);
        self.frame_started = true;
    }

   
pub fn end_frame_and_draw(
    &mut self,
    device: &Device,
    queue: &Queue,
    encoder: &mut CommandEncoder,
    window: &Window,
    target_view: &TextureView,
    pixels_per_point: f32,
    size_in_pixels: [u32; 2],
    staging_belt: &mut wgpu::util::StagingBelt,
) {
    if !self.frame_started {
        panic!("begin_frame must be called before end_frame_and_draw!");
    }

    // End the frame
    let full_output = self.state.egui_ctx().end_pass();

    // Platform output (clipboard, etc.)
    self.state.handle_platform_output(window, full_output.platform_output);

    // Upload textures
    for (id, image_delta) in &full_output.textures_delta.set {
        self.renderer.update_texture(device, queue, *id, image_delta);
    }

    // Tessellate shapes
   
let clipped_primitives = self.state.egui_ctx().tessellate(full_output.shapes, pixels_per_point);
    // Screen descriptor
    let screen_descriptor = ScreenDescriptor {
        size_in_pixels,
        pixels_per_point,
    };

    // Update buffers
  
self.renderer
    .update_buffers(device, queue, encoder, &clipped_primitives, &screen_descriptor);


   
{
    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("egui_render_pass"),
        color_attachments: &[Some(egui_wgpu::wgpu::RenderPassColorAttachment {
            view: target_view,
            resolve_target: None,
            ops: egui_wgpu::wgpu::Operations {
                load: egui_wgpu::wgpu::LoadOp::Load,
                store: egui_wgpu::wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    self.renderer.render(&mut rpass, &clipped_primitives, &screen_descriptor);
} // rpass is dropped here


    // Free textures
    for tex_id in &full_output.textures_delta.free {
        self.renderer.free_texture(tex_id);
    }

    self.frame_started = false;
}
}
