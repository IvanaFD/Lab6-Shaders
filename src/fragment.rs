use raylib::math::{Vector2, Vector3};

pub struct Fragment {
    pub position: Vector2,
    pub depth: f32,
    pub object_position: Vector3,
    pub normal: Vector3,
    pub intensity: f32,
}
