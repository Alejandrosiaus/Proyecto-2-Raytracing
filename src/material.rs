use crate::math::Vec3;
use crate::texture::Texture;

pub struct Material {
    pub name: &'static str,
    pub tex_top: usize,
    pub tex_side: usize,
    pub tex_bottom: usize,
    pub albedo: [f32; 2],
    pub specular: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub ior: f32,
    pub emission: f32,
    pub absorption: f32,
    pub tint: Vec3,
}

pub const T_ROCK: usize = 0;
pub const T_LOG_SIDE: usize = 1;
pub const T_LOG_TOP: usize = 2;
pub const T_GRASS_TOP: usize = 3;
pub const T_GRASS_SIDE: usize = 4;
pub const T_DIRT: usize = 5;
pub const T_FLOWERS: usize = 6;
pub const T_GLASS: usize = 7;

pub const TEXTURE_FILES: [&[u8]; 8] = [
    include_bytes!("../assets/textures/rock.png"),
    include_bytes!("../assets/textures/log_side.png"),
    include_bytes!("../assets/textures/log_top.png"),
    include_bytes!("../assets/textures/grass_top.png"),
    include_bytes!("../assets/textures/grass_side.png"),
    include_bytes!("../assets/textures/dirt.png"),
    include_bytes!("../assets/textures/flowers_top.png"),
    include_bytes!("../assets/textures/glass.png"),
];

pub const M_PIEDRA: usize = 0;
pub const M_MADERA: usize = 1;
pub const M_PASTO: usize = 2;
pub const M_FLORES: usize = 3;
pub const M_VIDRIO: usize = 4;

#[allow(clippy::too_many_arguments)]
fn mat(
    name: &'static str,
    tex: (usize, usize, usize),
    albedo: [f32; 2],
    specular: f32,
    transparency: f32,
    reflectivity: f32,
    ior: f32,
) -> Material {
    Material {
        name,
        tex_top: tex.0,
        tex_side: tex.1,
        tex_bottom: tex.2,
        albedo,
        specular,
        transparency,
        reflectivity,
        ior,
        emission: 0.0,
        absorption: 0.05,
        tint: Vec3::ONE,
    }
}

pub fn build_materials(textures: &[Texture]) -> Vec<Material> {
    let same = |t| (t, t, t);
    
    let mut m = vec![
        mat("Piedra",         same(T_ROCK),                           [0.90, 0.10],  12.0,  0.00,  0.00,  1.0),
        mat("Madera",         (T_LOG_TOP, T_LOG_SIDE, T_LOG_TOP),     [0.92, 0.08],   8.0,  0.00,  0.00,  1.0),
        mat("Pasto",          (T_GRASS_TOP, T_GRASS_SIDE, T_DIRT),    [0.95, 0.05],   6.0,  0.00,  0.00,  1.0),
        mat("Flores blancas", (T_FLOWERS, T_GRASS_SIDE, T_DIRT),      [0.92, 0.25],  20.0,  0.00,  0.00,  1.0),
        mat("Vidrio",         same(T_GLASS),                          [0.20, 0.80], 125.0,  0.90,  0.20,  1.5),
    ];
    for material in m.iter_mut() {
        let avg = textures[material.tex_top].average();
        material.tint = avg / avg.max_elem().max(1e-3);
    }
    m
}
