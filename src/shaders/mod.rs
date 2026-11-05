pub mod gas_giant;
pub mod moon;
pub mod noise_helpers;
pub mod rocky;
pub mod star;

use raylib::prelude::*;

use crate::{Uniforms, fragment::Fragment};

#[derive(Clone, Copy, PartialEq)]
pub enum Body {
    Star,
    Rocky,
    GasGiant,
    Moon,
}

impl Body {
    pub fn name(self) -> &'static str {
        match self {
            Body::Star => "Estrella",
            Body::Rocky => "Planeta rocoso",
            Body::GasGiant => "Gigante gaseoso",
            Body::Moon => "Luna",
        }
    }
}

pub fn mix(a: Vector3, b: Vector3, amount: f32) -> Vector3 {
    a * (1.0 - amount) + b * amount
}

pub fn smoothstep(start: f32, end: f32, value: f32) -> f32 {
    let t = ((value - start) / (end - start)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms, body: Body) -> Vector3 {
    match body {
        Body::Star => star::crystal_star(fragment, uniforms),
        Body::Rocky => rocky::volcanic(fragment, uniforms),
        Body::GasGiant => gas_giant::coca_giant(fragment, uniforms),
        Body::Moon => moon::pale_moon(fragment, uniforms),
    }
}
