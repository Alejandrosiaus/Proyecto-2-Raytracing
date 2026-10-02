"""
Generador de assets del diorama (texturas 16x16 estilo Minecraft + skybox cubemap).
Es una herramienta auxiliar: el proyecto en Rust solo lee los PNG resultantes.

Uso:  python tools/generate_assets.py     (requiere numpy y pillow)
"""
import os
import numpy as np
from PIL import Image

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets")
TEX = os.path.join(ROOT, "textures")
SKY = os.path.join(ROOT, "skybox")
os.makedirs(TEX, exist_ok=True)
os.makedirs(SKY, exist_ok=True)

N = 16
rng = np.random.default_rng(1234)


def save(name, rgb, alpha=None):
    rgb = np.clip(rgb, 0, 255).astype(np.uint8)
    if alpha is None:
        alpha = np.full(rgb.shape[:2], 255, np.uint8)
    img = np.dstack([rgb, np.clip(alpha, 0, 255).astype(np.uint8)])
    Image.fromarray(img, "RGBA").save(os.path.join(TEX, name + ".png"))


def noise_tex(base, var, n=N):
    base = np.array(base, float)
    k = rng.normal(0, 1, (n, n, 1)) * var
    return np.broadcast_to(base, (n, n, 3)) + k


def voronoi(n_seeds, n=N):
    """Devuelve (id de celda, distancia al borde) para piedras irregulares (tileable)."""
    seeds = rng.uniform(0, n, (n_seeds, 2))
    yy, xx = np.mgrid[0:n, 0:n] + 0.5
    d = []
    for sx, sy in seeds:
        dx = np.abs(xx - sx); dx = np.minimum(dx, n - dx)
        dy = np.abs(yy - sy); dy = np.minimum(dy, n - dy)
        d.append(np.sqrt(dx * dx + dy * dy))
    d = np.stack(d)
    order = np.sort(d, axis=0)
    return np.argmin(d, axis=0), order[1] - order[0]


# ---------------- Texturas de bloques ----------------

# Piedra oscura de la isla
rock = noise_tex((70, 70, 74), 9)
cid, edge = voronoi(6)
rock[edge < 0.9] *= 0.7
save("rock", rock)

# Musgo / pasto oscuro (paleta desaturada, ambiente del sueño)
grass = noise_tex((62, 84, 52), 10)
spots = rng.random((N, N)) < 0.15
grass[spots] *= 0.75
save("grass_top", grass)

# Tierra
dirt = noise_tex((88, 72, 58), 10)
save("dirt", dirt)

# Lado del bloque de pasto: tierra con borde de musgo irregular arriba
side = dirt.copy()
for x in range(N):
    h = 3 + rng.integers(0, 3)
    side[:h, x] = grass[:h, x]
save("grass_side", side)

# Flores blancas sobre pasto
flowers = grass.copy()
for _ in range(9):
    fx, fy = rng.integers(0, N - 1, 2)
    for dx, dy in [(0, 0), (1, 0), (0, 1), (1, 1)]:
        flowers[(fy + dy) % N, (fx + dx) % N] = rng.uniform(215, 245) * np.array([1, 1, 0.97])
    flowers[fy, fx] = (230, 215, 150)  # centro amarillento
for _ in range(10):  # pétalos sueltos
    fx, fy = rng.integers(0, N, 2)
    flowers[fy, fx] = (200, 200, 195)
save("flowers_top", flowers)

# Adoquín
cid, edge = voronoi(10)
tones = rng.uniform(95, 135, 10)
cob = np.zeros((N, N, 3))
for i in range(10):
    cob[cid == i] = tones[i]
cob += rng.normal(0, 6, (N, N, 1))
cob *= np.array([1.0, 0.98, 0.95])
cob[edge < 1.0] = (48, 47, 46)
save("cobble", cob)

# Tablones de madera
planks = np.zeros((N, N, 3))
for row in range(4):
    tone = rng.uniform(0.85, 1.1)
    planks[row * 4:(row + 1) * 4] = np.array([110, 74, 46]) * tone
    planks[row * 4 + 3] *= 0.6  # separación entre tablas
    cut = rng.integers(0, N)
    planks[row * 4:(row + 1) * 4 - 1, cut] *= 0.7
planks += rng.normal(0, 5, (N, N, 1))
planks[:, :, :] *= (1 + 0.08 * np.sin(np.arange(N) * 1.7))[None, :, None]
save("planks", planks)

# Ladrillos de piedra gótica
bricks = noise_tex((128, 122, 112), 7)
for row in range(4):
    y0 = row * 4
    bricks[y0 + 3] = (62, 58, 54)
    off = 0 if row % 2 == 0 else 4
    for x in (off, off + 8):
        bricks[y0:y0 + 3, x % N] = (62, 58, 54)
    bricks[y0:y0 + 3] *= rng.uniform(0.85, 1.05)
save("bricks", bricks)

