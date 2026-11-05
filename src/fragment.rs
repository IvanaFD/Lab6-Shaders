use raylib::math::{Vector2, Vector3};

// Datos que recibe el fragment shader por cada píxel
pub struct Fragment {
    pub position: Vector2,
    pub depth: f32,
    // Punto de la esfera unitaria (dirección desde el centro): base de los patrones
    pub object_position: Vector3,
    pub world_position: Vector3,
    // Normal en el mundo, normalizada
    pub normal: Vector3,
    // Iluminación difusa que da el renderer: max(dot(normal, luz), 0)
    pub intensity: f32,
}
