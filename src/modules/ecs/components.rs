use glam::{Vec3, Vec4};


#[derive(Copy, Clone)]

pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Vec3,
    pub scale: glam::Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
        }
    }
}

pub struct MeshHandle(pub u32); // index into GPU buffer


#[derive(Copy, Clone)]
pub struct Material {
    pub color: glam::Vec4,
}
