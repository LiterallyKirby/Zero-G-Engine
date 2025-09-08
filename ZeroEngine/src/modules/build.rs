use crate::Engine;
use crate::modules::ecs::components::{Material, MeshHandle, Transform};
use crate::modules::ecs::entity::{Camera, Entity, MeshType};
use crate::modules::ecs::entity::{set_active_camera, spawn_camera};
use crate::modules::ecs::scripts::Script;
use crate::modules::ecs::scripts::ScriptRegistry;
use crate::modules::ecs::world::EntityId;
use crate::modules::ecs::world::World;

use glam::{Vec3, Vec4};
use serde::Deserialize;

use std::fs;

#[derive(Deserialize)]
struct SceneEntity {
    name: String,
    #[serde(deserialize_with = "vec3_from_array")]
    position: Vec3,
    #[serde(deserialize_with = "vec3_from_array")]
    scale: Vec3,
    mesh: String,
    #[serde(deserialize_with = "vec4_from_array")]
    color: Vec4,
    camera: Option<CameraData>,
    scripts: Option<Vec<String>>,
    tags: Option<Vec<String>>,
}

#[derive(Copy, Clone, Deserialize)]
struct CameraData {
    fov: f32,
    near: f32,
    far: f32,
    active: bool,
}

#[derive(Deserialize)]
pub struct SceneTransform {
    #[serde(deserialize_with = "vec3_from_array")]
    pub position: Vec3,
    #[serde(deserialize_with = "vec3_from_array")]
    pub rotation: Vec3,
    #[serde(deserialize_with = "vec3_from_array")]
    pub scale: Vec3,
}

#[derive(Deserialize)]

pub struct SceneCamera {
    pub name: String,
    pub transform: SceneTransform,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub active: bool,
    pub tags: Vec<String>,
}
#[derive(Deserialize)]

struct SceneFile {
    entities: Vec<SceneEntity>,
    cameras: Vec<SceneCamera>,
}

impl Engine {
    pub fn load_scene(&mut self, path: String) -> Result<(), String> {
        let data = fs::read_to_string(path).map_err(|e| e.to_string())?;

        let scene: SceneFile = serde_json::from_str(&data).map_err(|e| {
            eprintln!("Serde error: {:?}", e);
            e.to_string()
        })?;

        for e in scene.entities {
            let mut builder = Entity::builder_with_world(
                &mut self.world,
                e.name,
                Vec3::from(e.position),
                Vec3::from(e.scale),
                match e.mesh.to_lowercase().as_str() {
                    "cube" => MeshType::Cube,
                    "triangle" => MeshType::Triangle,
                    _ => MeshType::Custom(0),
                },
                Vec4::from(e.color),
                e.scripts.as_ref().and_then(|s| s.get(0).cloned()),
            );

            if let Some(cam) = e.camera {
                builder = builder.with_camera(cam.fov, cam.near, cam.far);
            }

            if let Some(tags) = e.tags {
                builder = builder.with_tags(tags);
            }

            let entity_id = builder.build();

            // If camera is marked active, set it

            if let Some(cam) = &e.camera {
                if cam.active {
                    set_active_camera(&mut self.world, entity_id);
                }
            }
        }

        for c in scene.cameras {
            let camera_entity = crate::modules::ecs::entity::spawn_camera(
                &mut self.world,
                c.name,
                c.transform.position,
                c.transform.rotation,
                c.fov,
                c.near,
                c.far,
            );

            if c.active {
                set_active_camera(&mut self.world, camera_entity); // marks this as the active camera
            }
        }

        Ok(())
    }
}

trait IntoVec3 {
    fn into_vec3(self) -> Vec3;
}
impl IntoVec3 for [f32; 3] {
    fn into_vec3(self) -> Vec3 {
        Vec3::new(self[0], self[1], self[2])
    }
}

trait IntoVec4 {
    fn into_vec4(self) -> Vec4;
}
impl IntoVec4 for [f32; 4] {
    fn into_vec4(self) -> Vec4 {
        Vec4::new(self[0], self[1], self[2], self[3])
    }
}

fn vec3_from_array<'de, D>(deserializer: D) -> Result<Vec3, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let arr = <[f32; 3]>::deserialize(deserializer)?;
    Ok(Vec3::new(arr[0], arr[1], arr[2]))
}

fn vec4_from_array<'de, D>(deserializer: D) -> Result<Vec4, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let arr = <[f32; 4]>::deserialize(deserializer)?;
    Ok(Vec4::new(arr[0], arr[1], arr[2], arr[3]))
}
