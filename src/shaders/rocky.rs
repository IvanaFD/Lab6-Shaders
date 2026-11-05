// Planeta volcánico: piedra y ceniza mate, obsidiana vidriosa, lava que brilla,
// grietas incandescentes en el lado nocturno y nubes de ceniza con sus sombras.
use super::{
    mix,
    noise_helpers::{cellular, fbm, fbm_octaves, hash, hash3, sample},
    smoothstep,
};
use crate::{Uniforms, fragment::Fragment};
use noise::OpenSimplex;
use raylib::prelude::*;

// ---- Parámetros de apariencia ----
// Nivel del "mar" de lava (más = más superficie cubierta de lava)
const LAVA_LEVEL: f32 = 0.38;
// Ancho de los ríos de lava
const RIVER_WIDTH: f32 = 0.018;
// Ancho de la obsidiana alrededor de la lava (lava enfriada)
const OBSIDIAN_WIDTH: f32 = 0.04;
// Brillo de la obsidiana (exponente especular: más = reflejo más pequeño y nítido)
const OBSIDIAN_SHININESS: f32 = 110.0;
// Ancho de las grietas incandescentes del lado nocturno
const CRACK_WIDTH: f32 = 0.035;
// Rugosidad fina de la tierra en el relieve (0 = lisa)
const SOIL_ROUGHNESS: f32 = 1.0;
// Tamaño de las placas de tierra seca agrietada (más = placas más pequeñas)
const CRACK_CELL_SCALE: f32 = 18.0;
// Tamaño de las celdas de volcanes y fracción de celdas que tienen volcán
const VOLCANO_SCALE: f32 = 2.2;
const VOLCANO_DENSITY: f32 = 0.65;
// Fuerza del relieve (bump mapping)
const BUMP_STRENGTH: f32 = 1.6;
// Cobertura de nubes de ceniza (0 = cielo limpio, 1 = cubierto)
const CLOUD_COVERAGE: f32 = 0.45;
// Velocidad de giro de las nubes (radianes por segundo)
const CLOUD_SPEED: f32 = 0.06;
// Altura de las nubes sobre la superficie (controla cuánto se separa la sombra)
const CLOUD_HEIGHT: f32 = 0.06;

const LAVA_HOT: Vector3 = Vector3::new(1.0, 0.82, 0.3);
const LAVA: Vector3 = Vector3::new(0.95, 0.28, 0.03);
const CRUST: Vector3 = Vector3::new(0.22, 0.04, 0.02);
const OBSIDIAN: Vector3 = Vector3::new(0.035, 0.03, 0.045);
const DARK_ROCK: Vector3 = Vector3::new(0.15, 0.14, 0.13);
const STONE: Vector3 = Vector3::new(0.33, 0.31, 0.29);
const OCHRE: Vector3 = Vector3::new(0.42, 0.37, 0.32);
const ASH: Vector3 = Vector3::new(0.46, 0.45, 0.43);
const CLOUD: Vector3 = Vector3::new(0.7, 0.69, 0.68);
const SMOKE: Vector3 = Vector3::new(0.42, 0.41, 0.4);

// Volcanes: algunas celdas de una malla 3D tienen un volcán en un punto aleatorio,
// cada uno con su tamaño. Se revisan las celdas vecinas para que los conos no se corten.
// Devuelve (cono, cráter) en [0, 1].
fn volcano(p: Vector3) -> (f32, f32) {
    let q = p * VOLCANO_SCALE;
    let base = Vector3::new(q.x.floor(), q.y.floor(), q.z.floor());
    let mut cone: f32 = 0.0;
    let mut crater: f32 = 0.0;
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                let cell = base + Vector3::new(x as f32, y as f32, z as f32);
                if hash(cell, 3.0) > VOLCANO_DENSITY {
                    continue;
                }
                let size = 0.6 + hash(cell, 5.0) * 0.35;
                let d = (cell + hash3(cell) - q).length();
                cone = cone.max(1.0 - smoothstep(0.0, size, d));
                crater = crater.max(1.0 - smoothstep(size * 0.14, size * 0.24, d));
            }
        }
    }
    (cone, crater)
}

