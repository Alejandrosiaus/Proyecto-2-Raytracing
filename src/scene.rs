
use crate::math::{fbm, hash2, Vec3};
use crate::world::*;

pub const NX: i32 = 64;
pub const NY: i32 = 58;
pub const NZ: i32 = 86;

const OUTLINE_PX: [(f32, f32); 26] = [
    (480.0, 200.0), (620.0, 140.0), (780.0, 140.0), (900.0, 190.0), (990.0, 300.0),
    (1090.0, 280.0), (1250.0, 350.0), (1360.0, 470.0), (1430.0, 650.0), (1470.0, 900.0),
    (1480.0, 1200.0), (1440.0, 1450.0), (1350.0, 1600.0), (1200.0, 1700.0), (1000.0, 1790.0),
    (800.0, 1830.0), (600.0, 1800.0), (400.0, 1740.0), (220.0, 1640.0), (90.0, 1500.0),
    (40.0, 1300.0), (60.0, 1100.0), (140.0, 900.0), (210.0, 700.0), (260.0, 520.0),
    (350.0, 330.0),
];
const PX_TO_GRID: f32 = 64.0 / 1500.0;

const BASE: i32 = 32; // altura del suelo principal (celda superior)
const PLAZA: i32 = 30; // altura de la plaza baja

struct Builder {
    w: World,
    top: Vec<i32>, // celda superior sólida por columna (-1 = vacío)
    outline: Vec<(f32, f32)>,
}

impl Builder {
    fn top(&self, x: i32, z: i32) -> i32 {
        if x < 0 || z < 0 || x >= NX || z >= NZ {
            return -1;
        }
        self.top[(z * NX + x) as usize]
    }
    fn set_top(&mut self, x: i32, z: i32, y: i32) {
        if x >= 0 && z >= 0 && x < NX && z < NZ {
            self.top[(z * NX + x) as usize] = y;
        }
    }
    fn set(&mut self, x: i32, y: i32, z: i32, b: Block) {
        self.w.set(x, y, z, b);
    }
    fn surface(&mut self, x: i32, z: i32, b: Block) {
        let t = self.top(x, z);
        if t >= 0 {
            self.set(x, t, z, b);
        }
    }
    fn place_above(&mut self, x: i32, z: i32, b: Block) -> i32 {
        let t = self.top(x, z);
        if t < 0 {
            return -1;
        }
        self.set(x, t + 1, z, b);
        self.set_top(x, z, t + 1);
        t + 1
    }
    fn inside(&self, x: f32, z: f32) -> bool {
        // Punto dentro de polígono (ray casting 2D)
        let mut inside = false;
        let n = self.outline.len();
        let mut j = n - 1;
        for i in 0..n {
            let (xi, zi) = self.outline[i];
            let (xj, zj) = self.outline[j];
            if (zi > z) != (zj > z) && x < (xj - xi) * (z - zi) / (zj - zi) + xi {
                inside = !inside;
            }
            j = i;
        }
        inside
    }
    fn edge_dist(&self, x: f32, z: f32) -> f32 {
        let n = self.outline.len();
        let mut best = f32::MAX;
        for i in 0..n {
            let (ax, az) = self.outline[i];
            let (bx, bz) = self.outline[(i + 1) % n];
            let (dx, dz) = (bx - ax, bz - az);
            let t = (((x - ax) * dx + (z - az) * dz) / (dx * dx + dz * dz)).clamp(0.0, 1.0);
            let (px, pz) = (ax + dx * t - x, az + dz * t - z);
            best = best.min((px * px + pz * pz).sqrt());
        }
        best
    }
    fn add_light(&mut self, x: i32, y: i32, z: i32) {
        self.set(x, y, z, LANTERN);
        self.w.lights.push(PointLight {
            pos: Vec3::new(x as f32 + 0.5, y as f32 + 0.4, z as f32 + 0.5),
            color: Vec3::new(1.0, 0.62, 0.30),
            intensity: 6.0,
        });
    }
    fn lamp_post(&mut self, x: i32, z: i32) {
        let t = self.top(x, z);
        if t < 0 {
            return;
        }
        self.set(x, t + 1, z, POST);
        self.set(x, t + 2, z, POST);
        self.add_light(x, t + 3, z);
    }
    fn line(&mut self, a: Vec3, b: Vec3, block: Block, thick: bool) {
        let steps = ((b - a).length() * 2.0).ceil() as i32;
        for i in 0..=steps {
            let p = a.lerp(b, i as f32 / steps.max(1) as f32);
            let (x, y, z) = (p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32);
            self.set(x, y, z, block);
            if thick {
                self.set(x + 1, y, z, block);
                self.set(x, y, z + 1, block);
            }
        }
    }
    fn leaf_ball(&mut self, c: Vec3, r: f32, density: f32, seed: u32) {
        let ri = r.ceil() as i32;
        for dx in -ri..=ri {
            for dy in -ri..=ri {
                for dz in -ri..=ri {
                    let d = ((dx * dx + dy * dy + dz * dz) as f32).sqrt();
                    let (x, y, z) = (c.x as i32 + dx, c.y as i32 + dy, c.z as i32 + dz);
                    if d <= r && hash2(x * 7 + y, z * 13 - y, seed) < density && self.w.get(x, y, z) == AIR {
                        self.set(x, y, z, LEAVES);
                    }
                }
            }
        }
    }
    fn small_tree(&mut self, x: i32, z: i32, h: i32, seed: u32) {
        let t = self.top(x, z);
        if t < 0 {
            return;
        }
        for y in t + 1..=t + h {
            self.set(x, y, z, LOG);
        }
        let c = Vec3::new(x as f32 + 0.5, (t + h) as f32 + 0.5, z as f32 + 0.5);
        self.leaf_ball(c, 2.6, 0.8, seed);
    }
}

