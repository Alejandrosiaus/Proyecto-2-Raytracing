
use std::sync::Mutex;

use crate::camera::CameraBasis;
use crate::material::{build_materials, Material, TEXTURE_FILES};
use crate::math::{fresnel, hash2, reflect, refract, Vec3};
use crate::ray::Ray;
use crate::skybox::Skybox;
use crate::texture::Texture;
use crate::world::*;


pub const MAX_DEPTH: u32 = 5;

pub struct Scene {
    pub world: World,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub sky: Skybox,
    moon_dir: Vec3,
    moon_color: Vec3,
    fog_color: Vec3,
    fog_density: f32,
}

impl Scene {
    pub fn new(world: World) -> Scene {
        let textures: Vec<Texture> = TEXTURE_FILES.iter().map(|b| Texture::from_png(b)).collect();
        let materials = build_materials(&textures);
        Scene {
            world,
            materials,
            textures,
            sky: Skybox::load(),
            
            moon_dir: Vec3::new(-0.35, 0.42, -0.84).normalize(),
            moon_color: Vec3::new(0.80, 0.85, 1.0) * 1.35,
            fog_color: Vec3::new(0.55, 0.56, 0.59),
            fog_density: 0.0012,
        }
    }

    
    pub fn trace(&self, ray: &Ray, depth: u32, medium: Block) -> Vec3 {
        let Some(hit) = self.world.trace(ray, medium, 500.0) else {
            return self.sky.sample(ray.dir) + self.halo(ray, f32::INFINITY);
        };

        let mut color = if hit.exit {
            
            let m = &self.materials[BLOCKS[medium as usize].material];
            let nf = -hit.normal; 
            if depth >= MAX_DEPTH {
                self.sky.sample(ray.dir) * 0.5
            } else {
                match refract(ray.dir, nf, m.ior) {
                    Some(d2) => self.trace(&Ray::new(hit.point + hit.normal * 1e-3, d2), depth + 1, AIR),
                    
                    None => self.trace(&Ray::new(hit.point - hit.normal * 1e-3, reflect(ray.dir, nf)), depth + 1, medium),
                }
            }
        } else {
            self.shade(ray, &hit, depth, medium)
        };

        if medium != AIR {
            
            let m = &self.materials[BLOCKS[medium as usize].material];
            let absorb = (Vec3::ONE - m.tint * 0.7) * (-m.absorption * hit.t);
            color = color.mul_v(absorb.exp());
        } else {
            
            let f = 1.0 - (-hit.t * self.fog_density).exp();
            color = color.lerp(self.fog_color, f) + self.halo(ray, hit.t);
        }
        color
    }

    
    
    fn halo(&self, ray: &Ray, t_max: f32) -> Vec3 {
        let mut glow = Vec3::ZERO;
        for light in &self.world.lights {
            let t = (light.pos - ray.origin).dot(ray.dir);
            if t < 0.0 || t > t_max + 0.5 {
                continue;
            }
            let d2 = (ray.at(t) - light.pos).dot(ray.at(t) - light.pos);
            if d2 > 9.0 {
                continue;
            }
            glow += light.color * (light.intensity * (0.5 * (-d2 * 14.0).exp() + 0.012 / (d2 + 0.25)));
        }
        glow
    }

