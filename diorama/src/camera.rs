use crate::math::Vec3;
use crate::ray::Ray;

#[derive(Clone, Copy)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub yaw: f32,      // rotación horizontal (radianes)
    pub pitch: f32,    // rotación vertical (radianes)
    pub distance: f32, // zoom
    pub fov: f32,      // campo de visión vertical (grados)
}

pub struct CameraBasis {
    pub eye: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    half_h: f32,
}

pub const MIN_DIST: f32 = 18.0;
pub const MAX_DIST: f32 = 160.0;

impl OrbitCamera {
    pub fn new(target: Vec3, distance: f32) -> Self {
        Self { target, yaw: 0.65, pitch: 0.22, distance, fov: 45.0 }
    }

    pub fn rotate(&mut self, d_yaw: f32, d_pitch: f32) {
        self.yaw += d_yaw;
        // Limitar el pitch para no voltear la cámara (se permite ver la isla desde abajo)
        self.pitch = (self.pitch + d_pitch).clamp(-1.2, 1.45);
    }

    pub fn zoom(&mut self, amount: f32) {
        self.distance = (self.distance * (1.0 - amount)).clamp(MIN_DIST, MAX_DIST);
    }

    pub fn eye(&self) -> Vec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        self.target + Vec3::new(x, y, z)
    }

    pub fn basis(&self) -> CameraBasis {
        let eye = self.eye();
        let forward = (self.target - eye).normalize();
        let right = forward.cross(Vec3::new(0.0, 1.0, 0.0)).normalize();
        let up = right.cross(forward);
        let half_h = (self.fov.to_radians() * 0.5).tan();
        CameraBasis { eye, forward, right, up, half_h }
    }
}

impl CameraBasis {
    pub fn ray_for(&self, px: f32, py: f32, width: usize, height: usize) -> Ray {
        let aspect = width as f32 / height as f32;
        let sx = (2.0 * px / width as f32 - 1.0) * aspect * self.half_h;
        let sy = (1.0 - 2.0 * py / height as f32) * self.half_h;
        let dir = self.forward + self.right * sx + self.up * sy;
        Ray::new(self.eye, dir)
    }
}

