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

// Índices de texturas (orden de TEXTURE_FILES)
pub const T_ROCK: usize = 0;
pub const T_GRASS_TOP: usize = 1;
pub const T_GRASS_SIDE: usize = 2;
pub const T_DIRT: usize = 3;
pub const T_FLOWERS: usize = 4;
pub const T_COBBLE: usize = 5;
pub const T_PLANKS: usize = 6;
pub const T_BRICKS: usize = 7;
pub const T_ROOF: usize = 8;
pub const T_GLASS: usize = 9;
pub const T_WATER: usize = 10;
pub const T_LOG_SIDE: usize = 11;
pub const T_LOG_TOP: usize = 12;
pub const T_LEAVES: usize = 13;
pub const T_LANTERN: usize = 14;
pub const T_MARBLE: usize = 15;

pub const TEXTURE_FILES: [&[u8]; 16] = [
    include_bytes!("../assets/textures/rock.png"),
    include_bytes!("../assets/textures/grass_top.png"),
    include_bytes!("../assets/textures/grass_side.png"),
    include_bytes!("../assets/textures/dirt.png"),
    include_bytes!("../assets/textures/flowers_top.png"),
    include_bytes!("../assets/textures/cobble.png"),
    include_bytes!("../assets/textures/planks.png"),
    include_bytes!("../assets/textures/bricks.png"),
    include_bytes!("../assets/textures/roof.png"),
    include_bytes!("../assets/textures/glass.png"),
    include_bytes!("../assets/textures/water.png"),
    include_bytes!("../assets/textures/log_side.png"),
    include_bytes!("../assets/textures/log_top.png"),
    include_bytes!("../assets/textures/leaves.png"),
    include_bytes!("../assets/textures/lantern.png"),
    include_bytes!("../assets/textures/marble.png"),
];

// Índices de materiales
pub const M_ROCK: usize = 0;
pub const M_GRASS: usize = 1;
pub const M_FLOWERS: usize = 2;
pub const M_COBBLE: usize = 3;
pub const M_PLANKS: usize = 4;
pub const M_BRICKS: usize = 5;
pub const M_ROOF: usize = 6;
pub const M_GLASS: usize = 7;
pub const M_WATER: usize = 8;
pub const M_LOG: usize = 9;
pub const M_LEAVES: usize = 10;
pub const M_LANTERN: usize = 11;
pub const M_MARBLE: usize = 12;
pub const M_DIRT: usize = 13;

#[allow(clippy::too_many_arguments)]
fn mat(
    name: &'static str,
    tex: (usize, usize, usize),
    albedo: [f32; 2],
    specular: f32,
    transparency: f32,
    reflectivity: f32,
    ior: f32,
    emission: f32,
    absorption: f32,
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
        emission,
        absorption,
        tint: Vec3::ONE,
    }
}

pub fn build_materials(textures: &[Texture]) -> Vec<Material> {
    let same = |t| (t, t, t);
    //                 nombre             texturas (arriba, lado, abajo)         albedo      spec   transp refl   ior   emis  absorc
    let mut m = vec![
        mat("Piedra oscura",   same(T_ROCK),                                [0.90, 0.10], 12.0,  0.0, 0.00, 1.0,  0.0, 0.0),
        mat("Musgo",           (T_GRASS_TOP, T_GRASS_SIDE, T_DIRT),         [0.95, 0.05],  6.0,  0.0, 0.00, 1.0,  0.0, 0.0),
        mat("Flores blancas",  (T_FLOWERS, T_GRASS_SIDE, T_DIRT),           [0.92, 0.25], 20.0,  0.0, 0.00, 1.0,  0.0, 0.0),
        mat("Adoquin",         same(T_COBBLE),                              [0.85, 0.20], 18.0,  0.0, 0.03, 1.0,  0.0, 0.0),
        mat("Madera",          same(T_PLANKS),                              [0.88, 0.20], 25.0,  0.0, 0.00, 1.0,  0.0, 0.0),
        mat("Ladrillo gotico", same(T_BRICKS),                              [0.90, 0.10], 10.0,  0.0, 0.00, 1.0,  0.0, 0.0),
        mat("Tejas de pizarra",same(T_ROOF),                                [0.70, 0.45], 60.0,  0.0, 0.10, 1.0,  0.0, 0.0),
        mat("Vidrio",          same(T_GLASS),                               [0.20, 0.80],125.0,  0.90, 0.06, 1.5,  0.0, 0.05),
        mat("Agua",            same(T_WATER),                               [0.30, 0.70],150.0,  0.80, 0.10, 1.33, 0.0, 0.45),
        mat("Corteza",         (T_LOG_TOP, T_LOG_SIDE, T_LOG_TOP),          [0.95, 0.05],  5.0,  0.0, 0.00, 1.0,  0.0, 0.0),
        mat("Hojas",           same(T_LEAVES),                              [0.90, 0.10],  8.0,  0.0, 0.00, 1.0,  0.0, 0.0),
        mat("Farol",           same(T_LANTERN),                             [1.00, 0.00],  1.0,  0.0, 0.00, 1.0,  3.0, 0.0),
        mat("Marmol pulido",   same(T_MARBLE),                              [0.60, 0.60], 90.0,  0.0, 0.35, 1.0,  0.0, 0.0),
        mat("Tierra",          same(T_DIRT),                                [0.95, 0.05],  4.0,  0.0, 0.00, 1.0,  0.0, 0.0),
    ];
    for material in m.iter_mut() {
        let avg = textures[material.tex_top].average();
        material.tint = avg / avg.max_elem().max(1e-3);
    }
    m
}

