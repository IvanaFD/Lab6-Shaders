use raylib::prelude::*;

// Cámara que orbita alrededor de un punto
pub struct OrbitCamera {
    pub target: Vector3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl OrbitCamera {
    pub fn new() -> Self {
        Self {
            target: Vector3::zero(),
            yaw: 0.0,
            pitch: 0.0,
            distance: 3.2,
        }
    }

    pub fn eye(&self) -> Vector3 {
        self.target
            + Vector3::new(
                self.pitch.cos() * self.yaw.sin(),
                self.pitch.sin(),
                self.pitch.cos() * self.yaw.cos(),
            ) * self.distance
    }

    pub fn view_matrix(&self) -> Matrix {
        Matrix::look_at(self.eye(), self.target, Vector3::new(0.0, 1.0, 0.0))
    }

    pub fn orbit(&mut self, d_yaw: f32, d_pitch: f32) {
        self.yaw += d_yaw;
        let limit = std::f32::consts::FRAC_PI_2 - 0.05;
        self.pitch = (self.pitch + d_pitch).clamp(-limit, limit);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance + delta).clamp(1.6, 8.0);
    }
}
