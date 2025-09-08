// =========================================================
// Host functions provided by Rust (from "context" module)
// =========================================================

// @ts-ignore
@external("context", "get_entity_position_x")
declare function get_entity_position_x(id: u32): f32;
// @ts-ignore  
@external("context", "set_entity_position_x")
declare function set_entity_position_x(id: u32, val: f32): void;
// @ts-ignore
@external("context", "get_entity_position_y") 
declare function get_entity_position_y(id: u32): f32;
// @ts-ignore
@external("context", "set_entity_position_y")
declare function set_entity_position_y(id: u32, val: f32): void;
// @ts-ignore
@external("context", "get_entity_position_z")
declare function get_entity_position_z(id: u32): f32;
// @ts-ignore
@external("context", "set_entity_position_z") 
declare function set_entity_position_z(id: u32, val: f32): void;

// @ts-ignore
@external("context", "get_entity_rotation_x")
declare function get_entity_rotation_x(id: u32): f32;
// @ts-ignore
@external("context", "set_entity_rotation_x")
declare function set_entity_rotation_x(id: u32, val: f32): void;
// @ts-ignore
@external("context", "get_entity_rotation_y")
declare function get_entity_rotation_y(id: u32): f32;
// @ts-ignore
@external("context", "set_entity_rotation_y") 
declare function set_entity_rotation_y(id: u32, val: f32): void;
// @ts-ignore
@external("context", "get_entity_rotation_z")
declare function get_entity_rotation_z(id: u32): f32;
// @ts-ignore
@external("context", "set_entity_rotation_z")
declare function set_entity_rotation_z(id: u32, val: f32): void;

// @ts-ignore
@external("context", "get_entity_scale_x")
declare function get_entity_scale_x(id: u32): f32;
// @ts-ignore
@external("context", "set_entity_scale_x")
declare function set_entity_scale_x(id: u32, val: f32): void;
// @ts-ignore
@external("context", "get_entity_scale_y")
declare function get_entity_scale_y(id: u32): f32;
// @ts-ignore
@external("context", "set_entity_scale_y")
declare function set_entity_scale_y(id: u32, val: f32): void;
// @ts-ignore
@external("context", "get_entity_scale_z")
declare function get_entity_scale_z(id: u32): f32;
// @ts-ignore
@external("context", "set_entity_scale_z")
declare function set_entity_scale_z(id: u32, val: f32): void;

// =========================================================
// Global state
// =========================================================

let currentEntityId: u32 = 0;

export function setCurrentEntity(id: u32): void {
  currentEntityId = id;
}

// =========================================================
// Vec3 proxy
// =========================================================

class Vec3 {
  constructor(
    private id: u32,
    private getterX: (id: u32) => f32,
    private setterX: (id: u32, val: f32) => void,
    private getterY: (id: u32) => f32,
    private setterY: (id: u32, val: f32) => void,
    private getterZ: (id: u32) => f32,
    private setterZ: (id: u32, val: f32) => void,
  ) {}

  get x(): f32 { return this.getterX(this.id); }
  set x(val: f32) { this.setterX(this.id, val); }
  
  get y(): f32 { return this.getterY(this.id); }
  set y(val: f32) { this.setterY(this.id, val); }
  
  get z(): f32 { return this.getterZ(this.id); }
  set z(val: f32) { this.setterZ(this.id, val); }
}

// =========================================================
// Transform proxy
// =========================================================

class Transform {
  position: Vec3;
  rotation: Vec3;
  scale: Vec3;

  constructor(id: u32) {
    this.position = new Vec3(
      id,
      get_entity_position_x, set_entity_position_x,
      get_entity_position_y, set_entity_position_y,
      get_entity_position_z, set_entity_position_z,
    );
    
    this.rotation = new Vec3(
      id,
      get_entity_rotation_x, set_entity_rotation_x,
      get_entity_rotation_y, set_entity_rotation_y,
      get_entity_rotation_z, set_entity_rotation_z,
    );
    
    this.scale = new Vec3(
      id,
      get_entity_scale_x, set_entity_scale_x,
      get_entity_scale_y, set_entity_scale_y,
      get_entity_scale_z, set_entity_scale_z,
    );
  }
}

// =========================================================
// Entity proxy
// =========================================================

class Entity {
  transform: Transform;

  constructor(public id: u32) {
    this.transform = new Transform(id);
  }
}

// =========================================================
// self() accessor
// =========================================================

let _self: Entity | null = null;

export function self(): Entity {
  if (_self == null || _self!.id != currentEntityId) {
    _self = new Entity(currentEntityId);
  }
  return _self!;
}

// =========================================================
// Script lifecycle functions (implement these in your scripts)
// =========================================================

export function init(): void {
  // Called once when the script is first loaded
}

export function update(deltaTime: f32): void {
  // Called every frame with delta time in seconds
}
