// Gigante gaseoso "refresco de cola": bandas oscuras que fluyen como líquido,
// burbujas de gas que suben y revientan, espuma en el polo, un remolino que curva
// las bandas y una franja roja tipo etiqueta con una cinta blanca ondulada.
use super::{
    mix,
    noise_helpers::{cellular, fbm_octaves, hash},
    smoothstep,
};
use crate::{Uniforms, fragment::Fragment};
use raylib::prelude::*;

// ---- Parámetros de apariencia ----
// Cantidad de bandas (frecuencia en latitud)
const BAND_FREQUENCY: f32 = 16.0;
// Cuánto deforma el ruido a las bandas (más = transiciones más turbulentas)
const TURBULENCE: f32 = 2.6;
// Velocidad del flujo de las bandas
const FLOW_SPEED: f32 = 0.12;
// Tamaño de las burbujas (más = burbujas más pequeñas y numerosas)
const BUBBLE_SCALE: f32 = 9.0;
// Velocidad a la que suben las burbujas y fracción de celdas con burbuja
const BUBBLE_RISE: f32 = 0.35;
const BUBBLE_DENSITY: f32 = 0.75;
// Suavidad del borde de las burbujas (más = borde más difuminado)
const BUBBLE_SOFTNESS: f32 = 0.085;
// Remolino: centro, radio e intensidad del giro
const STORM_RADIUS: f32 = 0.38;
const STORM_STRENGTH: f32 = 5.0;
// Brillo dorado de las burbujas en el lado oscuro
const GLOW_STRENGTH: f32 = 1.3;
// Medio ancho de la franja roja
const LABEL_WIDTH: f32 = 0.13;

const COLA_BLACK: Vector3 = Vector3::new(0.03, 0.012, 0.008);
const COLA_DARK: Vector3 = Vector3::new(0.16, 0.05, 0.02);
const COLA_MID: Vector3 = Vector3::new(0.28, 0.11, 0.04);
const CARAMEL: Vector3 = Vector3::new(0.45, 0.2, 0.07);
const GOLD: Vector3 = Vector3::new(1.0, 0.72, 0.22);
const FOAM: Vector3 = Vector3::new(0.92, 0.85, 0.72);
const RED: Vector3 = Vector3::new(0.85, 0.02, 0.05);
const WHITE: Vector3 = Vector3::new(0.97, 0.97, 0.97);

fn storm_center() -> Vector3 {
    Vector3::new(0.55, -0.42, 0.72).normalized()
}

// Rota el punto p alrededor del eje (normalizado) un ángulo dado (fórmula de Rodrigues)
fn rotate_axis(p: Vector3, axis: Vector3, angle: f32) -> Vector3 {
    let (s, c) = angle.sin_cos();
    p * c + axis.cross(p) * s + axis * (axis.dot(p) * (1.0 - c))
}

// Remolino: gira las coordenadas alrededor del centro, más fuerte cerca de él.
// Devuelve el punto deformado y qué tan dentro del remolino está (0 afuera, 1 en el ojo).
fn vortex(p: Vector3, time: f32) -> (Vector3, f32) {
    let center = storm_center();
    let d = (p - center).length();
    let inside = (1.0 - smoothstep(0.0, STORM_RADIUS, d)).powf(2.0);
    let angle = inside * (STORM_STRENGTH + time * 0.8);
    (rotate_axis(p, center, angle), inside)
}

// Burbujas: cada celda (que sube con el tiempo) tiene una burbuja que crece y revienta.
// Devuelve (anillo, interior, halo alrededor).
fn bubbles(p: Vector3, time: f32) -> (f32, f32, f32) {
    let q = p * BUBBLE_SCALE - Vector3::new(0.0, time * BUBBLE_RISE * BUBBLE_SCALE * 0.3, 0.0);
    let (f1, _, id) = cellular(q);
    if hash(id, 1.0) > BUBBLE_DENSITY {
        return (0.0, 0.0, 0.0);
    }
    // ciclo de vida propio: nace, crece y revienta
    let life = (time * (0.3 + hash(id, 2.0) * 0.5) + hash(id, 3.0)).fract();
    let radius = 0.08 + life * (0.18 + hash(id, 4.0) * 0.15);
    let fade = 1.0 - smoothstep(0.85, 1.0, life);
    // borde difuminado: caída gaussiana alrededor del radio en lugar de un corte duro
    let d = (f1 - radius) / BUBBLE_SOFTNESS;
    let ring = (-d * d).exp() * fade;
    let interior = (1.0 - smoothstep(radius * 0.3, radius * 1.05, f1)) * fade;
    let halo = (1.0 - smoothstep(radius * 0.8, radius + 0.25, f1)) * fade;
    (ring, interior, halo)
}