pub fn build() -> World {
    let outline = OUTLINE_PX.iter().map(|&(x, z)| (x * PX_TO_GRID, z * PX_TO_GRID)).collect();
    let mut b = Builder { w: World::new(NX, NY, NZ), top: vec![-1; (NX * NZ) as usize], outline };

    // ---------------------------------------------------------------
    // 1) Isla: terreno y parte inferior rocosa (flotante)
    // ---------------------------------------------------------------
    let north_circle = (28.0f32, 13.5f32);
    for z in 0..NZ {
        for x in 0..NX {
            let (fx, fz) = (x as f32 + 0.5, z as f32 + 0.5);
            // Borde irregular con ruido
            let jx = (fbm(fx * 0.18, fz * 0.18, 1) - 0.5) * 3.0;
            let jz = (fbm(fx * 0.18, fz * 0.18, 2) - 0.5) * 3.0;
            if !b.inside(fx + jx, fz + jz) {
                continue;
            }
            let d = b.edge_dist(fx, fz);
            let mut top = BASE;

            // Colina del círculo de piedra al norte
            let dn = ((fx - north_circle.0).powi(2) + (fz - north_circle.1).powi(2)).sqrt();
            if dn < 7.0 {
                top = BASE + 1;
            }
            // Plaza baja (suroeste) + zona de llegada de la escalera
            let in_plaza = (x >= 2 && x <= 31 && z >= 51) || (z >= 49 && x <= 37 && x >= 22);
            if in_plaza {
                top = PLAZA;
            }
            // Borde un poco desgastado
            if d < 1.5 && hash2(x, z, 9) > 0.55 {
                top -= 1;
            }

            // Parte de abajo: roca en forma de cono irregular
            let depth = 2.0 + d * 1.35 + fbm(fx * 0.25, fz * 0.25, 3) * 8.0;
            let bottom = (top - depth as i32).max(1);
            for y in bottom..top {
                b.set(x, y, z, ROCK);
            }
            // Superficie: roca oscura con parches de musgo
            let moss = fbm(fx * 0.13, fz * 0.13, 4);
            let surf = if moss > 0.36 { GRASS } else { ROCK };
            b.set(x, top, z, surf);
            b.set_top(x, z, top);
        }
    }

    // Estalactitas colgando bajo la isla
    for i in 0..40 {
        let x = (hash2(i, 0, 21) * NX as f32) as i32;
        let z = (hash2(i, 1, 21) * NZ as f32) as i32;
        let mut y = 1;
        while y < NY && b.w.get(x, y, z) == AIR {
            y += 1;
        }
        if y < BASE && y > 1 {
            let len = 2 + (hash2(i, 2, 21) * 6.0) as i32;
            for k in 1..=len {
                b.set(x, y - k, z, ROCK);
            }
        }
    }

    // ---------------------------------------------------------------
    // 2) Musgo y flores: zona verde al oeste, flores alrededor del círculo
    // ---------------------------------------------------------------
    for z in 0..NZ {
        for x in 0..NX {
            if b.top(x, z) < 0 {
                continue;
            }
            let (fx, fz) = (x as f32 + 0.5, z as f32 + 0.5);
            // Hiedra/musgo denso al oeste
            if x < 18 && z > 18 && z < 52 {
                b.surface(x, z, GRASS);
            }
            // Círculo de piedra: pasto adentro y anillo de flores blancas
            let dn = ((fx - north_circle.0).powi(2) + (fz - north_circle.1).powi(2)).sqrt();
            if dn < 7.0 {
                let ring = if dn > 5.0 { FLOWERS } else { GRASS };
                b.surface(x, z, ring);
            }
            // Campo de flores blancas al este (alrededor del árbol hacia el sur)
            let (ex, ez) = ((fx - 51.0) / 10.5, (fz - 49.0) / 17.0);
            let field = ex * ex + ez * ez + (fbm(fx * 0.2, fz * 0.2, 5) - 0.5) * 0.9;
            if field < 1.0 {
                let f = fbm(fx * 0.35, fz * 0.35, 6);
                b.surface(x, z, if f > 0.44 { FLOWERS } else { GRASS });
            }
            // Flores sueltas por toda la isla
            if hash2(x, z, 7) > 0.96 && b.w.get(x, b.top(x, z), z) == GRASS {
                b.surface(x, z, FLOWERS);
            }
        }
    }

    // ---------------------------------------------------------------
    // 3) Plaza baja con jardín decorativo y camino perimetral
    // ---------------------------------------------------------------
    for z in 51..NZ {
        for x in 2..=31 {
            if b.top(x, z) != PLAZA && b.top(x, z) != PLAZA - 1 {
                continue;
            }
            let garden = x >= 8 && x <= 25 && z >= 57 && z <= 74;
            let border = x == 8 || x == 25 || z == 57 || z == 74;
            let block = if garden {
                if border {
                    COBBLE
                } else if x == 16 || x == 17 || z == 65 {
                    COBBLE // caminos en cruz dentro del jardín
                } else if x == 9 || x == 24 || z == 58 || z == 73 {
                    FLOWERS
                } else {
                    GRASS
                }
            } else if b.edge_dist(x as f32 + 0.5, z as f32 + 0.5) < 2.2 {
                FLOWERS // borde de flores de la plaza
            } else {
                COBBLE
            };
            b.surface(x, z, block);
        }
    }
    // Arbolito seco en el jardín
    {
        let (x, z) = (12, 61);
        let t = b.top(x, z);
        for y in t + 1..t + 6 {
            b.set(x, y, z, LOG);
        }
        let base = Vec3::new(x as f32 + 0.5, (t + 5) as f32, z as f32 + 0.5);
        b.line(base, base + Vec3::new(2.5, 2.0, 1.0), LOG, false);
        b.line(base, base + Vec3::new(-2.0, 2.5, -1.5), LOG, false);
        b.line(base - Vec3::new(0.0, 1.0, 0.0), base + Vec3::new(0.5, 1.5, 3.0), LOG, false);
    }

    // ---------------------------------------------------------------
    // 4) Taller / iglesia central
    // ---------------------------------------------------------------
    let (x0, x1, z0, z1) = (24, 36, 25, 45);
    // Explanada de adoquín alrededor
    for z in z0 - 2..=z1 + 1 {
        for x in x0 - 2..=x1 + 2 {
            if b.top(x, z) == BASE {
                b.surface(x, z, COBBLE);
            }
        }
    }
    // Suelo, cimientos y paredes
    for z in z0..=z1 {
        for x in x0..=x1 {
            let wall = x == x0 || x == x1 || z == z0 || z == z1;
            b.set(x, BASE, z, if wall { BRICKS } else { PLANKS });
            b.set_top(x, z, BASE);
            if wall {
                for y in BASE + 1..=BASE + 5 {
                    b.set(x, y, z, BRICKS);
                }
                b.set_top(x, z, BASE + 5);
            }
        }
    }
    // Ventanas laterales (vidrio con refracción)
    for &zc in &[28, 32, 36, 40] {
        for z in zc..=zc + 1 {
            for y in BASE + 2..=BASE + 4 {
                b.set(x0, y, z, GLASS);
                b.set(x1, y, z, GLASS);
            }
        }
    }
    // Fachada sur: puerta y ventanas
    for x in 29..=31 {
        for y in BASE + 1..=BASE + 3 {
            b.set(x, y, z1, AIR);
        }
    }
    for &xc in &[26, 33] {
        for x in xc..=xc + 1 {
            for y in BASE + 2..=BASE + 3 {
                b.set(x, y, z1, GLASS);
            }
        }
    }
    // Ventanal norte
    for x in 28..=32 {
        for y in BASE + 2..=BASE + 5 {
            b.set(x, y, z0, GLASS);
        }
    }
    // Techo a dos aguas con tragaluces de vidrio
    for k in 0..8 {
        let y = BASE + 6 + k;
        let (xl, xr) = (x0 - 1 + k, x1 + 1 - k);
        if xl > xr {
            break;
        }
        for z in z0 - 1..=z1 + 1 {
            let skylight = (k == 3 || k == 4) && ((29..=30).contains(&z) || (35..=36).contains(&z) || (40..=41).contains(&z));
            b.set(xl, y, z, if skylight { GLASS } else { ROOF });
            b.set(xr, y, z, ROOF);
            if xl == xr || xl + 1 == xr {
                b.set(xr, y, z, ROOF);
            }
            // Frontones de ladrillo
            if z == z0 || z == z1 {
                for x in xl + 1..xr {
                    b.set(x, y, z, BRICKS);
                }
            }
        }
    }
    // Rosetón de vidrio en los frontones
    for &z in &[z0, z1] {
        b.set(30, BASE + 8, z, GLASS);
        b.set(29, BASE + 8, z, GLASS);
        b.set(31, BASE + 8, z, GLASS);
        b.set(30, BASE + 9, z, GLASS);
        b.set(30, BASE + 7, z, GLASS);
    }
    // Chimenea
    for y in BASE + 6..=BASE + 15 {
        b.set(33, y, 31, BRICKS);
        b.set(33, y, 32, BRICKS);
    }
    // Interior: estantes, mesa de trabajo y faroles
    for x in 25..=27 {
        for y in BASE + 1..=BASE + 3 {
            b.set(x, y, z0 + 1, PLANKS);
        }
    }
    for x in 33..=35 {
        for y in BASE + 1..=BASE + 3 {
            b.set(x, y, z0 + 1, PLANKS);
        }
    }
    for z in 31..=34 {
        b.set(27, BASE + 1, z, PLANKS);
        b.set(28, BASE + 1, z, PLANKS);
    }
    b.add_light(26, BASE + 1, 28);
    b.add_light(34, BASE + 1, 29);
    b.add_light(26, BASE + 1, 42);
    b.add_light(34, BASE + 1, 42);
    b.add_light(30, BASE + 4, 36);

    // Jardineras de piedra alrededor del taller (como en el mapa)
    for &zc in &[27, 35, 43] {
        for &xc in &[21, 38] {
            for dz in 0..2 {
                for dx in 0..2 {
                    let (x, z) = (xc + dx, zc + dz);
                    b.set(x, BASE + 1, z, BRICKS);
                    b.set(x, BASE + 2, z, LEAVES);
                    b.set_top(x, z, BASE + 2);
                }
            }
        }
    }

    // ---------------------------------------------------------------
    // 5) Escalera semicircular hacia la plaza
    // ---------------------------------------------------------------
    let sc = (30.5f32, 46.0f32);
    for z in 46..NZ {
        for x in 20..=41 {
            let r = ((x as f32 + 0.5 - sc.0).powi(2) + (z as f32 + 0.5 - sc.1).powi(2)).sqrt();
            if r >= 8.5 || b.top(x, z) < 0 {
                continue;
            }
            // Altura de la superficie en medios bloques
            let half = 2 * (BASE + 1) - if r < 3.0 { 0 } else if r < 4.5 { 1 } else if r < 6.0 { 2 } else if r < 7.3 { 3 } else { 4 };
            let full_top = half / 2 - 1;
            let old = b.top(x, z);
            for y in (full_top + 1).min(old + 1)..=old.max(full_top + 1) {
                b.set(x, y, z, AIR);
            }
            let start = old.min(full_top);
            for y in start..=full_top {
                b.set(x, y, z, if y == full_top { COBBLE } else { ROCK });
            }
            b.set_top(x, z, full_top);
            if half % 2 == 1 {
                b.set(x, full_top + 1, z, COBBLE_SLAB);
            }
        }
    }

    // ---------------------------------------------------------------
    // 6) Caminos
    // ---------------------------------------------------------------
    // Camino norte: del taller al círculo de piedra
    for z in 16..z0 - 2 {
        for x in 29..=31 {
            b.surface(x, z, COBBLE);
        }
    }
    // Losas del camino oeste (piedras de paso)
    let stones = [(13, 51), (12, 47), (11, 43), (11, 39), (12, 35), (14, 31), (17, 27), (20, 23), (23, 19)];
    for &(sx, sz) in &stones {
        for dz in 0..2 {
            for dx in 0..2 {
                b.place_above(sx + dx, sz + dz, COBBLE_SLAB);
            }
        }
    }
    // Sendero de tierra al este (junto al campo de flores)
    let path = [(40.0, 22.0), (40.5, 30.0), (39.5, 40.0), (40.0, 50.0), (41.0, 60.0), (38.5, 70.0), (34.0, 77.0)];
    for w in path.windows(2) {
        let (a, c) = (w[0], w[1]);
        let steps = 30;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let (px, pz) = (a.0 + (c.0 - a.0) * t, a.1 + (c.1 - a.1) * t);
            let (x, z) = (px as i32, pz as i32);
            if b.top(x, z) >= BASE - 1 {
                b.surface(x, z, if (x + z) % 5 == 0 { COBBLE } else { DIRT });
            }
        }
    }

    // ---------------------------------------------------------------
    // 7) Círculo de piedra al norte: altar de mármol y pozo
    // ---------------------------------------------------------------
    for x in 27..=29 {
        for z in 12..=13 {
            b.place_above(x, z, MARBLE);
        }
    }
    b.place_above(28, 12, MARBLE);
    for x in 36..=37 {
        for z in 18..=19 {
            let t = b.top(x, z);
            for y in t - 3..=t {
                b.set(x, y, z, AIR);
            }
            b.set_top(x, z, t - 4);
        }
    }

    // ---------------------------------------------------------------
    // 8) Fuentes de mármol con agua (refracción + reflexión)
    // ---------------------------------------------------------------
    for &(cx, cz) in &[(17.0f32, 37.5f32), (17.5f32, 43.5f32)] {
        for z in (cz - 3.0) as i32..=(cz + 3.0) as i32 {
            for x in (cx - 3.0) as i32..=(cx + 3.0) as i32 {
                let r = ((x as f32 + 0.5 - cx).powi(2) + (z as f32 + 0.5 - cz).powi(2)).sqrt();
                if r >= 2.6 {
                    continue;
                }
                b.set(x, BASE - 1, z, MARBLE);
                if r >= 1.5 {
                    b.set(x, BASE, z, MARBLE);
                    b.set(x, BASE + 1, z, MARBLE);
                } else {
                    b.set(x, BASE, z, WATER);
                    b.set(x, BASE + 1, z, WATER);
                    b.set(x, BASE + 2, z, AIR);
                }
                b.set_top(x, z, BASE + 1);
            }
        }
    }

    // ---------------------------------------------------------------
    // 9) El gran árbol con raíces retorcidas
    // ---------------------------------------------------------------
    let (tx, tz) = (45.0f32, 17.5f32);
    let ground = BASE + 1;
    let mut trunk_top = Vec3::ZERO;
    for y in ground..ground + 14 {
        let k = (y - ground) as f32;
        let r = 1.9 - k * 0.09;
        let (ox, oz) = ((k * 0.35).sin() * 0.9, (k * 0.3).cos() * 0.6 - 0.6);
        let (cx, cz) = (tx + ox, tz + oz);
        for z in (cz - 3.0) as i32..=(cz + 3.0) as i32 {
            for x in (cx - 3.0) as i32..=(cx + 3.0) as i32 {
                let d = ((x as f32 + 0.5 - cx).powi(2) + (z as f32 + 0.5 - cz).powi(2)).sqrt();
                if d < r {
                    b.set(x, y, z, LOG);
                }
            }
        }
        trunk_top = Vec3::new(cx, y as f32, cz);
    }
    // Ramas
    let branches = [
        (Vec3::new(-6.0, 5.0, -2.0), 0.0),
        (Vec3::new(5.0, 6.0, -3.0), 1.0),
        (Vec3::new(2.0, 7.0, 5.0), 2.0),
        (Vec3::new(-4.0, 4.0, 5.0), 3.0),
        (Vec3::new(6.0, 3.0, 4.0), 4.0),
        (Vec3::new(0.0, 7.5, -5.0), 5.0),
    ];
    for &(dir, i) in &branches {
        let start = trunk_top - Vec3::new(0.0, 2.0 + i * 0.5, 0.0);
        let mid = start + dir * 0.55 + Vec3::new(0.0, 1.0, 0.0);
        let end = start + dir;
        b.line(start, mid, LOG, true);
        b.line(mid, end, LOG, false);
        // Pequeñas ramitas al final
        b.line(end, end + Vec3::new(dir.z * 0.3, 1.5, -dir.x * 0.3), LOG, false);
        if (i as i32) % 2 == 0 {
            b.leaf_ball(end + Vec3::new(0.0, 1.0, 0.0), 2.2, 0.45, i as u32 + 40);
        }
    }
    // Raíces en espiral sobre el suelo
    for i in 0..11 {
        let ang = i as f32 / 11.0 * std::f32::consts::TAU + 0.3;
        let len = 4.0 + hash2(i, 3, 33) * 4.0;
        let mut prev = Vec3::new(tx, ground as f32 + 0.5, tz);
        let segs = 6;
        for s in 1..=segs {
            let t = s as f32 / segs as f32;
            let a = ang + t * 0.9; // se enroscan
            let p = Vec3::new(
                tx + a.cos() * len * t,
                ground as f32 + 0.5 - t * 1.2,
                tz + a.sin() * len * t,
            );
            // Las raíces no deben salirse de la isla
            if b.top(p.x.floor() as i32, p.z.floor() as i32) < BASE - 1 {
                break;
            }
            b.line(prev, p, LOG, s <= 2);
            prev = p;
        }
    }

    // ---------------------------------------------------------------
    // 9b) Muro de piedras: encierra la zona del árbol y el campo de flores
    //     (sigue el sendero de tierra por su lado este, de norte a sur)
    // ---------------------------------------------------------------
    let wall_line = [(39.5f32, 6.0f32), (40.0, 22.0), (40.5, 30.0), (39.5, 40.0), (40.0, 50.0), (41.0, 60.0), (38.5, 70.0), (34.0, 77.0), (33.0, 82.0)];
    let gates = [(30, 33), (54, 56)]; // entradas (rangos de z sin muro)
    let mut prev_x: Option<i32> = None;
    for z in 6..=82 {
        // x del sendero en esta z (interpolación entre los puntos)
        let seg = wall_line.windows(2).find(|w| (z as f32) >= w[0].1 && (z as f32) <= w[1].1);
        let Some(w) = seg else { continue };
        let t = (z as f32 - w[0].1) / (w[1].1 - w[0].1);
        let x = (w[0].0 + (w[1].0 - w[0].0) * t + 1.5).floor() as i32;
        let in_gate = gates.iter().any(|&(a, c)| z >= a && z <= c);
        // Rellenar saltos en diagonal para que el muro no tenga huecos
        let from = prev_x.unwrap_or(x);
        prev_x = Some(x);
        if in_gate {
            continue;
        }
        for wx in from.min(x)..=from.max(x) {
            if b.top(wx, z) >= BASE - 1 && b.w.get(wx, b.top(wx, z), z) != COBBLE_SLAB {
                b.place_above(wx, z, if hash2(wx, z, 90) > 0.5 { COBBLE } else { ROCK });
            }
        }
    }

    // ---------------------------------------------------------------
    // 10) Lápidas en el campo de flores
    // ---------------------------------------------------------------
    let graves = [(55, 42), (52, 45), (47, 51), (44, 48), (50, 53), (51, 49), (56, 50), (48, 57), (54, 58), (45, 60), (58, 46), (53, 38)];
    for &(x, z) in &graves {
        b.place_above(x, z, GRAVE);
    }

    // ---------------------------------------------------------------
    // 11) Vegetación: hiedra y árboles al oeste, arbustos al sureste
    // ---------------------------------------------------------------
    for z in 0..NZ {
        for x in 0..NX {
            let t = b.top(x, z);
            if t < 0 || b.w.get(x, t, z) != GRASS {
                continue;
            }
            let (fx, fz) = (x as f32, z as f32);
            let bush = fbm(fx * 0.3, fz * 0.3, 8);
            let west = x < 16 && z > 18 && z < 52;
            let southeast = x > 40 && z > 68;
            if (west && bush > 0.52) || (southeast && bush > 0.55) || bush > 0.72 {
                b.place_above(x, z, LEAVES);
                if bush > 0.62 {
                    b.place_above(x, z, LEAVES);
                }
            }
        }
    }
    b.small_tree(8, 28, 6, 60);
    b.small_tree(7, 40, 7, 61);
    b.small_tree(6, 47, 5, 62);
    b.small_tree(47, 73, 5, 63);
    b.small_tree(20, 14, 5, 64);

    // ---------------------------------------------------------------
    // 12) Faroles exteriores
    // ---------------------------------------------------------------
    for &(x, z) in &[(22, 48), (39, 48), (27, 21), (33, 21), (6, 55), (28, 78), (40, 33), (13, 69)] {
        b.lamp_post(x, z);
    }

    b.w
}

