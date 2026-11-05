use raylib::math::Vector3;

#[derive(Clone, Debug)]
pub struct Vertex {
    pub position: Vector3,
    pub normal: Vector3,
    pub transformed_position: Vector3,
    pub world_position: Vector3,
    pub world_normal: Vector3,
    pub inv_w: f32,
}

impl Vertex {
    pub fn new(position: Vector3, normal: Vector3) -> Self {
        Self {
            position,
            normal,
            transformed_position: position,
            world_position: position,
            world_normal: normal,
            inv_w: 1.0,
        }
    }
}