    fn shade(&self, ray: &Ray, hit: &Hit, depth: u32, medium: Block) -> Vec3 {
        let def = BLOCKS[hit.block as usize];
        let m = &self.materials[def.material];
        let n = hit.normal;

        
        let (u, v) = face_uv(hit.point, n);
        let tex = if n.y > 0.5 { m.tex_top } else if n.y < -0.5 { m.tex_bottom } else { m.tex_side };
        let (base, alpha) = self.textures[tex].sample(u, v);

        if m.emission > 0.0 {
            return base * m.emission;
        }

        let p = hit.point + n * 2e-3;
        let view = -ray.dir;

        
        let sky_up = Vec3::new(0.30, 0.32, 0.38);
        let sky_down = Vec3::new(0.16, 0.16, 0.18);
        let hemi = sky_down.lerp(sky_up, n.y * 0.5 + 0.5);
        let ao = self.ambient_occlusion(hit.point, n);
        let vis_dir = (n + Vec3::new(0.0, 1.2, 0.0)).normalize();
        let sky_vis = 0.25 + 0.75 * self.world.transmittance(&Ray::new(p, vis_dir), 30.0, &self.materials);
        let ambient = hemi * (ao * sky_vis);

        let mut diffuse = Vec3::ZERO;
        let mut spec = Vec3::ZERO;

        
        let l = self.moon_dir;
        let ndl = n.dot(l);
        if ndl > 0.0 {
            let shadow = self.world.transmittance(&Ray::new(p, l), 200.0, &self.materials);
            if shadow > 0.0 {
                diffuse += self.moon_color * (ndl * shadow);
                let r = reflect(-l, n);
                spec += self.moon_color * (r.dot(view).max(0.0).powf(m.specular) * shadow);
            }
        }

        
        for light in &self.world.lights {
            let to_l = light.pos - p;
            let d = to_l.length();
            if d > 14.0 {
                continue;
            }
            let l = to_l / d;
            let ndl = n.dot(l);
            if ndl <= 0.0 {
                continue;
            }
            let atten = light.intensity / (1.0 + 0.5 * d * d);
            let shadow = self.world.transmittance(&Ray::new(p, l), d - 0.45, &self.materials);
            if shadow <= 0.0 {
                continue;
            }
            let c = light.color * (atten * shadow);
            diffuse += c * ndl;
            let r = reflect(-l, n);
            spec += c * r.dot(view).max(0.0).powf(m.specular);
        }

        let local = base.mul_v(ambient + diffuse * m.albedo[0]) + spec * m.albedo[1];

        
        let transparency = m.transparency * (1.0 - alpha);
        let (n1, n2) = if medium == AIR { (1.0, m.ior) } else { (m.ior, 1.0) };
        let kr = if transparency > 0.0 { fresnel(ray.dir, n, n1, n2) } else { 0.0 };
        let refl_w = (m.reflectivity + transparency * kr).min(1.0);
        let refr_w = transparency * (1.0 - kr);
        let local_w = (1.0 - refl_w - refr_w).max(0.0);

        let mut color = local * local_w;
        if depth < MAX_DEPTH {
            if refl_w > 0.01 {
                let r = reflect(ray.dir, n);
                color += self.trace(&Ray::new(p, r), depth + 1, medium) * refl_w;
            }
            if refr_w > 0.01 {
                if let Some(d2) = refract(ray.dir, n, n1 / n2) {
                    let inside = hit.point - n * 2e-3;
                    let tint = Vec3::ONE.lerp(m.tint, 0.35);
                    color += self.trace(&Ray::new(inside, d2), depth + 1, hit.block).mul_v(tint) * refr_w;
                }
            }
        } else {
            color += self.sky.sample(ray.dir) * (refl_w + refr_w) * 0.5;
        }
        color
    }

    
    fn ambient_occlusion(&self, point: Vec3, n: Vec3) -> f32 {
        let na = if n.x.abs() > 0.5 { 0 } else if n.y.abs() > 0.5 { 1 } else { 2 };
        let (ua, va) = match na {
            0 => (1, 2),
            1 => (0, 2),
            _ => (0, 1),
        };
        let front = point + n * 0.01;
        let c = [front.x.floor() as i32, front.y.floor() as i32, front.z.floor() as i32];
        let lu = front.get(ua) - front.get(ua).floor();
        let lv = front.get(va) - front.get(va).floor();
        let occ = |du: i32, dv: i32| -> f32 {
            let mut q = c;
            q[ua] += du;
            q[va] += dv;
            if self.world.occludes(q[0], q[1], q[2]) { 1.0 } else { 0.0 }
        };
        let vertex = |su: i32, sv: i32| -> f32 {
            let (s1, s2, cr) = (occ(su, 0), occ(0, sv), occ(su, sv));
            if s1 > 0.0 && s2 > 0.0 { 0.0 } else { (3.0 - s1 - s2 - cr) / 3.0 }
        };
        let a00 = vertex(-1, -1);
        let a10 = vertex(1, -1);
        let a01 = vertex(-1, 1);
        let a11 = vertex(1, 1);
        let top = a00 + (a10 - a00) * lu;
        let bot = a01 + (a11 - a01) * lu;
        let ao = top + (bot - top) * lv;
        0.35 + 0.65 * ao
    }
}