# Tejas de pizarra
roof = np.zeros((N, N, 3))
for row in range(4):
    y0 = row * 4
    off = 0 if row % 2 == 0 else 2
    for tile in range(4):
        x0 = (tile * 4 + off) % N
        tone = rng.uniform(0.8, 1.15)
        for dy in range(4):
            for dx in range(4):
                c = np.array([58, 62, 74]) * tone * (1.0 - dy * 0.06)
                roof[y0 + dy, (x0 + dx) % N] = c
        roof[y0 + 3, x0:x0 + 4] *= 0.55
        roof[y0:y0 + 4, x0 % N] *= 0.7
roof += rng.normal(0, 4, (N, N, 1))
save("roof", roof)

# Vidrio de ventana gótica: marco de hierro opaco + paneles translúcidos
glass = np.broadcast_to(np.array([200, 215, 225], float), (N, N, 3)).copy()
galpha = np.full((N, N), 18.0)
frame = np.zeros((N, N), bool)
frame[0, :] = frame[-1, :] = frame[:, 0] = frame[:, -1] = True
frame[:, 7:9] = True     # parteluz vertical
frame[7:9, :] = True     # travesaño
glass[frame] = (40, 40, 44)
galpha[frame] = 255
for (x, y) in [(3, 3), (4, 2), (11, 11), (12, 10)]:  # brillos
    glass[y, x] = (250, 250, 255)
    galpha[y, x] = 90
save("glass", glass, galpha)

# Agua
yy, xx = np.mgrid[0:N, 0:N]
water = np.zeros((N, N, 3))
wave = np.sin(xx * 0.8 + yy * 0.3) * 0.5 + np.sin(yy * 1.1 - xx * 0.4) * 0.5
water[:] = np.array([40, 82, 105])
water += (wave * 14)[..., None]
save("water", water, np.full((N, N), 30))

# Tronco: corteza y anillos
bark = np.zeros((N, N, 3))
for x in range(N):
    tone = rng.uniform(0.7, 1.1)
    bark[:, x] = np.array([72, 56, 44]) * tone
bark += rng.normal(0, 6, (N, N, 1))
for _ in range(6):
    x, y = rng.integers(0, N, 2)
    bark[y:y + 3, x] *= 0.55
save("log_side", bark)

cx = cy = 7.5
r = np.sqrt((xx - cx) ** 2 + (yy - cy) ** 2)
logtop = np.zeros((N, N, 3))
logtop[:] = np.array([130, 100, 70])
logtop *= (0.85 + 0.15 * np.sin(r * 2.2))[..., None]
logtop[r > 6.8] = bark[r > 6.8]
save("log_top", logtop)

# Hojas / hiedra
leaves = noise_tex((46, 70, 42), 14)
holes = rng.random((N, N)) < 0.18
leaves[holes] *= 0.45
save("leaves", leaves)

# Farol (emisivo)
lantern = np.zeros((N, N, 3))
d = np.sqrt((xx - 7.5) ** 2 + (yy - 7.5) ** 2)
glow = np.clip(1.2 - d / 7, 0, 1)
lantern[:] = np.stack([255 * glow + 150, 190 * glow + 90, 90 * glow + 30], -1)
lantern[frame] = (35, 32, 30)
save("lantern", lantern)

# Mármol pulido
marble = noise_tex((205, 202, 196), 5)
vein = np.abs(np.sin(xx * 0.45 + yy * 0.9 + np.sin(yy * 0.7) * 2.0))
marble[vein < 0.12] *= 0.72
save("marble", marble)


# ---------------- Skybox (cubemap 6 caras) ----------------
S = 512
perm = rng.permutation(256)
perm = np.concatenate([perm, perm])


def fade(t):
    return t * t * (3 - 2 * t)


def hash3(ix, iy, iz):
    return perm[(perm[(perm[ix & 255] + iy) & 255] + iz) & 255] / 255.0


def value_noise(p):
    ip = np.floor(p).astype(int)
    f = p - ip
    u = fade(f)
    ix, iy, iz = ip[..., 0], ip[..., 1], ip[..., 2]
    res = 0
    for dx in (0, 1):
        for dy in (0, 1):
            for dz in (0, 1):
                w = (u[..., 0] if dx else 1 - u[..., 0]) * \
                    (u[..., 1] if dy else 1 - u[..., 1]) * \
                    (u[..., 2] if dz else 1 - u[..., 2])
                res = res + w * hash3(ix + dx, iy + dy, iz + dz)
    return res


def fbm(p, octaves=6):
    total, amp, norm = 0, 0.5, 0
    for _ in range(octaves):
        total = total + amp * value_noise(p)
        norm += amp
        p = p * 2.03
        amp *= 0.5
    return total / norm


def smoothstep(a, b, x):
    t = np.clip((x - a) / (b - a), 0, 1)
    return t * t * (3 - 2 * t)


# Debe coincidir con MOON_DIR en src/renderer.rs
moon = np.array([-0.35, 0.42, -0.84])
moon /= np.linalg.norm(moon)


