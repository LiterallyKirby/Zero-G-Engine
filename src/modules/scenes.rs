use rhai::{Engine, Scope};
use three_d::*;

// ---------- Transform ----------
#[derive(Debug, Clone)]
pub struct Transform {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_translation(vec3(self.x as f32, self.y as f32, self.z as f32))
    }
}

// ---------- Script Component ----------
pub struct Script {
    pub code: String,
}

impl Script {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.to_string(),
        }
    }
}

// ---------- Entity ----------

pub struct Entity {
    pub model: Gm<Mesh, ColorMaterial>,
    pub scripts: Vec<rhai::AST>, // store compiled scripts
    pub transform: Transform,
}

// ---------- Scene ----------
pub struct Scene {
    pub entities: Vec<Entity>,
    pub cameras: Vec<Camera>,
    pub active_camera: usize,
    pub engine: Engine,
}

impl Scene {
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Register Transform type with Rhai
        engine.register_type_with_name::<Transform>("Transform");
        engine.register_get_set(
            "x",
            |t: &mut Transform| t.x,
            |t: &mut Transform, val: f64| t.x = val,
        );
        engine.register_get_set(
            "y",
            |t: &mut Transform| t.y,
            |t: &mut Transform, val: f64| t.y = val,
        );
        engine.register_get_set(
            "z",
            |t: &mut Transform| t.z,
            |t: &mut Transform, val: f64| t.z = val,
        );

        Self {
            entities: Vec::new(),
            cameras: Vec::new(),
            active_camera: 0,
            engine,
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn add_camera(&mut self, camera: Camera) {
        self.cameras.push(camera);
    }

    pub fn set_active_camera(&mut self, index: usize) {
        if index < self.cameras.len() {
            self.active_camera = index;
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        for entity in &mut self.entities {
            entity.model.animate(delta_time);
            let mut scope = Scope::new();
            scope.push("dt", delta_time as f64);
            scope.push("transform", entity.transform.clone()); // Still clone initially

            for script in &entity.scripts {
                // Run the script to define the functions
                self.engine
                    .eval_ast_with_scope::<()>(&mut scope, script)
                    .unwrap();

                // Call the entity-specific update function
                self.engine
                    .call_fn::<()>(&mut scope, script, "update", ())
                    .unwrap();
            }

            // Get the modified transform back from the scope
            if let Some(modified_transform) = scope.get_value::<Transform>("transform") {
                entity.transform = modified_transform;
            }

            // Apply transform to model
            let transform_matrix = entity.transform.to_matrix();
            entity.model.set_transformation(transform_matrix);
        }
    }

    pub fn render(&self, target: &mut RenderTarget) {
        if let Some(camera) = self.cameras.get(self.active_camera) {
            for entity in &self.entities {
                target.render(camera, &entity.model, &[]);
            }
        }
    }
}

