// imports (pick what you need)
use egui::{Context as EguiContext};
use egui_winit;           // crate name may be egui_winit or egui-winit depending on your Cargo
use egui_wgpu;
use winit::event::WindowEvent;
use winit::window::Window;
use wgpu::{Device, Queue, CommandEncoder, TextureFormat};

pub struct Gui {
    pub egui_ctx: EguiContext,
    pub egui_winit: egui_winit::State,
    pub egui_rpass: egui_wgpu::Renderer,
}

impl Gui {
    /// call during init (after you have `device` and surface format)
    pub fn new(device: &Device, surface_format: TextureFormat, window: &Window) -> Self {
        // create egui context + winit state for the main/root viewport
        let egui_ctx = EguiContext::default();
        let egui_winit = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            window,
            /*native_pixels_per_point=*/ None,
            /*theme=*/ None,
            /*max_texture_side=*/ None,
        );

        // create egui-wgpu renderer: (device, color_format, depth_format, msaa_samples, dithering)
        // depth_format can be `None` if you don't use a depth attachment for the GUI pass
        let egui_rpass = egui_wgpu::Renderer::new(
            device,
            surface_format,
            /*output_depth_format*/ None,
            /*msaa_samples*/ 1,
            /*dithering*/ false,
        );

        Self { egui_ctx, egui_winit, egui_rpass }
    }

    /// call from your event handling (winit WindowEvent). Returns true if egui consumed it.
    pub fn on_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let resp = self.egui_winit.on_window_event(window, event);
        // `consumed` -> egui used the input (e.g. clicking a widget); don't feed it to your app when true.
        resp.consumed
    }

    /// prepare and run egui each frame; returns the tessellated paint jobs
    /// `encoder` & `queue` are your wgpu ones; `window_size` is in physical pixels
    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        window: &Window,
        encoder: &mut CommandEncoder,
        physical_size: (u32, u32),
    ) -> Vec<egui::ClippedPrimitive> {
        // Update viewport info (required before take_egui_input).
        // If your window was just created, pass is_init=true once.
        // This updates internal egui viewport/screen info from winit window.
        let view_info = self.egui_winit.egui_input_mut()
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .expect("root viewport missing");
        egui_winit::update_viewport_info(view_info, &self.egui_ctx, window, /*is_init=*/ false);

        // take egui inputs for this frame (time, pointer, keys etc)
        let raw_input = self.egui_winit.take_egui_input(window);

        // run egui (your UI code goes in the closure)
        let full_output = self.egui_ctx.run(raw_input, |_ctx| {
            // UI will be built externally by calling code
        });

        // handle platform output: clipboard, cursor, open urls etc
        self.egui_winit.handle_platform_output(window, full_output.platform_output.clone());

        // tessellate to paint jobs - fix: add the pixels_per_point parameter
        let paint_jobs = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

        // apply texture deltas (fonts, images)
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_rpass
                .update_texture(device, queue, *id, image_delta);
        }
        for id in &full_output.textures_delta.free {
            self.egui_rpass.free_texture(id);
        }

        // Build screen descriptor (size + scale). adjust field names to your egui-wgpu version if needed:
        let pixels_per_point = full_output.pixels_per_point;
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [physical_size.0, physical_size.1],
            pixels_per_point,
        };

        // upload vertex/index/uniform data to the GPU (must be called before render)
        // NOTE: update_buffers returns command buffers (user callbacks). If you use extra command buffers, submit them too.
        let _user_cmds = self.egui_rpass.update_buffers(
            device,
            queue,
            encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        // return paint jobs for the caller to render inside a wgpu::RenderPass
        paint_jobs
    }

    /// actually render the GUI: call this inside a wgpu render pass that targets your swapchain surface
    pub fn render(
        &mut self,
        rpass: &mut wgpu::RenderPass<'_>, // Fix lifetime issue by adding explicit lifetime
        paint_jobs: &[egui::ClippedPrimitive],
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
    ) {
        // this writes into the passed-in render pass
        self.egui_rpass.render(rpass, paint_jobs, screen_descriptor);
    }
}
