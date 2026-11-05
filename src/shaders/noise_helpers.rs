use noise::{NoiseFn, OpenSimplex};
use raylib::prelude::*;

pub fn sample(noise: &OpenSimplex, position: Vector3) -> f32 {
    noise.get([position.x as f64, position.y as f64, position.z as f64]) as f32
}

pub fn fbm(noise: &OpenSimplex, position: Vector3) -> f32 {
    fbm_octaves(noise, position, 4)
}

pub fn fbm_octaves(noise: &OpenSimplex, mut position: Vector3, octaves: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    let mut weight = 0.0;
    for _ in 0..octaves {
        value += sample(noise, position) * amplitude;
        weight += amplitude;
        position = position * 2.0;
        amplitude *= 0.5;
    }
    value / weight * 0.5 + 0.5
}

pub fn hash(cell: Vector3, seed: f32) -> f32 {
    let d = cell.x * 127.1 + cell.y * 311.7 + cell.z * 74.7 + seed * 53.3;
    (d.sin() * 43758.547).fract().abs()
}

pub fn hash3(cell: Vector3) -> Vector3 {
    Vector3::new(hash(cell, 0.0), hash(cell, 1.0), hash(cell, 2.0))
}

pub fn cellular(p: Vector3) -> (f32, f32, Vector3) {
    let base = Vector3::new(p.x.floor(), p.y.floor(), p.z.floor());
    let mut f1 = f32::MAX;
    let mut f2 = f32::MAX;
    let mut id = base;
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                let cell = base + Vector3::new(x as f32, y as f32, z as f32);
                let d = (cell + hash3(cell) - p).length();
                if d < f1 {
                    f2 = f1;
                    f1 = d;
                    id = cell;
                } else if d < f2 {
                    f2 = d;
                }
            }
        }
    }
    (f1, f2, id)
}
