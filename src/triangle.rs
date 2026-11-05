use raylib::math::{Vector2, Vector3};

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::vertex::Vertex;

// Punto en pantalla ajustado a una malla de 1/256 de píxel, en f64.
// Así la función de arista es exacta y dos triángulos vecinos nunca
// descartan el mismo píxel (sin grietas entre caras).
#[derive(Clone, Copy)]
struct P {
    x: f64,
    y: f64,
}

fn snap(v: Vector3) -> P {
    P {
        x: (v.x as f64 * 256.0).round() / 256.0,
        y: (v.y as f64 * 256.0).round() / 256.0,
    }
}

// Área con signo (x2) del triángulo a, b, p
fn edge_function(a: P, b: P, p: P) -> f64 {
    (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
}

// Regla top-left: arista izquierda o arista horizontal superior
fn is_top_left(a: P, b: P) -> bool {
    let normal_x = -(b.y - a.y);
    let normal_y = b.x - a.x;
    normal_x > 0.0 || (normal_x == 0.0 && normal_y > 0.0)
}

fn is_inside(w: f64, top_left: bool) -> bool {
    w > 0.0 || (w == 0.0 && top_left)
}

fn interpolate(a: Vector3, b: Vector3, c: Vector3, w: [f32; 3]) -> Vector3 {
    a * w[0] + b * w[1] + c * w[2]
}

// Relleno con coordenadas baricéntricas; interpola los atributos de cada vértice
pub fn triangle(v1: &Vertex, v2: &Vertex, v3: &Vertex, uniforms: &Uniforms) -> Vec<Fragment> {
    let mut fragments = Vec::new();

    let mut verts = [v1, v2, v3];
    let mut area = edge_function(
        snap(verts[0].transformed_position),
        snap(verts[1].transformed_position),
        snap(verts[2].transformed_position),
    );

    // Caso degenerado
    if area == 0.0 {
        return fragments;
    }

    // Orden inverso
    if area < 0.0 {
        verts.swap(1, 2);
        area = -area;
    }

    let [za, zb, zc] = verts.map(|v| v.transformed_position.z);
    let [a, b, c] = verts.map(|v| snap(v.transformed_position));
    let top_left_bc = is_top_left(b, c);
    let top_left_ca = is_top_left(c, a);
    let top_left_ab = is_top_left(a, b);

    // Bounding box recortada a la pantalla
    let max_w = uniforms.width as i32;
    let max_h = uniforms.height as i32;
    let min_x = (a.x.min(b.x).min(c.x).floor() as i32).max(0);
    let min_y = (a.y.min(b.y).min(c.y).floor() as i32).max(0);
    let max_x = (a.x.max(b.x).max(c.x).ceil() as i32).min(max_w);
    let max_y = (a.y.max(b.y).max(c.y).ceil() as i32).min(max_h);

    for y in min_y..max_y {
        for x in min_x..max_x {
            // Centro del píxel
            let p = P {
                x: x as f64 + 0.5,
                y: y as f64 + 0.5,
            };

            let w1 = edge_function(b, c, p);
            let w2 = edge_function(c, a, p);
            let w3 = edge_function(a, b, p);

            if !(is_inside(w1, top_left_bc) && is_inside(w2, top_left_ca) && is_inside(w3, top_left_ab)) {
                continue;
            }

            // Coordenadas baricéntricas en pantalla
            let l = [(w1 / area) as f32, (w2 / area) as f32, (w3 / area) as f32];
            let depth = l[0] * za + l[1] * zb + l[2] * zc;

            // Perspectiva correcta: pesar con 1/w y normalizar
            let pw = [
                l[0] * verts[0].inv_w,
                l[1] * verts[1].inv_w,
                l[2] * verts[2].inv_w,
            ];
            let sum = pw[0] + pw[1] + pw[2];
            let pw = [pw[0] / sum, pw[1] / sum, pw[2] / sum];

            let object_position =
                interpolate(verts[0].position, verts[1].position, verts[2].position, pw).normalized();
            let normal = interpolate(
                verts[0].world_normal,
                verts[1].world_normal,
                verts[2].world_normal,
                pw,
            )
            .normalized();
            let intensity = normal.dot(uniforms.light_dir).max(0.0);

            fragments.push(Fragment {
                position: Vector2::new(x as f32, y as f32),
                depth,
                object_position,
                normal,
                intensity,
            });
        }
    }

    fragments
}
