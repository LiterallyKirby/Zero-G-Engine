use ZeroGPU::modules::scenes::{Entity, Transform};
use ZeroGPU::modules::{self, scenes};
use std::collections::HashSet;
use three_d::*;

pub fn main() {
    // Create a window (a canvas on web)
    let window = Window::new(WindowSettings {
        title: "Zero Engine".to_string(),
        max_size: Some((1920, 1080)),
        ..Default::default()
    })
    .unwrap();

    // Get the graphics context from the window
    let context = window.gl();

    // Create cameras
    let camera = Camera::new_perspective(
        window.viewport(),
        vec3(0.0, 0.0, 2.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(90.0),
        0.1,
        10.0,
    );

    let camera2 = Camera::new_perspective(
        window.viewport(),
        vec3(0.0, 0.0, 2.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        10.0,
    );

    // Create a CPU-side mesh consisting of a single colored triangle
    let positions = vec![
        vec3(0.5, -0.5, 0.0),  // bottom right
        vec3(-0.5, -0.5, 0.0), // bottom left
        vec3(0.0, 0.5, 0.0),   // top
    ];
    let colors = vec![
        Srgba::RED,   // bottom right
        Srgba::GREEN, // bottom left
        Srgba::BLUE,  // top
    ];
    let cpu_mesh = CpuMesh {
        positions: Positions::F32(positions),
        colors: Some(colors),
        ..Default::default()
    };

    // Construct a model, with a default color material
    let model = Gm::new(Mesh::new(&context, &cpu_mesh), ColorMaterial::default());

    // Create the scene
    let mut scene = scenes::Scene::new();

    // Compile the script
    let script_ast = scene
        .engine
        .compile(
            r#"
   
fn update() {
    transform.x += 1.0 * dt; // multiplied to make it visible
    transform.y += 0.5 * dt;
}

"#,
        )
        .unwrap();

    // Add cameras to scene
    scene.add_camera(camera);
    scene.add_camera(camera2);

    // Create entity with script and transform
    let entity = Entity {
        model,
        scripts: vec![script_ast],
        transform: Transform {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    };

    scene.add_entity(entity);

    let key_to_camera = [(Key::Num0, 0), (Key::Num1, 1)];

    // Start the main render loop
    let mut pressed_keys: HashSet<Key> = HashSet::new();
    window.render_loop(move |frame_input| {
        // Handle input
        for event in &frame_input.events {
            match event {
                Event::KeyPress { kind, .. } => {
                    pressed_keys.insert(*kind);
                }
                Event::KeyRelease { kind, .. } => {
                    pressed_keys.remove(kind);
                }
                _ => {}
            }
        }

        // Update camera viewport
        if let Some(active_camera) = scene.cameras.get_mut(scene.active_camera) {
            active_camera.set_viewport(frame_input.viewport);
        }

        // Handle camera switching
        for (key, cam_idx) in &key_to_camera {
            if pressed_keys.contains(key) {
                scene.set_active_camera(*cam_idx);
                break;
            }
        }

        // Update the scene
        let delta_time = (frame_input.elapsed_time / 1000.0) as f32; // Convert to seconds and f32
        scene.update(delta_time);

        // Render
        let mut screen = frame_input.screen();
        screen.clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0));
        scene.render(&mut screen);

        FrameOutput::default()
    });
}
