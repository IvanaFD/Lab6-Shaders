use raylib::prelude::*;

use crate::Uniforms;
use crate::vertex::Vertex;

pub fn multiply_matrix_vector4(matrix: &Matrix, vector: &Vector4) -> Vector4 {
    Vector4::new(
        matrix.m0 * vector.x + matrix.m4 * vector.y + matrix.m8 * vector.z + matrix.m12 * vector.w,
        matrix.m1 * vector.x + matrix.m5 * vector.y + matrix.m9 * vector.z + matrix.m13 * vector.w,
        matrix.m2 * vector.x + matrix.m6 * vector.y + matrix.m10 * vector.z + matrix.m14 * vector.w,
        matrix.m3 * vector.x + matrix.m7 * vector.y + matrix.m11 * vector.z + matrix.m15 * vector.w,
    )
}

// Modelo -> mundo -> cámara -> clip -> NDC -> pantalla
pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let local = Vector4::new(vertex.position.x, vertex.position.y, vertex.position.z, 1.0);
    let world = multiply_matrix_vector4(&uniforms.model_matrix, &local);
    let view = multiply_matrix_vector4(&uniforms.view_matrix, &world);
    let clip = multiply_matrix_vector4(&uniforms.projection_matrix, &view);

    let ndc = Vector3::new(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w);

    // Viewport: NDC [-1, 1] a píxeles, y hacia abajo
    let screen = Vector3::new(
        (ndc.x + 1.0) * 0.5 * uniforms.width as f32,
        (1.0 - ndc.y) * 0.5 * uniforms.height as f32,
        ndc.z,
    );

    // La matriz de modelo solo escala y traslada: la normal no cambia de dirección
    let normal4 = Vector4::new(vertex.normal.x, vertex.normal.y, vertex.normal.z, 0.0);
    let world_normal = multiply_matrix_vector4(&uniforms.model_matrix, &normal4);

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        transformed_position: screen,
        world_position: Vector3::new(world.x, world.y, world.z),
        world_normal: Vector3::new(world_normal.x, world_normal.y, world_normal.z).normalized(),
        inv_w: 1.0 / clip.w,
    }
}

// Cara trasera: la normal apunta en sentido contrario a la cámara
pub fn is_back_face(tri: &[Vertex], uniforms: &Uniforms) -> bool {
    let center = (tri[0].world_position + tri[1].world_position + tri[2].world_position) / 3.0;
    let normal = tri[0].world_normal + tri[1].world_normal + tri[2].world_normal;
    normal.dot(uniforms.camera_position - center) <= 0.0
}