// Relieve base, como continentes: ruido fbm con el dominio deformado por otro ruido,
// así las costas de los mares de lava quedan irregulares y naturales.
fn terrain(noise: &OpenSimplex, p: Vector3) -> f32 {
    let warp = Vector3::new(
        sample(noise, p * 1.3 + Vector3::new(3.1, 0.0, 0.0)),
        sample(noise, p * 1.3 + Vector3::new(0.0, 7.7, 0.0)),
        sample(noise, p * 1.3 + Vector3::new(0.0, 0.0, 1.9)),
    ) * 0.45;
    fbm(noise, p * 1.8 + warp)
}

// Ríos de lava: el cauce sigue la línea media de un fbm deformado (fractal, se ramifica).
// Se calcula una sola vez por píxel: (cauce principal, afluentes, variación de ancho).
struct RiverField {
    main: f32,
    branch: f32,
    width_noise: f32,
}

fn river_field(noise: &OpenSimplex, p: Vector3) -> RiverField {
    let warp = Vector3::new(sample(noise, p * 2.2 + Vector3::new(9.0, 0.0, 0.0)), 0.0, sample(noise, p * 2.2 - Vector3::new(0.0, 4.0, 0.0))) * 0.3;
    RiverField {
        main: (fbm_octaves(noise, p * 2.4 + warp, 3) - 0.5).abs(),
        branch: (fbm_octaves(noise, p * 5.0 + warp * 2.0 + Vector3::new(2.0, 2.0, 2.0), 2) - 0.5).abs(),
        width_noise: fbm_octaves(noise, p * 4.0, 2),
    }
}

// Cuánto río hay en el punto, en [0, 1]. El ancho cambia a lo largo del río y crece
// al acercarse al mar de lava; `extra` ensancha (se usa para la orilla de obsidiana).
fn river(field: &RiverField, land: f32, extra: f32) -> f32 {
    let width = (RIVER_WIDTH + extra) * (0.5 + field.width_noise) * (1.0 + 1.5 * (1.0 - smoothstep(0.0, 0.08, land)));
    // los ríos nacen en tierras medias y no suben a las zonas altas
    let fade = 1.0 - smoothstep(0.08, 0.16, land);
    let branch_fade = 1.0 - smoothstep(0.03, 0.09, land);
    let main_river = 1.0 - smoothstep(width * 0.6, width, field.main);
    let side_river = 1.0 - smoothstep(width * 0.3, width * 0.5, field.branch);
    (main_river * fade).max(side_river * branch_fade)
}

// Campo de altura para el relieve: conos volcánicos con cráter + rugosidad de la roca.
// (El terreno de los continentes es de baja frecuencia: casi no aporta relieve visible,
// así que se deja fuera para que el bump sea barato.)
fn height(noise: &OpenSimplex, p: Vector3) -> f32 {
    let (cone, crater) = volcano(p);
    cone.powf(1.5) * 0.5 - crater * 0.3
        + fbm_octaves(noise, p * 5.0, 2) * 0.15
        // rugosidad fina de la tierra: la luz revela el grano
        + fbm_octaves(noise, p * 22.0, 2) * 0.035 * SOIL_ROUGHNESS
}

// Gira un punto alrededor del eje y
fn rotate_y(p: Vector3, angle: f32) -> Vector3 {
    let (s, c) = angle.sin_cos();
    Vector3::new(p.x * c + p.z * s, p.y, -p.x * s + p.z * c)
}

