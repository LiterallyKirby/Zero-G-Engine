#[derive(Copy, Clone)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

pub struct MeshHandle(pub u32); // index into GPU buffer


#[derive(Copy, Clone)]
pub struct Material {
    pub color: [f32; 4],
}
