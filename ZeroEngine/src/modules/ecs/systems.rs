use crate::modules::ecs::world::*;
use crate::modules::ecs::scripts::*;
use crate::modules::state::State;
use anyhow::{Context, Result};

/// Initialize all script instances for entities that have scripts
pub fn init_scripts(world: &mut World, registry: &mut ScriptRegistry) -> Result<()> {
    // Use the new unified script system
    run_script_system(world, registry, 0.0)
        .context("Failed to initialize scripts")?;
    
    println!("Script initialization completed");
    Ok(())
}

/// Update all script instances with delta time
pub fn handle_scripts(
    world: &mut World,
    registry: &mut ScriptRegistry,
    dt: f32,
) -> Result<()> {
    if dt < 0.0 || dt > 1.0 {
        eprintln!("Warning: Unusual delta time: {:.6}s", dt);
    }

    // Use the new unified script system
    run_script_system(world, registry, dt)
        .context("Failed to update scripts")?;

    Ok(())
}

/// Main update and render loop
pub fn update_and_render(
    world: &mut World,
    state: &mut State,
    registry: &mut ScriptRegistry,
    delta_time: f32,
) -> Result<()> {
    // 1. Update all scripts
    handle_scripts(world, registry, delta_time)
        .context("Failed to update scripts")?;

    // 2. Apply any pending world changes (if you have a system for this)
    // world.apply_pending_changes();

    // 3. Render the current world state
    state.render(world);
state.get_window().request_redraw();

    Ok(())
}