// Máscara de nubes de ceniza; giran con su propia velocidad, independientes del terreno
fn cloud_mask(noise: &OpenSimplex, p: Vector3, time: f32) -> f32 {
    let q = rotate_y(p, time * CLOUD_SPEED);
    // deformación lenta para que cambien de forma
    let warp = Vector3::new(0.0, sample(noise, q * 1.5 + Vector3::new(time * 0.05, 0.0, 0.0)) * 0.3, 0.0);
    let density = fbm_octaves(noise, q * Vector3::new(2.2, 4.0, 2.2) + warp, 3);
    let threshold = 0.72 - CLOUD_COVERAGE * 0.4;
    smoothstep(threshold, threshold + 0.08, density)
}

// Base tangente en un punto de la esfera
fn tangent_basis(n: Vector3) -> (Vector3, Vector3) {
    let up = if n.y.abs() > 0.99 { Vector3::new(1.0, 0.0, 0.0) } else { Vector3::new(0.0, 1.0, 0.0) };
    let t = up.cross(n).normalized();
    (t, n.cross(t))
}

pub fn volcanic(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let noise = &uniforms.noise;
    let p = fragment.object_position;
    let n = fragment.normal;
    let time = uniforms.time;
    let light = uniforms.light_dir;
    // La esfera es unitaria y está en el origen: p es el punto exacto de la superficie
    let view = (uniforms.camera_position - p).normalized();

    // 1. Relieve: el gradiente del campo de altura inclina la normal (bump mapping)
    let h = height(noise, p);
    let (t1, t2) = tangent_basis(p);
    let eps = 0.01;
    let dh1 = (height(noise, (p + t1 * eps).normalized()) - h) / eps;
    let dh2 = (height(noise, (p + t2 * eps).normalized()) - h) / eps;
    let bumped = (n - (t1 * dh1 + t2 * dh2) * (BUMP_STRENGTH * 0.1)).normalized();

    let diffuse = bumped.dot(light).max(0.0);
    let half = (light + view).normalized();

    // 2. Regiones (como un planeta cuyos océanos y ríos se volvieron lava)
    let t = terrain(noise, p);
    let land = t - LAVA_LEVEL;
    let (cone, crater) = volcano(p);
    let on_land = smoothstep(0.02, 0.08, land);
    let cone = cone * on_land;
    let crater = crater * on_land;
    //    Lava: mares y lagos en lo bajo, ríos que bajan hacia ellos y el interior de los cráteres
    let sea = 1.0 - smoothstep(-0.005, 0.005, land);
    let rivers = river_field(noise, p);
    let stream = river(&rivers, land, 0.0);
    let lava_amount = sea.max(stream).max(smoothstep(0.6, 0.9, crater));
    //    Obsidiana: lava enfriada en las costas (ancho irregular), orillas de ríos y cráteres
    let grain = fbm_octaves(noise, p * 6.0, 3);
    let coast_width = OBSIDIAN_WIDTH * (0.4 + grain * 1.2);
    let near_sea = 1.0 - smoothstep(coast_width * 0.5, coast_width, land);
    let near_river = river(&rivers, land, OBSIDIAN_WIDTH * 0.35);
    let obsidian_amount = near_sea.max(near_river).max(smoothstep(0.2, 0.5, crater));
    let hills = t + 0.5 - LAVA_LEVEL;
    //    Ceniza: cubre las partes altas y las laderas de los volcanes
    let ash_amount = smoothstep(0.52, 0.68, hills + fbm_octaves(noise, p * 9.0, 2) * 0.3 - 0.12);

    // 3. Cada material responde distinto a la luz
    // Piedra y ceniza: mate (solo difusa)
    // Tierra en varias escalas: manchas grandes, parches ocres, suelo agrietado y grano fino
    let rock = mix(DARK_ROCK, STONE, smoothstep(0.3, 0.7, grain));
    let patches = fbm_octaves(noise, p * 16.0 + Vector3::new(4.0, 1.0, 7.0), 2);
    let rock = mix(rock, OCHRE, smoothstep(0.5, 0.75, patches) * 0.5);
    //   tierra seca agrietada: bordes entre celdas (ruido celular), más marcada en las planicies
    let (c1, c2, _) = cellular(p * CRACK_CELL_SCALE);
    let mud_crack = (1.0 - smoothstep(0.0, 0.06, c2 - c1)) * (1.0 - smoothstep(0.1, 0.3, cone));
    let rock = rock * (1.0 - mud_crack * 0.55);
    //   grano fino tipo grava
    let speckle = sample(noise, p * 70.0) * 0.5 + 0.5;
    let rock = rock * (0.82 + speckle * 0.3);
    let rock = mix(rock, ASH * (0.85 + speckle * 0.25), ash_amount);
    // Laderas de los volcanes: basalto oscuro con vetas de ceniza que bajan del cráter
    let streaks = smoothstep(0.55, 0.75, fbm_octaves(noise, p * 14.0, 2));
    let rock = mix(rock, mix(DARK_ROCK * 0.7, ASH, streaks * 0.6), smoothstep(0.1, 0.5, cone));
    let rock_lit = rock * (0.04 + diffuse * 0.96);
    // Obsidiana: casi negra, reflejo especular nítido + reflejo de borde (Fresnel)
    let fresnel = (1.0 - bumped.dot(view).max(0.0)).powf(4.0);
    let spec = bumped.dot(half).max(0.0).powf(OBSIDIAN_SHININESS) * smoothstep(0.0, 0.1, diffuse);
    let obsidian_lit = OBSIDIAN * (0.04 + diffuse) + Vector3::new(0.85, 0.8, 1.0) * (spec * 1.6)
        + Vector3::new(0.35, 0.25, 0.45) * fresnel * diffuse.max(0.15) * 0.5;
    let mut surface = mix(rock_lit, obsidian_lit, obsidian_amount);

    // 4. Sombras de nubes: se busca la nube que tapa la luz sobre este punto
    let shadow_point = (p + light * (CLOUD_HEIGHT / n.dot(light).max(0.15))).normalized();
    let shadow = cloud_mask(noise, shadow_point, time);
    surface = surface * (1.0 - shadow * 0.85);

    // 5. Lava: emisiva (no depende de la luz); su corteza fluye con el tiempo
    let flow = fbm_octaves(noise, p * 7.0 + Vector3::new(time * 0.12, time * 0.05, -time * 0.08), 3);
    let lava_color = mix(CRUST, LAVA, smoothstep(0.35, 0.6, flow));
    let lava_color = mix(lava_color, LAVA_HOT, smoothstep(0.62, 0.8, flow));
    surface = mix(surface, lava_color, lava_amount);
    // Resplandor de la lava sobre el borde del cráter
    let rim_glow = smoothstep(0.0, 0.6, crater) * (1.0 - smoothstep(0.6, 0.9, crater));
    surface = surface + LAVA * rim_glow * 0.35;

    // 6. Lado nocturno: grietas incandescentes que solo se ven sin luz
    let night = 1.0 - smoothstep(-0.05, 0.2, n.dot(light));
    let crack = 1.0 - smoothstep(0.0, CRACK_WIDTH, sample(noise, p * 8.0).abs());
    let crack = crack * (1.0 - lava_amount) * (1.0 - ash_amount * 0.7);
    surface = surface + LAVA * crack * night * 1.2;

    // 7. Nubes de ceniza encima; de noche las ilumina el resplandor de la lava
    let clouds = cloud_mask(noise, p, time);
    let cloud_light = fragment.intensity;
    // tonos de gris: humo más oscuro donde la nube es más espesa
    let thickness = smoothstep(0.4, 0.8, fbm_octaves(noise, rotate_y(p, time * CLOUD_SPEED) * 6.0, 2));
    let cloud_color = mix(CLOUD, SMOKE, thickness) * (0.05 + cloud_light * 0.9) + LAVA * (lava_amount * 0.25 + night * 0.04);
    mix(surface, cloud_color, clouds * 0.9)
}
