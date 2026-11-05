use raylib::math::Vector3;

#[derive(Clone, Debug)]
pub struct Vertex {
    // Posición original en el modelo
    pub position: Vector3,
    // Normal en el modelo (la esfera es suave: dirección desde el centro)
    pub normal: Vector3,
    // Después del vertex shader: x, y en pantalla y z de profundidad
    pub transformed_position: Vector3,
    // Posición y normal en el mundo (para iluminación y vista)
    pub world_position: Vector3,
    pub world_normal: Vector3,
    // 1 / w del clip space, para interpolar con perspectiva correcta
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
