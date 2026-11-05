// Estrella de cristal: blanca y celeste, con facetas que reflejan la luz,
// vetas internas con profundidad (parallax), destellos que parpadean y un aura en el borde.
use super::{mix, noise_helpers::fbm, smoothstep};
use crate::{Uniforms, fragment::Fragment};
use raylib::prelude::*;

// ---- Parámetros de apariencia ----
// Cantidad de facetas del cristal (frecuencia de las celdas)
const FACET_SCALE: f32 = 5.0;
// Cuánto se inclina la normal de cada faceta (más = reflejos más dispersos)
const FACET_TILT: f32 = 0.35;
// Profundidad aparente de las vetas internas (parallax)
const PARALLAX_DEPTH: f32 = 0.35;
// Velocidad de parpadeo de los destellos
const SPARKLE_SPEED: f32 = 2.2;
// Fuerza del aura en el borde
const AURA_STRENGTH: f32 = 1.6;
// Exposición del tone mapping (más = estrella más luminosa)
const EXPOSURE: f32 = 2.0;

const WHITE: Vector3 = Vector3::new(0.93, 0.97, 1.0);
const ICE: Vector3 = Vector3::new(0.4, 0.82, 1.0);
const DEEP: Vector3 = Vector3::new(0.1, 0.45, 0.9);

// Número pseudoaleatorio en [0, 1) a partir de una celda entera
fn hash(cell: Vector3, seed: f32) -> f32 {
    let d = cell.x * 127.1 + cell.y * 311.7 + cell.z * 74.7 + seed * 53.3;
    (d.sin() * 43758.547).fract().abs()
}

fn hash3(cell: Vector3) -> Vector3 {
    Vector3::new(hash(cell, 0.0), hash(cell, 1.0), hash(cell, 2.0))
}

// Ruido celular (Voronoi): distancia al punto más cercano, al segundo, y la celda ganadora
fn cellular(p: Vector3) -> (f32, f32, Vector3) {
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

// Color iridiscente (arcoíris pastel) según un tono en [0, 1)
fn iridescent(hue: f32) -> Vector3 {
    let tau = std::f32::consts::TAU;
    let rainbow = Vector3::new(
        0.5 + 0.5 * (tau * hue).cos(),
        0.5 + 0.5 * (tau * (hue + 0.33)).cos(),
        0.5 + 0.5 * (tau * (hue + 0.67)).cos(),
    );
    mix(rainbow, WHITE, 0.35)
}

// Destello de una capa de celdas: cada celda parpadea con su propio ritmo
fn sparkle_layer(p: Vector3, scale: f32, time: f32, seed: f32) -> f32 {
    let (f1, _, id) = cellular(p * scale);
    let phase = hash(id, seed) * std::f32::consts::TAU;
    let speed = SPARKLE_SPEED * (0.5 + hash(id, seed + 2.0));
    let twinkle = (time * speed + phase).sin().max(0.0).powf(10.0);
    // núcleo intenso + halo suave alrededor
    twinkle * ((1.0 - smoothstep(0.0, 0.08, f1)) * 1.6 + (1.0 - smoothstep(0.0, 0.4, f1)) * 0.5)
}

pub fn crystal_star(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_position;
    let n = fragment.normal;
    let time = uniforms.time;
    let view = (uniforms.camera_position - fragment.world_position).normalized();
    let n_dot_v = n.dot(view).max(0.0);

    // 1. Parallax: el interior "vive" debajo de la superficie. Se desplaza el punto de
    //    muestreo en contra de la vista, según un campo de altura procedural.
    let height = fbm(&uniforms.noise, p * 2.5);
    let view_tangent = view - n * view.dot(n);
    let inner = p - view_tangent * (PARALLAX_DEPTH * (1.0 - height) / n_dot_v.max(0.35));

    // 2. Superficie activa: plasma interno que fluye con el tiempo (zonas más calientes)
    let drift = Vector3::new(time * 0.15, -time * 0.1, time * 0.08);
    let plasma = fbm(&uniforms.noise, inner * 3.0 + drift);
    let heat = smoothstep(0.35, 0.75, plasma);
    let veins = 1.0 - (fbm(&uniforms.noise, inner * 4.0 - drift * 0.5) * 2.0 - 1.0).abs();
    let veins = smoothstep(0.85, 0.98, veins);
    let body = mix(DEEP, ICE, heat);
    let body = mix(body, WHITE, veins * 0.6 + heat * heat * 0.2);

    // Núcleo blanco caliente al centro, celeste hacia el borde
    let core = n_dot_v.powf(2.0);
    let body = mix(body, WHITE, core * 0.25) * (0.9 + core * 0.3);

    // 3. Facetas: cada celda tiene su propia normal inclinada; aristas suaves
    let (f1, f2, id) = cellular(p * FACET_SCALE);
    let edge = 1.0 - smoothstep(0.0, 0.035, f2 - f1);
    let facet_normal = (n + (hash3(id) * 2.0 - Vector3::one()) * FACET_TILT).normalized();

    // 4. Reflejo de colores: brillo especular por faceta, teñido según la faceta y el ángulo de vista
    let half = (uniforms.light_dir + view).normalized();
    let specular = facet_normal.dot(half).max(0.0).powf(40.0);
    let glint = facet_normal.dot(half).max(0.0).powf(6.0) * 0.35;
    let tint = iridescent(hash(id, 4.0) + n_dot_v * 0.6 + time * 0.05);
    let reflection = tint * (specular * 1.4 + glint);

    // 5. Destellos (animados): facetas grandes + brillo fino
    let sparkle = sparkle_layer(p, FACET_SCALE, time, 7.0) + sparkle_layer(p, 13.0, time * 1.3, 11.0) * 0.8;
    let sparkle_color = mix(WHITE, iridescent(hash(id, 5.0)), 0.3);

    // 6. Aura: resplandor tipo Fresnel en el borde de la silueta
    let fresnel = (1.0 - n_dot_v).powf(2.0);
    let aura = mix(ICE, WHITE, fresnel) * fresnel * AURA_STRENGTH;

    // Emisiva: no depende de la iluminación difusa externa
    let pulse = 0.92 + 0.08 * (time * 1.3).sin();
    let color = body * pulse + WHITE * (edge * 0.12) + reflection + sparkle_color * sparkle + aura;

    // Tone mapping suave: brillo intenso sin quemar los tonos a blanco plano
    Vector3::new(
        1.0 - (-color.x * EXPOSURE).exp(),
        1.0 - (-color.y * EXPOSURE).exp(),
        1.0 - (-color.z * EXPOSURE).exp(),
    )
}
