use raylib::prelude::*;

// Origen arriba a la izquierda, y hacia abajo
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    background_color: Vector3,
    // RGBA, 4 bytes por píxel
    color_buffer: Vec<u8>,
    pub depths: Vec<f32>,
    texture: Option<Texture2D>,
}

fn to_rgba(color: Vector3) -> [u8; 4] {
    [
        (color.x.clamp(0.0, 1.0) * 255.0) as u8,
        (color.y.clamp(0.0, 1.0) * 255.0) as u8,
        (color.z.clamp(0.0, 1.0) * 255.0) as u8,
        255,
    ]
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut framebuffer = Framebuffer {
            width,
            height,
            background_color: Vector3::zero(),
            color_buffer: vec![0; width * height * 4],
            depths: vec![f32::INFINITY; width * height],
            texture: None,
        };
        framebuffer.clear();
        framebuffer
    }

    pub fn set_background_color(&mut self, color: Vector3) {
        self.background_color = color;
    }

    pub fn clear(&mut self) {
        let rgba = to_rgba(self.background_color);
        for pixel in self.color_buffer.chunks_exact_mut(4) {
            pixel.copy_from_slice(&rgba);
        }
        self.depths.fill(f32::INFINITY);
    }

    pub fn set_color(&mut self, index: usize, color: Vector3) {
        self.color_buffer[index * 4..index * 4 + 4].copy_from_slice(&to_rgba(color));
    }

    pub fn init_texture(&mut self, window: &mut RaylibHandle, thread: &RaylibThread) {
        let image = Image::gen_image_color(self.width as i32, self.height as i32, Color::BLACK);
        self.texture = window.load_texture_from_image(thread, &image).ok();
    }

    pub fn swap_buffers(&mut self, window: &mut RaylibHandle, thread: &RaylibThread, hud: &[String]) {
        let Some(texture) = self.texture.as_mut() else {
            return;
        };
        let _ = texture.update_texture(&self.color_buffer);

        let screen_width = window.get_screen_width() as f32;
        let screen_height = window.get_screen_height() as f32;

        let mut render = window.begin_drawing(thread);
        render.draw_texture_pro(
            &*texture,
            Rectangle::new(0.0, 0.0, self.width as f32, self.height as f32),
            Rectangle::new(0.0, 0.0, screen_width, screen_height),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );
        for (i, line) in hud.iter().enumerate() {
            render.draw_text(line, 10, 10 + i as i32 * 18, 16, Color::new(200, 210, 230, 255));
        }
    }

    // Exportar a PNG
    pub fn render_to_file(&self, file_path: &str) {
        let mut image = Image::gen_image_color(self.width as i32, self.height as i32, Color::BLACK);
        for y in 0..self.height {
            for x in 0..self.width {
                let index = (y * self.width + x) * 4;
                let p = &self.color_buffer[index..index + 4];
                image.draw_pixel(x as i32, y as i32, Color::new(p[0], p[1], p[2], p[3]));
            }
        }
        image.export_image(file_path);
    }
}
