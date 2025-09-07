// assembly/context.ts
// Declare the host functions that will be provided by Rust
declare function get_entity_position_x(id: u32): f32;
declare function set_entity_position_x(id: u32, val: f32): void;
declare function get_entity_position_y(id: u32): f32;
declare function set_entity_position_y(id: u32, val: f32): void;
declare function get_entity_position_z(id: u32): f32;
declare function set_entity_position_z(id: u32, val: f32): void;

// Global variable to track current entity
let currentEntityId: u32 = 0;

// Function called by Rust to set the current entity context
export function setCurrentEntity(id: u32): void {
  currentEntityId = id;
}

// Proxy class to make entity access feel natural
class EntityProxy {
  constructor(private id: u32) {}
  
  get positionX(): f32 { 
    return get_entity_position_x(this.id); 
  }
  
  set positionX(val: f32) { 
    set_entity_position_x(this.id, val); 
  }
  
  get positionY(): f32 { 
    return get_entity_position_y(this.id); 
  }
  
  set positionY(val: f32) { 
    set_entity_position_y(this.id, val); 
  }
  
  get positionZ(): f32 { 
    return get_entity_position_z(this.id); 
  }
  
  set positionZ(val: f32) { 
    set_entity_position_z(this.id, val); 
  }
}

// Function to get a proxy for the current entity
export function self(): EntityProxy {
  return new EntityProxy(currentEntityId);
}
