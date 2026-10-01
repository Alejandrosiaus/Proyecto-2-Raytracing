
use crate::material::*;
use crate::math::Vec3;
use crate::ray::Ray;

pub type Block = u8;

pub const AIR: Block = 0;
pub const ROCK: Block = 1;
pub const GRASS: Block = 2;
pub const FLOWERS: Block = 3;
pub const COBBLE: Block = 4;
pub const COBBLE_SLAB: Block = 5;
pub const PLANKS: Block = 6;
pub const BRICKS: Block = 7;
pub const ROOF: Block = 8;
pub const GLASS: Block = 9;
#[allow(dead_code)]
pub const WATER: Block = 10;
pub const LOG: Block = 11;
#[allow(dead_code)]
pub const LEAVES: Block = 12;
pub const LANTERN: Block = 13;
pub const MARBLE: Block = 14;
pub const GRAVE: Block = 15;
pub const DIRT: Block = 16;
pub const POST: Block = 17;
#[allow(dead_code)]
pub const FENCE: Block = 18;
pub const BLOCK_COUNT: usize = 19;


#[derive(Clone, Copy)]
pub enum Shape {
    Full,
    Box([f32; 3], [f32; 3]),
}

#[derive(Clone, Copy)]
pub struct BlockDef {
    pub material: usize,
    pub shape: Shape,
}

pub const BLOCKS: [BlockDef; BLOCK_COUNT] = [
    BlockDef { material: M_PIEDRA, shape: Shape::Full },     
    BlockDef { material: M_PIEDRA, shape: Shape::Full },     
    BlockDef { material: M_PASTO, shape: Shape::Full },      
    BlockDef { material: M_FLORES, shape: Shape::Full },    
    BlockDef { material: M_PIEDRA, shape: Shape::Full },     
    BlockDef { material: M_PIEDRA, shape: Shape::Box([0.0, 0.0, 0.0], [1.0, 0.5, 1.0]) }, 
    BlockDef { material: M_MADERA, shape: Shape::Full },     
    BlockDef { material: M_PIEDRA, shape: Shape::Full },     
    BlockDef { material: M_MADERA, shape: Shape::Full },     
    BlockDef { material: M_VIDRIO, shape: Shape::Full },      
    BlockDef { material: M_VIDRIO, shape: Shape::Full },     
    BlockDef { material: M_MADERA, shape: Shape::Full },        
    BlockDef { material: M_PASTO, shape: Shape::Full },      
    BlockDef { material: M_VIDRIO, shape: Shape::Box([0.28, 0.05, 0.28], [0.72, 0.7, 0.72]) }, 
    BlockDef { material: M_PIEDRA, shape: Shape::Full },     
    BlockDef { material: M_PIEDRA, shape: Shape::Box([0.18, 0.0, 0.4], [0.82, 0.9, 0.6]) },     
    BlockDef { material: M_PIEDRA, shape: Shape::Full },       
    BlockDef { material: M_MADERA, shape: Shape::Box([0.42, 0.0, 0.42], [0.58, 1.0, 0.58]) },   
    BlockDef { material: M_MADERA, shape: Shape::Box([0.0, 0.0, 0.43], [1.0, 1.0, 0.57]) },   
];

#[derive(Clone, Copy)]
pub struct PointLight {
    pub pos: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}


#[derive(Clone, Copy)]
pub struct Hit {
    pub t: f32,
    pub point: Vec3,
    
    
    pub normal: Vec3,
    pub block: Block,
    
    pub exit: bool,
}

pub struct World {
    pub nx: i32,
    pub ny: i32,
    pub nz: i32,
    cells: Vec<Block>,
    pub lights: Vec<PointLight>,
}

impl World {
    pub fn new(nx: i32, ny: i32, nz: i32) -> Self {
        Self { nx, ny, nz, cells: vec![AIR; (nx * ny * nz) as usize], lights: Vec::new() }
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32, z: i32) -> bool {
        x >= 0 && y >= 0 && z >= 0 && x < self.nx && y < self.ny && z < self.nz
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32, z: i32) -> Block {
        if self.in_bounds(x, y, z) {
            self.cells[((y * self.nz + z) * self.nx + x) as usize]
        } else {
            AIR
        }
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, b: Block) {
        if self.in_bounds(x, y, z) {
            let i = ((y * self.nz + z) * self.nx + x) as usize;
            self.cells[i] = b;
        }
    }

    pub fn count_blocks(&self) -> usize {
        self.cells.iter().filter(|&&b| b != AIR).count()
    }

    
    
