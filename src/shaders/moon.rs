use super::{
    mix,
    noise_helpers::{fbm_octaves, hash, hash3, sample},
    smoothstep,
};
use crate::{Uniforms, fragment::Fragment};
use raylib::prelude::*;

const CRATER_SCALE: f32 = 3.5;
const CRATER_DENSITY: f32 = 0.45;
const BUMP_STRENGTH: f32 = 0.6;

const WHITE: Vector3 = Vector3::new(1.0, 0.98, 0.92);
const BEIGE: Vector3 = Vector3::new(0.95, 0.88, 0.68);
const MARE: Vector3 = Vector3::new(0.8, 0.74, 0.6);

// Cráteres: hundimiento con borde elevado en algunas celdas de una malla 3D
fn craters(p: Vector3) -> f32 {
    let q = p * CRATER_SCALE;
    let base = Vector3::new(q.x.floor(), q.y.floor(), q.z.floor());
    let mut h: f32 = 0.0;
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                let cell = base + Vector3::new(x as f32, y as f32, z as f32);
                if hash(cell, 21.0) > CRATER_DENSITY {
                    continue;
                }
                let size = 0.15 + hash(cell, 22.0).powi(2) * 0.55;
                let d = (cell + hash3(cell) - q).length() / size;
                if d < 1.3 {
                    let bowl = -(1.0 - smoothstep(0.0, 0.85, d));
                    let rim = (1.0 - ((d - 0.95) / 0.2).powi(2)).max(0.0) * 0.45;
                    h += bowl + rim;
                }
            }
        }
    }
    h
}

fn height(uniforms: &Uniforms, p: Vector3) -> f32 {
    craters(p) * 0.25 + fbm_octaves(&uniforms.noise, p * 12.0, 2) * 0.06
}

fn tangent_basis(n: Vector3) -> (Vector3, Vector3) {
    let up = if n.y.abs() > 0.99 { Vector3::new(1.0, 0.0, 0.0) } else { Vector3::new(0.0, 1.0, 0.0) };
    let t = up.cross(n).normalized();
    (t, n.cross(t))
}

pub fn pale_moon(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_position;
    let n = fragment.normal;

    let h = height(uniforms, p);
    let (t1, t2) = tangent_basis(p);
    let eps = 0.01;
    let dh1 = (height(uniforms, (p + t1 * eps).normalized()) - h) / eps;
    let dh2 = (height(uniforms, (p + t2 * eps).normalized()) - h) / eps;
    let bumped = (n - (t1 * dh1 + t2 * dh2) * (BUMP_STRENGTH * 0.1)).normalized();
    let diffuse = bumped.dot(uniforms.light_dir).max(0.0);

    let maria = smoothstep(0.45, 0.65, fbm_octaves(&uniforms.noise, p * 1.8 + Vector3::new(11.0, 3.0, 5.0), 3));
    let tone = smoothstep(0.3, 0.7, fbm_octaves(&uniforms.noise, p * 5.0 + Vector3::new(2.0, 9.0, 4.0), 2));
    let mut color = mix(BEIGE, WHITE, tone);
    color = mix(color, MARE, maria * 0.5);
    let crater_floor = smoothstep(-0.2, -0.6, craters(p));
    color = mix(color, MARE, crater_floor * 0.35);
    let speckle = sample(&uniforms.noise, p * 60.0) * 0.5 + 0.5;
    color = color * (0.88 + speckle * 0.2);

    color * (0.05 + diffuse * 1.05)
}
