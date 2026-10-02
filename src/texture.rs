use crate::math::Vec3;
use raylib::prelude::Image;

pub struct Texture {
    pub w: usize,
    pub h: usize,
    color: Vec<Vec3>,
    alpha: Vec<f32>,
}

fn srgb_to_linear(c: u8) -> f32 {
    (c as f32 / 255.0).powf(2.2)
}

impl Texture {
    pub fn from_png(bytes: &[u8]) -> Texture {
        let img = Image::load_image_from_mem(".png", bytes).expect("no se pudo leer PNG");
        let (w, h) = (img.width() as usize, img.height() as usize);
        let data = img.get_image_data();
        let mut color = Vec::with_capacity(w * h);
        let mut alpha = Vec::with_capacity(w * h);
        for c in data.iter() {
            color.push(Vec3::new(srgb_to_linear(c.r), srgb_to_linear(c.g), srgb_to_linear(c.b)));
            alpha.push(c.a as f32 / 255.0);
        }
        Texture { w, h, color, alpha }
    }

    pub fn sample(&self, u: f32, v: f32) -> (Vec3, f32) {
        let x = ((u.clamp(0.0, 0.9999)) * self.w as f32) as usize;
        let y = ((v.clamp(0.0, 0.9999)) * self.h as f32) as usize;
        let i = y * self.w + x;
        (self.color[i], self.alpha[i])
    }

    pub fn sample_bilinear(&self, u: f32, v: f32) -> Vec3 {
        let fx = (u.clamp(0.0, 1.0) * self.w as f32 - 0.5).max(0.0);
        let fy = (v.clamp(0.0, 1.0) * self.h as f32 - 0.5).max(0.0);
        let (x0, y0) = (fx as usize, fy as usize);
        let (x1, y1) = ((x0 + 1).min(self.w - 1), (y0 + 1).min(self.h - 1));
        let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
        let c = |x: usize, y: usize| self.color[y * self.w + x];
        let top = c(x0, y0).lerp(c(x1, y0), tx);
        let bot = c(x0, y1).lerp(c(x1, y1), tx);
        top.lerp(bot, ty)
    }

    pub fn average(&self) -> Vec3 {
        let mut s = Vec3::ZERO;
        for c in &self.color {
            s += *c;
        }
        s / self.color.len() as f32
    }
}