    fn dda<T>(
        &self,
        ray: &Ray,
        t_limit: f32,
        mut visit: impl FnMut([i32; 3], f32, f32, usize, [i32; 3]) -> Option<T>,
    ) -> Option<T> {
        let o = ray.origin;
        let d = ray.dir;
        let size = Vec3::new(self.nx as f32, self.ny as f32, self.nz as f32);

        
        let mut t0 = 0.0f32;
        let mut t1 = t_limit;
        let mut entry_axis = 3;
        for a in 0..3 {
            let (oa, da, sa) = (o.get(a), d.get(a), size.get(a));
            if da.abs() < 1e-9 {
                if oa < 0.0 || oa > sa {
                    return None;
                }
            } else {
                let inv = 1.0 / da;
                let (mut ta, mut tb) = (-oa * inv, (sa - oa) * inv);
                if ta > tb {
                    std::mem::swap(&mut ta, &mut tb);
                }
                if ta > t0 {
                    t0 = ta;
                    entry_axis = a;
                }
                t1 = t1.min(tb);
            }
        }
        if t0 > t1 {
            return None;
        }

        let p = ray.at(t0 + 1e-4);
        let mut cell = [
            (p.x.floor() as i32).clamp(0, self.nx - 1),
            (p.y.floor() as i32).clamp(0, self.ny - 1),
            (p.z.floor() as i32).clamp(0, self.nz - 1),
        ];
        let mut step = [0i32; 3];
        let mut t_max = [f32::INFINITY; 3];
        let mut t_delta = [f32::INFINITY; 3];
        for a in 0..3 {
            let da = d.get(a);
            if da > 0.0 {
                step[a] = 1;
                t_max[a] = ((cell[a] + 1) as f32 - o.get(a)) / da;
                t_delta[a] = 1.0 / da;
            } else if da < 0.0 {
                step[a] = -1;
                t_max[a] = (cell[a] as f32 - o.get(a)) / da;
                t_delta[a] = -1.0 / da;
            }
        }

        let mut t_enter = t0;
        let mut axis = entry_axis;
        loop {
            let t_exit = t_max[0].min(t_max[1]).min(t_max[2]);
            if let Some(r) = visit(cell, t_enter, t_exit, axis, step) {
                return Some(r);
            }
            
            axis = if t_max[0] < t_max[1] {
                if t_max[0] < t_max[2] { 0 } else { 2 }
            } else if t_max[1] < t_max[2] { 1 } else { 2 };
            t_enter = t_max[axis];
            if t_enter > t1 {
                return None;
            }
            cell[axis] += step[axis];
            t_max[axis] += t_delta[axis];
            if cell[axis] < 0
                || cell[axis] >= [self.nx, self.ny, self.nz][axis]
            {
                return None;
            }
        }
    }

    
    fn box_hit(ray: &Ray, cell: [i32; 3], shape: Shape, t_enter: f32, axis: usize, step: [i32; 3]) -> Option<(f32, Vec3)> {
        match shape {
            Shape::Full => {
                if axis == 3 {
                    return None; 
                }
                let mut n = [0.0f32; 3];
                n[axis] = -step[axis] as f32;
                Some((t_enter, Vec3::new(n[0], n[1], n[2])))
            }
            Shape::Box(bmin, bmax) => {
                let mut tmin = f32::NEG_INFINITY;
                let mut tmax = f32::INFINITY;
                let mut n_axis = 0;
                for a in 0..3 {
                    let o = ray.origin.get(a);
                    let d = ray.dir.get(a);
                    let lo = cell[a] as f32 + bmin[a];
                    let hi = cell[a] as f32 + bmax[a];
                    if d.abs() < 1e-9 {
                        if o < lo || o > hi {
                            return None;
                        }
                        continue;
                    }
                    let (mut ta, mut tb) = ((lo - o) / d, (hi - o) / d);
                    if ta > tb {
                        std::mem::swap(&mut ta, &mut tb);
                    }
                    if ta > tmin {
                        tmin = ta;
                        n_axis = a;
                    }
                    tmax = tmax.min(tb);
                }
                if tmin > tmax || tmin < 1e-4 {
                    return None;
                }
                let mut n = [0.0f32; 3];
                n[n_axis] = if ray.dir.get(n_axis) > 0.0 { -1.0 } else { 1.0 };
                Some((tmin, Vec3::new(n[0], n[1], n[2])))
            }
        }
    }

    
    
    pub fn trace(&self, ray: &Ray, medium: Block, t_limit: f32) -> Option<Hit> {
        self.dda(ray, t_limit, |cell, t_enter, _t_exit, axis, step| {
            let b = self.get(cell[0], cell[1], cell[2]);
            if medium != AIR {
                if b == medium {
                    return None; 
                }
                if axis == 3 {
                    return None;
                }
                let full = matches!(BLOCKS[b as usize].shape, Shape::Full);
                let mut n = [0.0f32; 3];
                n[axis] = step[axis] as f32;
                let normal = Vec3::new(n[0], n[1], n[2]);
                return if b == AIR || !full {
                    
                    Some(Hit { t: t_enter, point: ray.at(t_enter), normal, block: medium, exit: true })
                } else {
                    
                    Some(Hit { t: t_enter, point: ray.at(t_enter), normal: -normal, block: b, exit: false })
                };
            }
            if b == AIR {
                return None;
            }
            let shape = BLOCKS[b as usize].shape;
            Self::box_hit(ray, cell, shape, t_enter, axis, step)
                .map(|(t, normal)| Hit { t, point: ray.at(t), normal, block: b, exit: false })
        })
    }

    
    
    pub fn transmittance(&self, ray: &Ray, t_limit: f32, materials: &[Material]) -> f32 {
        let mut light = 1.0f32;
        let mut last_transparent: Block = AIR;
        let blocked = self.dda(ray, t_limit, |cell, t_enter, _t_exit, axis, step| {
            if t_enter > t_limit {
                return Some(false);
            }
            let b = self.get(cell[0], cell[1], cell[2]);
            if b == AIR || b == LANTERN {
                last_transparent = AIR;
                return None;
            }
            let def = BLOCKS[b as usize];
            let m = &materials[def.material];
            if m.transparency > 0.0 {
                if b != last_transparent {
                    light *= m.transparency * 0.85;
                }
                last_transparent = b;
                return if light < 0.02 { Some(true) } else { None };
            }
            last_transparent = AIR;
            match Self::box_hit(ray, cell, def.shape, t_enter, axis, step) {
                Some((t, _)) if t < t_limit => Some(true),
                _ => None,
            }
        });
        if blocked == Some(true) { 0.0 } else { light }
    }

    
    #[inline]
    pub fn occludes(&self, x: i32, y: i32, z: i32) -> bool {
        let b = self.get(x, y, z);
        b != AIR && b != GLASS && b != WATER && b != LANTERN && b != POST && b != GRAVE && b != FENCE
    }
}
