use glam::{Vec3, Vec4};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Default, Debug)]
pub struct SceneEntity {
    pub name: String,
    #[serde(deserialize_with = "vec3_from_array")]
    pub position: Vec3,
    #[serde(deserialize_with = "vec3_from_array")]
    pub scale: Vec3,
    pub mesh: String,
    #[serde(deserialize_with = "vec4_from_array")]
    pub color: Vec4,
    #[serde(default)]
    pub scripts: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Deserialize, Default, Debug)]
pub struct SceneTransform {
    #[serde(deserialize_with = "vec3_from_array")]
    pub position: Vec3,
    #[serde(deserialize_with = "vec3_from_array")]
    pub rotation: Vec3,
    #[serde(deserialize_with = "vec3_from_array")]
    pub scale: Vec3,
}

#[derive(Deserialize, Default, Debug)]
pub struct SceneCamera {
    pub name: String,
    pub transform: SceneTransform,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub active: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Deserialize, Default, Debug)]
pub struct SceneFile {
    pub entities: Vec<SceneEntity>,
    pub cameras: Vec<SceneCamera>,
}

// Keep your existing custom deserializers
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

// You can remove these traits and implementations as they're not used
// trait IntoVec3 { ... }
// trait IntoVec4 { ... }