pub fn cola_giant(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let noise = &uniforms.noise;
    let p = fragment.object_position;
    let n = fragment.normal;
    let time = uniforms.time;
    let light = uniforms.light_dir;
    // La esfera es unitaria y está en el origen: p es el punto exacto de la superficie
    let view = (uniforms.camera_position - p).normalized();

    // 1. Remolino: deforma las coordenadas antes de calcular las bandas
    let (q, storm) = vortex(p, time);

    // 2. Flujo líquido: cada latitud gira a distinta velocidad y el ruido deforma las bandas
    let shear = (q.y * 5.0).sin() * FLOW_SPEED * time;
    let (s, c) = shear.sin_cos();
    let flowing = Vector3::new(q.x * c + q.z * s, q.y, -q.x * s + q.z * c);
    let warp = fbm_octaves(noise, flowing * 2.5 + Vector3::new(time * 0.05, 0.0, 0.0), 3) * 2.0 - 1.0;
    let fine = fbm_octaves(noise, flowing * 7.0 - Vector3::new(0.0, 0.0, time * 0.08), 2) * 2.0 - 1.0;
    let band = (q.y * BAND_FREQUENCY + warp * TURBULENCE + fine * 0.6).sin() * 0.5 + 0.5;
    // Gradación suave de cafés: transiciones anchas + variación lenta de tono
    let tone = band * 0.75 + fbm_octaves(noise, flowing * 1.5, 2) * 0.35 - 0.05;
    let mut color = mix(COLA_BLACK, COLA_DARK, smoothstep(0.0, 0.55, tone));
    color = mix(color, COLA_MID, smoothstep(0.35, 0.85, tone) * 0.8);
    color = mix(color, CARAMEL, smoothstep(0.7, 1.1, tone) * 0.45);

    // Ojo del remolino: espiral color caramelo con centro negro
    color = mix(color, CARAMEL * 0.8, smoothstep(0.3, 0.7, storm) * smoothstep(0.4, 1.0, band) * 0.8);
    color = mix(color, COLA_BLACK, smoothstep(0.85, 1.0, storm));

    // 3. Espuma en el polo norte, con borde irregular
    let foam = smoothstep(0.72, 0.82, p.y + (fbm_octaves(noise, p * 6.0 + Vector3::new(0.0, time * 0.1, 0.0), 2) - 0.5) * 0.3);
    color = mix(color, FOAM, foam);

    // 4. Burbujas de gas (blancas por fuera, oscuras por dentro)
    let (ring, interior, halo) = bubbles(p, time);
    let bubble_zone = 1.0 - foam;
    color = mix(color, COLA_BLACK, interior * 0.45 * bubble_zone);
    color = mix(color, WHITE * 0.9, ring * 0.6 * bubble_zone);

    // 5. Franja roja tipo etiqueta con cinta blanca ondulada
    let longitude = p.z.atan2(p.x);
    let label = 1.0 - smoothstep(LABEL_WIDTH - 0.01, LABEL_WIDTH, p.y.abs());
    let wave_y = (longitude * 2.0).sin() * 0.05;
    let ribbon = 1.0 - smoothstep(0.012, 0.022, (p.y - wave_y).abs());
    let label_color = mix(RED, WHITE, ribbon);
    color = mix(color, label_color, label);

    // 6. Iluminación: difusa + brillo de líquido (especular) + borde de atmósfera
    let diffuse = fragment.intensity;
    let half = (light + view).normalized();
    let gloss = 1.0 - label * 0.6;
    let specular = n.dot(half).max(0.0).powf(60.0) * 0.6 * gloss;
    let rim = (1.0 - n.dot(view).max(0.0)).powf(3.0);
    let lit = color * (0.06 + diffuse * 0.94) + Vector3::new(1.0, 0.95, 0.9) * specular + CARAMEL * rim * (0.2 + diffuse * 0.5);

    // 7. Lado oscuro: las burbujas de gas brillan con luz dorada propia (solo donde no llega luz)
    let night = 1.0 - smoothstep(-0.05, 0.25, n.dot(light));
    let glow = (ring * 0.9 + interior * 0.3 + halo * 0.3) * bubble_zone * (1.0 - label);
    lit + GOLD * glow * night * GLOW_STRENGTH
}
