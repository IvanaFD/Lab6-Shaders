pub mod noise_helpers;
pub mod star;

use raylib::prelude::*;

use crate::{Uniforms, fragment::Fragment};

// Cuerpo celeste que se está dibujando
#[derive(Clone, Copy, PartialEq)]
pub enum Body {
    Star,
    Rocky,
    GasGiant,
}

impl Body {
    pub fn name(self) -> &'static str {
        match self {
            Body::Star => "Estrella",
            Body::Rocky => "Planeta rocoso",
            Body::GasGiant => "Gigante gaseoso",
        }
    }
}

// lerp
pub fn mix(a: Vector3, b: Vector3, amount: f32) -> Vector3 {
    a * (1.0 - amount) + b * amount
}

pub fn smoothstep(start: f32, end: f32, value: f32) -> f32 {
    let t = ((value - start) / (end - start)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// Punto de entrada: elige el shader según el cuerpo activo
pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    if uniforms.body == Body::Star {
        return star::crystal_star(fragment, uniforms);
    }
    let base = match uniforms.body {
        Body::Star => Vector3::new(1.0, 0.7, 0.2),
        Body::Rocky => Vector3::new(0.3, 0.6, 0.4),
        Body::GasGiant => Vector3::new(0.8, 0.6, 0.4),
    };
    base * (0.1 + fragment.intensity * 0.9)
}
