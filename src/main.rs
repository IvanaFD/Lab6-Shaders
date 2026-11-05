mod camera;
mod fragment;
mod framebuffer;
mod obj;
mod pipeline;
mod shaders;
mod triangle;
mod vertex;

use noise::OpenSimplex;
use raylib::prelude::*;

use camera::OrbitCamera;
use framebuffer::Framebuffer;
use obj::load_obj;
use shaders::{Body, fragment_shader};
use vertex::Vertex;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const MOON_SIZE: f32 = 0.2;
const MOON_ORBIT: f32 = 1.45;
const MOON_SPEED: f32 = 0.3;

pub struct Uniforms {
    pub model_matrix: Matrix,
    pub view_matrix: Matrix,
    pub projection_matrix: Matrix,
    pub width: usize,
    pub height: usize,
    pub time: f32,
    pub noise: OpenSimplex,
    pub light_dir: Vector3,
    pub camera_position: Vector3,
    pub body: Body,
}

fn render(framebuffer: &mut Framebuffer, vertices: &[Vertex], uniforms: &mut Uniforms, objects: &[(Matrix, Body)]) {
    framebuffer.clear();

    let mut visible: Vec<(usize, fragment::Fragment, Body)> = Vec::new();
    for &(model_matrix, body) in objects {
        uniforms.model_matrix = model_matrix;
        let transformed: Vec<Vertex> = vertices
            .iter()
            .map(|v| pipeline::vertex_shader(v, uniforms))
            .collect();
        for tri in transformed.chunks_exact(3) {
            if pipeline::is_back_face(tri, uniforms) {
                continue;
            }
            for fragment in triangle::triangle(&tri[0], &tri[1], &tri[2], uniforms) {
                let index = fragment.position.y as usize * framebuffer.width + fragment.position.x as usize;
                if fragment.depth < framebuffer.depths[index] {
                    framebuffer.depths[index] = fragment.depth;
                    visible.push((index, fragment, body));
                }
            }
        }
    }
    visible.retain(|(index, fragment, _)| fragment.depth == framebuffer.depths[*index]);

    let uniforms: &Uniforms = uniforms;
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunk = visible.len().div_ceil(threads).max(1);
    let colors: Vec<Vec<Vector3>> = std::thread::scope(|scope| {
        let handles: Vec<_> = visible
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || part.iter().map(|(_, f, body)| fragment_shader(f, uniforms, *body)).collect())
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for ((index, _, _), color) in visible.iter().zip(colors.into_iter().flatten()) {
        framebuffer.set_color(*index, color);
    }
}

// El cuerpo elegido y, si es el planeta rocoso, su luna en órbita
fn scene(base: Matrix, body: Body, time: f32) -> Vec<(Matrix, Body)> {
    let mut objects = vec![(base, body)];
    if body == Body::Rocky {
        let angle = time * MOON_SPEED;
        let position = Vector3::new(angle.cos() * MOON_ORBIT, angle.sin() * MOON_ORBIT * 0.2, angle.sin() * MOON_ORBIT);
        let moon = base * Matrix::scale(MOON_SIZE, MOON_SIZE, MOON_SIZE) * Matrix::translate(position.x, position.y, position.z);
        objects.push((moon, Body::Moon));
    }
    objects
}

fn light_direction(angle: f32) -> Vector3 {
    Vector3::new(angle.sin(), 0.35, angle.cos()).normalized()
}

fn main() {
    let obj = match load_obj("assets/sphere.obj") {
        Ok(obj) => obj,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };

    let (center, _) = obj.bounds();
    let radius = obj.radius(center);
    let vertices = obj.get_vertex_array(center);
    println!(
        "Esfera cargada: {} vértices, {} triángulos",
        obj.vertices.len(),
        vertices.len() / 3
    );
    let model_matrix =
        Matrix::translate(-center.x, -center.y, -center.z) * Matrix::scale(1.0 / radius, 1.0 / radius, 1.0 / radius);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    framebuffer.set_background_color(Vector3::new(0.01, 0.01, 0.03));

    let mut camera = OrbitCamera::new();
    let mut light_angle: f32 = 0.6;
    let mut uniforms = Uniforms {
        model_matrix,
        view_matrix: camera.view_matrix(),
        projection_matrix: Matrix::perspective(45f32.to_radians(), WIDTH as f32 / HEIGHT as f32, 0.1, 100.0),
        width: WIDTH,
        height: HEIGHT,
        time: 0.0,
        noise: OpenSimplex::new(7),
        light_dir: light_direction(light_angle),
        camera_position: camera.eye(),
        body: Body::Star,
    };

    if std::env::args().any(|arg| arg == "--captura") {
        let _ = std::fs::create_dir_all("capturas");
        uniforms.time = 3.0;
        for (body, file) in [
            (Body::Star, "estrella"),
            (Body::Rocky, "rocoso"),
            (Body::GasGiant, "gaseoso"),
        ] {
            uniforms.body = body;
            for (angle, suffix) in [(light_angle, ""), (2.2, "_lado_oscuro")] {
                uniforms.light_dir = light_direction(angle);
                let start = std::time::Instant::now();
                let objects = scene(model_matrix, uniforms.body, uniforms.time);
                render(&mut framebuffer, &vertices, &mut uniforms, &objects);
                if suffix.is_empty() {
                    println!("{}: {:.1} ms", body.name(), start.elapsed().as_secs_f32() * 1000.0);
                }
                framebuffer.render_to_file(&format!("capturas/{file}{suffix}.png"));
            }
        }
        println!("Capturas guardadas en capturas/");
        return;
    }

    let (mut window, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Lab 6 - Shaders")
        .build();
    window.set_target_fps(60);
    framebuffer.init_texture(&mut window, &thread);

    let mut paused = false;

    while !window.window_should_close() {
        let dt = window.get_frame_time();

        if window.is_key_pressed(KeyboardKey::KEY_ONE) {
            uniforms.body = Body::Star;
        }
        if window.is_key_pressed(KeyboardKey::KEY_TWO) {
            uniforms.body = Body::Rocky;
        }
        if window.is_key_pressed(KeyboardKey::KEY_THREE) {
            uniforms.body = Body::GasGiant;
        }

        let step = 1.5 * dt;
        if window.is_key_down(KeyboardKey::KEY_LEFT) || window.is_key_down(KeyboardKey::KEY_A) {
            camera.orbit(-step, 0.0);
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) || window.is_key_down(KeyboardKey::KEY_D) {
            camera.orbit(step, 0.0);
        }
        if window.is_key_down(KeyboardKey::KEY_UP) || window.is_key_down(KeyboardKey::KEY_W) {
            camera.orbit(0.0, step);
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) || window.is_key_down(KeyboardKey::KEY_S) {
            camera.orbit(0.0, -step);
        }
        if window.is_key_down(KeyboardKey::KEY_Q) {
            camera.zoom(-2.0 * dt);
        }
        if window.is_key_down(KeyboardKey::KEY_E) {
            camera.zoom(2.0 * dt);
        }
        if window.is_key_pressed(KeyboardKey::KEY_R) {
            camera = OrbitCamera::new();
            light_angle = 0.6;
        }

        if window.is_key_down(KeyboardKey::KEY_J) {
            light_angle -= step;
        }
        if window.is_key_down(KeyboardKey::KEY_L) {
            light_angle += step;
        }

        if window.is_key_pressed(KeyboardKey::KEY_SPACE) {
            paused = !paused;
        }
        if !paused {
            uniforms.time += dt;
        }

        uniforms.view_matrix = camera.view_matrix();
        uniforms.camera_position = camera.eye();
        uniforms.light_dir = light_direction(light_angle);

        let objects = scene(model_matrix, uniforms.body, uniforms.time);
        render(&mut framebuffer, &vertices, &mut uniforms, &objects);

        let hud = vec![
            format!("{}  |  FPS {}", uniforms.body.name(), window.get_fps()),
            "1/2/3 cuerpo   WASD/flechas orbitar   Q/E zoom".to_string(),
            "J/L mover luz   Espacio pausa   R reiniciar".to_string(),
        ];
        framebuffer.swap_buffers(&mut window, &thread, &hud);
    }
}
