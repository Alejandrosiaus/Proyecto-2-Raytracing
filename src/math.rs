
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3::splat(0.0);
    pub const ONE: Vec3 = Vec3::splat(1.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }
    pub fn dot(self, o: Vec3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    pub fn cross(self, o: Vec3) -> Vec3 {
        Vec3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn normalize(self) -> Vec3 {
        let l = self.length();
        if l > 0.0 { self / l } else { self }
    }
    
    pub fn mul_v(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x * o.x, self.y * o.y, self.z * o.z)
    }
    pub fn lerp(self, o: Vec3, t: f32) -> Vec3 {
        self * (1.0 - t) + o * t
    }
    pub fn max_elem(self) -> f32 {
        self.x.max(self.y).max(self.z)
    }
    pub fn exp(self) -> Vec3 {
        Vec3::new(self.x.exp(), self.y.exp(), self.z.exp())
    }
    pub fn get(self, axis: usize) -> f32 {
        match axis {
            0 => self.x,
            1 => self.y,
            _ => self.z,
        }
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 { Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z) }
}
impl AddAssign for Vec3 {
    fn add_assign(&mut self, o: Vec3) { self.x += o.x; self.y += o.y; self.z += o.z; }
}
impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 { Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z) }
}
impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, s: f32) -> Vec3 { Vec3::new(self.x * s, self.y * s, self.z * s) }
}
impl Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, s: f32) -> Vec3 { Vec3::new(self.x / s, self.y / s, self.z / s) }
}
impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 { Vec3::new(-self.x, -self.y, -self.z) }
}


pub fn reflect(i: Vec3, n: Vec3) -> Vec3 {
    i - n * (2.0 * i.dot(n))
}



pub fn refract(i: Vec3, n: Vec3, eta: f32) -> Option<Vec3> {
    let cos_i = -i.dot(n).clamp(-1.0, 1.0);
    let k = 1.0 - eta * eta * (1.0 - cos_i * cos_i);
    if k < 0.0 {
        None
    } else {
        Some((i * eta + n * (eta * cos_i - k.sqrt())).normalize())
    }
}


pub fn fresnel(i: Vec3, n: Vec3, n1: f32, n2: f32) -> f32 {
    let mut cos_i = -i.dot(n).clamp(-1.0, 1.0);
    if n1 > n2 {
        let eta = n1 / n2;
        let sin_t2 = eta * eta * (1.0 - cos_i * cos_i);
        if sin_t2 > 1.0 {
            return 1.0; 
        }
        cos_i = (1.0 - sin_t2).sqrt();
    }
    let r0 = ((n1 - n2) / (n1 + n2)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cos_i).powi(5)
}


pub fn hash2(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ seed.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    (h & 0x00FF_FFFF) as f32 / 16_777_216.0
}


pub fn value_noise(x: f32, y: f32, seed: u32) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let (ux, uy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let a = hash2(ix, iy, seed);
    let b = hash2(ix + 1, iy, seed);
    let c = hash2(ix, iy + 1, seed);
    let d = hash2(ix + 1, iy + 1, seed);
    let top = a + (b - a) * ux;
    let bot = c + (d - c) * ux;
    top + (bot - top) * uy
}


pub fn fbm(x: f32, y: f32, seed: u32) -> f32 {
    let (mut sum, mut amp, mut freq, mut norm) = (0.0, 0.5, 1.0, 0.0);
    for o in 0..4 {
        sum += amp * value_noise(x * freq, y * freq, seed + o);
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm
}
