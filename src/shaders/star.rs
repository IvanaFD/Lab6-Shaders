use super::{
    mix,
    noise_helpers::{cellular, fbm_octaves, hash, hash3},
    smoothstep,
};
use crate::{Uniforms, fragment::Fragment};
use raylib::prelude::*;

const FACET_SCALE: f32 = 5.0;
const FACET_TILT: f32 = 0.35;
const PARALLAX_DEPTH: f32 = 0.35;
const SPARKLE_SPEED: f32 = 2.2;
const AURA_STRENGTH: f32 = 1.6;
const EXPOSURE: f32 = 2.0;

const WHITE: Vector3 = Vector3::new(0.93, 0.97, 1.0);
const ICE: Vector3 = Vector3::new(0.4, 0.82, 1.0);
const DEEP: Vector3 = Vector3::new(0.1, 0.45, 0.9);

fn iridescent(hue: f32) -> Vector3 {
    let tau = std::f32::consts::TAU;
    let rainbow = Vector3::new(
        0.5 + 0.5 * (tau * hue).cos(),
        0.5 + 0.5 * (tau * (hue + 0.33)).cos(),
        0.5 + 0.5 * (tau * (hue + 0.67)).cos(),
    );
    mix(rainbow, WHITE, 0.35)
}

fn sparkle_at(f1: f32, id: Vector3, time: f32, seed: f32) -> f32 {
    let phase = hash(id, seed) * std::f32::consts::TAU;
    let speed = SPARKLE_SPEED * (0.5 + hash(id, seed + 2.0));
    let twinkle = (time * speed + phase).sin().max(0.0).powf(10.0);
    twinkle * ((1.0 - smoothstep(0.0, 0.08, f1)) * 1.6 + (1.0 - smoothstep(0.0, 0.4, f1)) * 0.5)
}

pub fn crystal_star(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_position;
    let n = fragment.normal;
    let time = uniforms.time;
    let view = (uniforms.camera_position - p).normalized();
    let n_dot_v = n.dot(view).max(0.0);

    let height = fbm_octaves(&uniforms.noise, p * 2.5, 3);
    let view_tangent = view - n * view.dot(n);
    let inner = p - view_tangent * (PARALLAX_DEPTH * (1.0 - height) / n_dot_v.max(0.35));

    let drift = Vector3::new(time * 0.15, -time * 0.1, time * 0.08);
    let plasma = fbm_octaves(&uniforms.noise, inner * 3.0 + drift, 3);
    let heat = smoothstep(0.35, 0.75, plasma);
    let veins = 1.0 - (fbm_octaves(&uniforms.noise, inner * 4.0 - drift * 0.5, 3) * 2.0 - 1.0).abs();
    let veins = smoothstep(0.85, 0.98, veins);
    let body = mix(DEEP, ICE, heat);
    let body = mix(body, WHITE, veins * 0.6 + heat * heat * 0.2);

    let core = n_dot_v.powf(2.0);
    let body = mix(body, WHITE, core * 0.25) * (0.9 + core * 0.3);

    let (f1, f2, id) = cellular(p * FACET_SCALE);
    let edge = 1.0 - smoothstep(0.0, 0.035, f2 - f1);
    let facet_normal = (n + (hash3(id) * 2.0 - Vector3::one()) * FACET_TILT).normalized();

    let half = (uniforms.light_dir + view).normalized();
    let specular = facet_normal.dot(half).max(0.0).powf(40.0);
    let glint = facet_normal.dot(half).max(0.0).powf(6.0) * 0.35;
    let tint = iridescent(hash(id, 4.0) + n_dot_v * 0.6 + time * 0.05);
    let reflection = tint * (specular * 1.4 + glint);

    let (fine_f1, _, fine_id) = cellular(p * 13.0);
    let sparkle = sparkle_at(f1, id, time, 7.0) + sparkle_at(fine_f1, fine_id, time * 1.3, 11.0) * 0.8;
    let sparkle_color = mix(WHITE, iridescent(hash(id, 5.0)), 0.3);

    let fresnel = (1.0 - n_dot_v).powf(2.0);
    let aura = mix(ICE, WHITE, fresnel) * fresnel * AURA_STRENGTH;

    let pulse = 0.92 + 0.08 * (time * 1.3).sin();
    let color = body * pulse + WHITE * (edge * 0.12) + reflection + sparkle_color * sparkle + aura;

    Vector3::new(
        1.0 - (-color.x * EXPOSURE).exp(),
        1.0 - (-color.y * EXPOSURE).exp(),
        1.0 - (-color.z * EXPOSURE).exp(),
    )
}