fn face_uv(p: Vec3, n: Vec3) -> (f32, f32) {
    let q = p - n * 1e-3; 
    let l = Vec3::new(q.x - q.x.floor(), q.y - q.y.floor(), q.z - q.z.floor());
    if n.x > 0.5 {
        (1.0 - l.z, 1.0 - l.y)
    } else if n.x < -0.5 {
        (l.z, 1.0 - l.y)
    } else if n.y > 0.5 {
        (l.x, l.z)
    } else if n.y < -0.5 {
        (l.x, 1.0 - l.z)
    } else if n.z > 0.5 {
        (l.x, 1.0 - l.y)
    } else {
        (1.0 - l.x, 1.0 - l.y)
    }
}


pub struct Accum {
    pub w: usize,
    pub h: usize,
    pub data: Vec<Vec3>,
    pub samples: u32,
    pub rgba: Vec<u8>,
}

impl Accum {
    pub fn new(w: usize, h: usize) -> Self {
        Self { w, h, data: vec![Vec3::ZERO; w * h], samples: 0, rgba: vec![255; w * h * 4] }
    }
    pub fn reset(&mut self) {
        self.samples = 0;
        self.data.iter_mut().for_each(|c| *c = Vec3::ZERO);
    }
}

fn threads() -> usize {
    std::thread::available_parallelism().map_or(4, |n| n.get())
}



pub fn render_pass(scene: &Scene, cam: &CameraBasis, acc: &mut Accum) {
    let (w, h) = (acc.w, acc.h);
    let sample = acc.samples;
    let jitter = sample > 0;
    const ROWS: usize = 2;
    let chunks = Mutex::new(acc.data.chunks_mut(w * ROWS).enumerate());
    std::thread::scope(|s| {
        for _ in 0..threads() {
            s.spawn(|| loop {
                let next = chunks.lock().unwrap().next();
                let Some((ci, chunk)) = next else { break };
                for (k, px) in chunk.iter_mut().enumerate() {
                    let x = k % w;
                    let y = ci * ROWS + k / w;
                    let (jx, jy) = if jitter {
                        (hash2(x as i32, y as i32, sample * 2 + 1), hash2(x as i32, y as i32, sample * 2 + 2))
                    } else {
                        (0.5, 0.5)
                    };
                    let ray = cam.ray_for(x as f32 + jx, y as f32 + jy, w, h);
                    *px += scene.trace(&ray, 0, AIR);
                }
            });
        }
    });
    acc.samples += 1;
    resolve(acc);
}


fn resolve(acc: &mut Accum) {
    let inv = 1.0 / acc.samples.max(1) as f32;
    let aces = |x: f32| ((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14)).clamp(0.0, 1.0);
    let n = threads();
    let per = (acc.data.len() + n - 1) / n;
    std::thread::scope(|s| {
        for (src, dst) in acc.data.chunks(per).zip(acc.rgba.chunks_mut(per * 4)) {
            s.spawn(move || {
                for (c, o) in src.iter().zip(dst.chunks_mut(4)) {
                    let c = *c * inv * 1.1; 
                    o[0] = (aces(c.x).powf(1.0 / 2.2) * 255.0) as u8;
                    o[1] = (aces(c.y).powf(1.0 / 2.2) * 255.0) as u8;
                    o[2] = (aces(c.z).powf(1.0 / 2.2) * 255.0) as u8;
                    o[3] = 255;
                }
            });
        }
    });
}