# Pilares lejanos que salen del mar de niebla (como en el Sueño del Cazador)
prng = np.random.default_rng(77)
PILLARS = []
for i in range(46):
    k = prng.uniform(0.0, 1.0)            # 0 = cerca, 1 = muy lejos
    PILLARS.append(dict(
        az=prng.uniform(-np.pi, np.pi),
        k=k,
        w=0.008 + (1 - k) ** 2 * 0.022,    # ancho angular
        top=prng.uniform(0.12, 0.42) + (1 - k) * 0.38,
        seed=i,
    ))
PILLARS.sort(key=lambda p: -p["k"])      # dibujar de lejos a cerca


def sky_color(d):
    y = d[..., 1]
    el = np.arcsin(np.clip(y, -1, 1))
    az = np.arctan2(d[..., 0], d[..., 2])

    # Cielo nublado gris (sin nubes marcadas)
    zenith = np.array([92, 98, 112], float)
    horizon = np.array([186, 189, 194], float)
    t = np.clip(y, 0, 1) ** 0.6
    col = horizon * (1 - t[..., None]) + zenith * t[..., None]
    streak = fbm(np.stack([d[..., 0] * 1.5, d[..., 1] * 9.0, d[..., 2] * 1.5], -1) + 5.0)
    col = col * (0.93 + 0.12 * streak[..., None])

    # Mar de niebla bajo el horizonte
    fog_hi = np.array([192, 194, 199], float)
    fog_lo = np.array([128, 131, 140], float)
    tb = np.clip(-y * 1.6, 0, 1)[..., None]
    fog_sea = fog_hi * (1 - tb) + fog_lo * tb
    fog_sea = fog_sea * (0.95 + 0.08 * fbm(d * 4.0 + 1.0)[..., None])
    col = np.where((y < 0)[..., None], fog_sea, col)

    # Luna tenue con halo
    c = np.clip((d * moon).sum(-1), -1, 1)
    halo = np.exp(-(1 - c) * 40)[..., None]
    col = col + np.array([120, 120, 110]) * halo * 0.45
    disc = smoothstep(0.99845, 0.9987, c)[..., None]
    moon_col = np.array([236, 233, 218]) * (0.85 + 0.15 * fbm(d * 60.0)[..., None])
    col = col * (1 - disc) + moon_col * disc

    # Pilares
    moon_az = np.arctan2(moon[0], moon[2])
    cos_el = np.cos(el)
    for p in PILLARS:
        da = (az - p["az"] + np.pi) % (2 * np.pi) - np.pi
        x = da * cos_el / p["w"]                       # -1..1 a lo ancho del pilar
        top = p["top"] + 0.025 * (value_noise(np.stack([x * 2 + p["seed"] * 7, x * 0 + p["seed"], x * 0], -1)) - 0.5)
        mask = (np.abs(x) < 1.0) & (el < top) & (el > -0.32)
        if not mask.any():
            continue
        # Sombreado de cilindro: un lado iluminado por la luna
        side = np.sin(moon_az - p["az"])
        shade = 0.62 + 0.28 * (-x * np.sign(side)) - 0.15 * x ** 4
        bands = value_noise(np.stack([x * 0 + p["seed"] * 3.1, el * 40.0, x * 0 + 2.0], -1))
        stone = np.array([118, 118, 124], float) * (shade * (0.9 + 0.12 * bands))[..., None]
        # Niebla: más fuerte cerca del horizonte y en los pilares lejanos
        # Niebla: los pilares nacen del mar de niebla y los lejanos se desvanecen
        fogf = np.clip((0.06 - el) / 0.36, 0, 1) ** 1.1
        amount = np.clip(fogf + p["k"] * 0.38, 0, 1)[..., None]
        pillar = stone * (1 - amount) + col * amount
        col = np.where(mask[..., None], pillar, col)

    # Banda de neblina que baña la base de todo
    mist = np.exp(-(y / 0.06) ** 2)[..., None] * 0.45
    col = col * (1 - mist) + np.array([210, 211, 215]) * mist
    return col


faces = {
    "px": lambda u, v: np.stack([np.ones_like(u), -v, -u], -1),
    "nx": lambda u, v: np.stack([-np.ones_like(u), -v, u], -1),
    "py": lambda u, v: np.stack([u, np.ones_like(u), v], -1),
    "ny": lambda u, v: np.stack([u, -np.ones_like(u), -v], -1),
    "pz": lambda u, v: np.stack([u, -v, np.ones_like(u)], -1),
    "nz": lambda u, v: np.stack([-u, -v, -np.ones_like(u)], -1),
}
jj, ii = np.mgrid[0:S, 0:S]
u = (ii + 0.5) / S * 2 - 1
v = (jj + 0.5) / S * 2 - 1
for name, f in faces.items():
    d = f(u, v)
    d = d / np.linalg.norm(d, axis=-1, keepdims=True)
    col = np.clip(sky_color(d), 0, 255).astype(np.uint8)
    Image.fromarray(col, "RGB").save(os.path.join(SKY, name + ".png"))
    print("skybox", name)

print("listo")
