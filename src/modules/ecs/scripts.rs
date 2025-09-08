use crate::modules::ecs::entity::*;
use crate::modules::ecs::world::*;
use anyhow::Result;
use std::cell::RefCell;
use std::collections::HashMap;
use wasmtime::*;

pub type ScriptId = u32;

// ============================================================================
// SCRIPT TAG REGISTRY
// ============================================================================

pub struct ScriptTagRegistry {
    name_to_id: HashMap<String, u32>,
    next_id: u32,
}

impl ScriptTagRegistry {
    pub fn new() -> Self {
        Self {
            name_to_id: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn get_or_create(&mut self, name: &str) -> u32 {
        if let Some(&id) = self.name_to_id.get(name) {
            id
        } else {
            let id = self.next_id;
            self.name_to_id.insert(name.to_string(), id);
            self.next_id += 1;
            id
        }
    }
}

// ============================================================================
// CORE SCRIPT TYPES
// ============================================================================

pub struct Script {
    pub script_path: String,
    pub is_initialized: bool,
    pub script_data: HashMap<String, f32>,
}

impl Script {
    pub fn new(script_path: impl Into<String>) -> Self {
        Self {
            script_path: script_path.into(),
            is_initialized: false,
            script_data: HashMap::new(),
        }
    }
}

pub struct ScriptRegistry {
    tags: ScriptTagRegistry,
    paths: HashMap<ScriptId, String>,
}

impl ScriptRegistry {
    pub fn new() -> Self {
        Self {
            tags: ScriptTagRegistry::new(),
            paths: HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, path: &str) -> ScriptId {
        let id = self.tags.get_or_create(path);
        self.paths.entry(id).or_insert_with(|| path.to_string());
        id
    }

    pub fn resolve(&self, id: ScriptId) -> Option<&String> {
        self.paths.get(&id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScriptInstanceId {
    pub entity_id: EntityId,
    pub script_id: ScriptId,
}

impl ScriptInstanceId {
    pub fn new(entity_id: EntityId, script_id: ScriptId) -> Self {
        Self {
            entity_id,
            script_id,
        }
    }
}

// ============================================================================
// WASM SCRIPT CONTEXT AND RUNTIME
// ============================================================================

#[derive(Clone)]
pub struct ScriptContext {
    pub current_entity_id: Option<EntityId>,
    pub entity_handle: Option<u32>,
}

pub struct ScriptRuntime {
    engine: Engine,
    instances: HashMap<ScriptInstanceId, Instance>,
    stores: HashMap<ScriptInstanceId, Store<ScriptContext>>,
    world_ptr: Option<*mut World>,
    // Map EntityId to a simple u32 for WASM communication
    entity_id_map: HashMap<EntityId, u32>,
    reverse_entity_id_map: HashMap<u32, EntityId>,
    next_entity_handle: u32,
}

impl ScriptRuntime {
    pub fn new() -> Self {
        Self {
            engine: Engine::default(),
            instances: HashMap::new(),
            stores: HashMap::new(),
            world_ptr: None,
            entity_id_map: HashMap::new(),
            reverse_entity_id_map: HashMap::new(),
            next_entity_handle: 1,
        }
    }

    pub fn set_world_reference(&mut self, world: *mut World) {
        self.world_ptr = Some(world);
    }

    fn get_or_create_entity_handle(&mut self, entity_id: EntityId) -> u32 {
        if let Some(&handle) = self.entity_id_map.get(&entity_id) {
            return handle;
        }

        let handle = self.next_entity_handle;
        self.entity_id_map.insert(entity_id, handle);
        self.reverse_entity_id_map.insert(handle, entity_id);
        self.next_entity_handle += 1;
        handle
    }

    fn get_entity_id_from_handle(&self, handle: u32) -> Option<EntityId> {
        self.reverse_entity_id_map.get(&handle).copied()
    }

    pub fn init_script_instance(
        &mut self,
        instance_id: ScriptInstanceId,
        wasm_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let module = Module::from_file(&self.engine, wasm_path)?;
        let entity_handle = self.get_or_create_entity_handle(instance_id.entity_id);

        // Create store with context that includes the entity handle
        let mut store = Store::new(
            &self.engine,
            ScriptContext {
                current_entity_id: Some(instance_id.entity_id),
                entity_handle: Some(entity_handle),
            },
        );

        // Create linker and add host functions
        let mut linker = Linker::new(&self.engine);
        self.add_host_functions(&mut linker)?;

        // Create and initialize instance
        let instance = linker.instantiate(&mut store, &module)?;

        // Call setCurrentEntity if it exists
        if let Ok(set_current_entity) =
            instance.get_typed_func::<u32, ()>(&mut store, "setCurrentEntity")
        {
            let _ = set_current_entity.call(&mut store, entity_handle);
        }

        // Call init function if it exists
        if let Ok(init_func) = instance.get_typed_func::<(), ()>(&mut store, "init") {
            let _ = init_func.call(&mut store, ());
        }

        // Store the instance and store
        self.instances.insert(instance_id, instance);
        self.stores.insert(instance_id, store);

        Ok(())
    }

    pub fn update_script_instance(
        &mut self,
        instance_id: ScriptInstanceId,
        dt: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(store) = self.stores.get_mut(&instance_id) {
            if let Some(instance) = self.instances.get(&instance_id) {
                let update_func = instance.get_typed_func::<f32, ()>(&mut *store, "update")?;
                update_func.call(&mut *store, dt)?;
            }
        }
        Ok(())
    }

    // ========================================================================
    // HOST FUNCTION REGISTRATION
    // ========================================================================

    fn add_host_functions(
        &self,
        linker: &mut Linker<ScriptContext>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Position getters
        linker.func_wrap("context", "get_entity_position_x", Self::get_position_x)?;
        linker.func_wrap("context", "get_entity_position_y", Self::get_position_y)?;
        linker.func_wrap("context", "get_entity_position_z", Self::get_position_z)?;

        // Position setters
        linker.func_wrap("context", "set_entity_position_x", Self::set_position_x)?;
        linker.func_wrap("context", "set_entity_position_y", Self::set_position_y)?;
        linker.func_wrap("context", "set_entity_position_z", Self::set_position_z)?;

        // Rotation setters
        linker.func_wrap("context", "set_entity_rotation_x", Self::set_rotation_x)?;
        linker.func_wrap("context", "set_entity_rotation_y", Self::set_rotation_y)?;
        linker.func_wrap("context", "set_entity_rotation_z", Self::set_rotation_z)?;

        // Rotation getters
        linker.func_wrap("context", "get_entity_rotation_x", Self::get_rotation_x)?;
        linker.func_wrap("context", "get_entity_rotation_y", Self::get_rotation_y)?;
        linker.func_wrap("context", "get_entity_rotation_z", Self::get_rotation_z)?;

        // Scale setters
        linker.func_wrap("context", "set_entity_scale_x", Self::set_scale_x)?;
        linker.func_wrap("context", "set_entity_scale_y", Self::set_scale_y)?;
        linker.func_wrap("context", "set_entity_scale_z", Self::set_scale_z)?;

        // Scale getters
        linker.func_wrap("context", "get_entity_scale_x", Self::get_scale_x)?;
        linker.func_wrap("context", "get_entity_scale_y", Self::get_scale_y)?;
        linker.func_wrap("context", "get_entity_scale_z", Self::get_scale_z)?;

        linker.func_wrap(
            "env",
            "console.log",
            |mut caller: Caller<'_, ScriptContext>, ptr: i32| {
                let memory = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .expect("WASM must export memory");

                // AssemblyScript strings are UTF-16, starting with length at ptr-4
                let data = memory.data(&caller);
                let len =
                    u32::from_le_bytes(data[(ptr - 4) as usize..ptr as usize].try_into().unwrap());
                let bytes = &data[ptr as usize..(ptr as usize + len as usize * 2)];

                // Decode UTF-16
                let utf16: Vec<u16> = bytes
                    .chunks(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();

                let string = String::from_utf16(&utf16).unwrap_or("<utf16 error>".to_string());
                println!("[WASM] {}", string);
            },
        )?;

        // Error handling
        linker.func_wrap("env", "abort", Self::abort_handler)?;

        Ok(())
    }

    // Position getter functions
    fn get_position_x(caller: Caller<'_, ScriptContext>, _entity_handle: u32) -> f32 {
        Self::get_position_component(&caller, |pos| pos.x)
    }

    fn get_position_y(caller: Caller<'_, ScriptContext>, _entity_handle: u32) -> f32 {
        Self::get_position_component(&caller, |pos| pos.y)
    }

    fn get_position_z(caller: Caller<'_, ScriptContext>, _entity_handle: u32) -> f32 {
        Self::get_position_component(&caller, |pos| pos.z)
    }

    // Position setter functions
    fn set_position_x(caller: Caller<'_, ScriptContext>, _entity_handle: u32, val: f32) {
        Self::set_position_component(&caller, |pos| pos.x = val);
    }

    fn set_position_y(caller: Caller<'_, ScriptContext>, _entity_handle: u32, val: f32) {
        Self::set_position_component(&caller, |pos| pos.y = val);
    }

    fn set_position_z(caller: Caller<'_, ScriptContext>, _entity_handle: u32, val: f32) {
        Self::set_position_component(&caller, |pos| pos.z = val);
    }

    // Rotation setter functions
    fn set_rotation_x(caller: Caller<'_, ScriptContext>, _entity_handle: u32, val: f32) {
        Self::set_rotation_component(&caller, |pos| pos.x = val);
    }

    fn set_rotation_y(caller: Caller<'_, ScriptContext>, _entity_handle: u32, val: f32) {
        Self::set_rotation_component(&caller, |pos| pos.y = val);
    }

    fn set_rotation_z(caller: Caller<'_, ScriptContext>, _entity_handle: u32, val: f32) {
        Self::set_rotation_component(&caller, |pos| pos.z = val);
    }

    // Rotation getter functions
    fn get_rotation_x(caller: Caller<'_, ScriptContext>, _entity_handle: u32) -> f32 {
        Self::get_rotation_component(&caller, |pos| pos.x)
    }

    fn get_rotation_y(caller: Caller<'_, ScriptContext>, _entity_handle: u32) -> f32 {
        Self::get_rotation_component(&caller, |pos| pos.y)
    }

    fn get_rotation_z(caller: Caller<'_, ScriptContext>, _entity_handle: u32) -> f32 {
        Self::get_rotation_component(&caller, |pos| pos.z)
    }

    // Scale getter functions
    fn get_scale_x(caller: Caller<'_, ScriptContext>, _entity_handle: u32) -> f32 {
        Self::get_scale_component(&caller, |pos| pos.x)
    }

    fn get_scale_y(caller: Caller<'_, ScriptContext>, _entity_handle: u32) -> f32 {
        Self::get_scale_component(&caller, |pos| pos.y)
    }

    fn get_scale_z(caller: Caller<'_, ScriptContext>, _entity_handle: u32) -> f32 {
        Self::get_scale_component(&caller, |pos| pos.z)
    }

    // Scale setter functions
    fn set_scale_x(caller: Caller<'_, ScriptContext>, _entity_handle: u32, val: f32) {
        Self::set_scale_component(&caller, |pos| pos.x = val);
    }

    fn set_scale_y(caller: Caller<'_, ScriptContext>, _entity_handle: u32, val: f32) {
        Self::set_scale_component(&caller, |pos| pos.y = val);
    }

    fn set_scale_z(caller: Caller<'_, ScriptContext>, _entity_handle: u32, val: f32) {
        Self::set_scale_component(&caller, |pos| pos.z = val);
    }

    // Helper functions for transform component manipulation
    fn get_position_component<F>(caller: &Caller<'_, ScriptContext>, accessor: F) -> f32
    where
        F: Fn(&glam::Vec3) -> f32,
    {
        let context = caller.data();
        if let Some(entity_id) = context.current_entity_id {
            unsafe {
                if let Some(world_ptr) = MAIN_WORLD_PTR {
                    let world = &*world_ptr;
                    if let Some(entity) = world.get_entity(entity_id) {
                        if let Some(transform) = &entity.transform {
                            return accessor(&transform.position);
                        }
                    }
                }
            }
        }
        0.0
    }

    fn set_position_component<F>(caller: &Caller<'_, ScriptContext>, mutator: F)
    where
        F: Fn(&mut glam::Vec3),
    {
        let context = caller.data();
        if let Some(entity_id) = context.current_entity_id {
            unsafe {
                if let Some(world_ptr) = MAIN_WORLD_PTR {
                    let world = &mut *(world_ptr as *mut World);
                    if let Some(entity) = world.get_entity_mut(entity_id) {
                        if let Some(transform) = &mut entity.transform {
                            mutator(&mut transform.position);
                        }
                    }
                }
            }
        }
    }

    fn get_rotation_component<F>(caller: &Caller<'_, ScriptContext>, accessor: F) -> f32
    where
        F: Fn(&glam::Vec3) -> f32,
    {
        let context = caller.data();
        if let Some(entity_id) = context.current_entity_id {
            unsafe {
                if let Some(world_ptr) = MAIN_WORLD_PTR {
                    let world = &*world_ptr;
                    if let Some(entity) = world.get_entity(entity_id) {
                        if let Some(transform) = &entity.transform {
                            return accessor(&transform.rotation);
                        }
                    }
                }
            }
        }
        0.0
    }

    fn set_rotation_component<F>(caller: &Caller<'_, ScriptContext>, mutator: F)
    where
        F: Fn(&mut glam::Vec3),
    {
        let context = caller.data();
        if let Some(entity_id) = context.current_entity_id {
            unsafe {
                if let Some(world_ptr) = MAIN_WORLD_PTR {
                    let world = &mut *(world_ptr as *mut World);
                    if let Some(entity) = world.get_entity_mut(entity_id) {
                        if let Some(transform) = &mut entity.transform {
                            mutator(&mut transform.rotation);
                        }
                    }
                }
            }
        }
    }

    fn get_scale_component<F>(caller: &Caller<'_, ScriptContext>, accessor: F) -> f32
    where
        F: Fn(&glam::Vec3) -> f32,
    {
        let context = caller.data();
        if let Some(entity_id) = context.current_entity_id {
            unsafe {
                if let Some(world_ptr) = MAIN_WORLD_PTR {
                    let world = &*world_ptr;
                    if let Some(entity) = world.get_entity(entity_id) {
                        if let Some(transform) = &entity.transform {
                            return accessor(&transform.scale);
                        }
                    }
                }
            }
        }
        0.0
    }

    fn set_scale_component<F>(caller: &Caller<'_, ScriptContext>, mutator: F)
    where
        F: Fn(&mut glam::Vec3),
    {
        let context = caller.data();
        if let Some(entity_id) = context.current_entity_id {
            unsafe {
                if let Some(world_ptr) = MAIN_WORLD_PTR {
                    let world = &mut *(world_ptr as *mut World);
                    if let Some(entity) = world.get_entity_mut(entity_id) {
                        if let Some(transform) = &mut entity.transform {
                            mutator(&mut transform.scale);
                        }
                    }
                }
            }
        }
    }

    fn abort_handler(msg_ptr: i32, file_ptr: i32, line: i32, col: i32) {
        panic!(
            "WASM called abort at {}:{} (msg ptr {}) col {}",
            file_ptr, line, msg_ptr, col
        );
    }
}

// ============================================================================
// GLOBAL SCRIPT SYSTEM
// ============================================================================

// Global instances
thread_local! {
    pub static SCRIPT_RUNTIME: RefCell<ScriptRuntime> = RefCell::new(ScriptRuntime::new());
}

// Store reference to the main world that scripts can access
static mut MAIN_WORLD_PTR: Option<*mut World> = None;

pub fn set_script_world_reference(world: &mut World) {
    unsafe {
        MAIN_WORLD_PTR = Some(world as *mut World);
    }

    // Also set it in the script runtime
    SCRIPT_RUNTIME.with(|runtime| {
        runtime
            .borrow_mut()
            .set_world_reference(world as *mut World);
    });
}

/// Main script system - handles multiple scripts per entity
pub fn run_script_system(
    world: &mut World,
    registry: &mut ScriptRegistry,
    delta_time: f32,
) -> anyhow::Result<()> {
    // Set the world reference first
    set_script_world_reference(world);

    // Collect all entity-script combinations
    let script_instances = collect_script_instances(world);

    SCRIPT_RUNTIME.with(|runtime| -> anyhow::Result<()> {
        let mut runtime = runtime.borrow_mut();

        for (entity_id, script_index, script_path, is_initialized) in script_instances {
            let script_id = registry.get_or_create(&script_path);
            let instance_id = ScriptInstanceId::new(entity_id, script_id);

            if !is_initialized {
                // Initialize the script instance
                if let Err(e) = runtime.init_script_instance(instance_id, &script_path) {
                    eprintln!(
                        "Failed to initialize script '{}' for entity {:?}: {}",
                        script_path, entity_id, e
                    );
                    continue;
                }

                // Mark as initialized in the world
                mark_script_initialized(entity_id, script_index);
            }

            // Update the script instance
            if let Err(e) = runtime.update_script_instance(instance_id, delta_time) {
                eprintln!(
                    "Failed to update script '{}' for entity {:?}: {}",
                    script_path, entity_id, e
                );
            }
        }

        Ok(())
    })?;

    Ok(())
}

fn collect_script_instances(world: &World) -> Vec<(EntityId, usize, String, bool)> {
    world
        .iter_entities()
        .flat_map(|(entity_id, entity)| {
            if let Some(scripts) = &entity.scripts {
                scripts
                    .iter()
                    .enumerate()
                    .map(move |(index, script)| {
                        (
                            entity_id,
                            index,
                            script.script_path.clone(),
                            script.is_initialized,
                        )
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        })
        .collect()
}

fn mark_script_initialized(entity_id: EntityId, script_index: usize) {
    unsafe {
        if let Some(world_ptr) = MAIN_WORLD_PTR {
            let world = &mut *(world_ptr as *mut World);
            if let Some(entity) = world.get_entity_mut(entity_id) {
                if let Some(scripts) = &mut entity.scripts {
                    if let Some(script) = scripts.get_mut(script_index) {
                        script.is_initialized = true;
                    }
                }
            }
        }
    }
}

// ============================================================================
// ENTITY SCRIPT MANAGEMENT EXTENSIONS
// ============================================================================

impl Entity {
    pub fn add_script(&mut self, script: Script) {
        self.scripts.get_or_insert_with(Vec::new).push(script);
    }

    pub fn add_scripts(&mut self, scripts: Vec<Script>) {
        self.scripts.get_or_insert_with(Vec::new).extend(scripts);
    }

    pub fn remove_script(&mut self, index: usize) -> Option<Script> {
        if let Some(scripts) = &mut self.scripts {
            if index < scripts.len() {
                return Some(scripts.remove(index));
            }
        }
        None
    }

    pub fn has_scripts(&self) -> bool {
        self.scripts.as_ref().map_or(false, |s| !s.is_empty())
    }

    pub fn script_count(&self) -> usize {
        self.scripts.as_ref().map_or(0, |s| s.len())
    }

    pub fn get_script(&self, index: usize) -> Option<&Script> {
        self.scripts.as_ref()?.get(index)
    }

    pub fn get_script_mut(&mut self, index: usize) -> Option<&mut Script> {
        self.scripts.as_mut()?.get_mut(index)
    }
}
