use ZeroEngine::Engine;
use ZeroEngine::modules::state::State;
use std::env;
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
    engine: Engine,
    last_frame_time: Option<Instant>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: None,
            engine: Engine::new(),
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
        let mut state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);
        let args: Vec<String> = env::args().collect();
        if args.len() <= 0 {
            eprintln!("No command-line arguments provided.");
        } 



        // Initialize engine AFTER State
        
self.engine.init_with_state(self.state.as_mut().unwrap(), args[1].clone());

        self.last_frame_time = Some(Instant::now());
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(state) = self.state.as_mut() {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::RedrawRequested => {
                    let now = Instant::now();
                    let dt = if let Some(last) = self.last_frame_time {
                        let elapsed = now.duration_since(last);
                        let max_delta = Duration::from_millis(50); // cap at 50ms
                        elapsed.min(max_delta).as_secs_f32()
                    } else {
                        0.016 // first frame ~60FPS
                    };
                    self.last_frame_time = Some(now);

                    // Call the engine to update and render
                    if let Err(e) = self.engine.update_and_render(state, dt) {
                        eprintln!("Update/render error: {}", e);
                    }

                    state.get_window().request_redraw();
                }
                WindowEvent::Resized(size) => state.resize(size),
                _ => (),
            }
        }
    }
}
