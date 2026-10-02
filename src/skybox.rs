
use crate::math::Vec3;
use crate::texture::Texture;

pub struct Skybox {
    // Orden: +X, -X, +Y, -Y, +Z, -Z
    faces: [Texture; 6],
    pub intensity: f32,
}

impl Skybox {
    pub fn load() -> Skybox {
        Skybox {
            faces: [
                Texture::from_png(include_bytes!("../assets/skybox/px.png")),
                Texture::from_png(include_bytes!("../assets/skybox/nx.png")),
                Texture::from_png(include_bytes!("../assets/skybox/py.png")),
                Texture::from_png(include_bytes!("../assets/skybox/ny.png")),
                Texture::from_png(include_bytes!("../assets/skybox/pz.png")),
                Texture::from_png(include_bytes!("../assets/skybox/nz.png")),
            ],
            intensity: 0.8,
        }
    }

    pub fn sample(&self, d: Vec3) -> Vec3 {
        let (ax, ay, az) = (d.x.abs(), d.y.abs(), d.z.abs());
        let (face, u, v) = if ax >= ay && ax >= az {
            if d.x > 0.0 { (0, -d.z / ax, -d.y / ax) } else { (1, d.z / ax, -d.y / ax) }
        } else if ay >= az {
            if d.y > 0.0 { (2, d.x / ay, d.z / ay) } else { (3, d.x / ay, -d.z / ay) }
        } else if d.z > 0.0 {
            (4, d.x / az, -d.y / az)
        } else {
            (5, -d.x / az, -d.y / az)
        };
        self.faces[face].sample_bilinear((u + 1.0) * 0.5, (v + 1.0) * 0.5) * self.intensity
    }
}

