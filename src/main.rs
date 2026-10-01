
mod camera;
mod material;
mod math;
mod ray;
mod renderer;
mod scene;
mod skybox;
mod texture;
mod world;

use camera::OrbitCamera;
use math::Vec3;
use raylib::prelude::*;
use renderer::{render_pass, Accum, Scene};
use std::f32::consts::TAU;
use std::time::Instant;

const WIN_W: i32 = 1280;
const WIN_H: i32 = 800;

const LOW: (usize, usize) = (320, 200);
const HIGH: (usize, usize) = (640, 400);
const HIGH_MAX_SAMPLES: u32 = 48;

const VIDEO_FRAMES: u32 = 300;
const VIDEO_SPP: u32 = 3;
const VIDEO_RES: (usize, usize) = (1280, 800);

fn default_camera() -> OrbitCamera {
    OrbitCamera::new(Vec3::new(32.0, 30.0, 43.0), 88.0)
}


fn save_png(path: &str, acc: &Accum) {
    let mut img = Image::gen_image_color(acc.w as i32, acc.h as i32, Color::BLACK);
    for y in 0..acc.h {
        for x in 0..acc.w {
            let i = (y * acc.w + x) * 4;
            img.draw_pixel(x as i32, y as i32, Color::new(acc.rgba[i], acc.rgba[i + 1], acc.rgba[i + 2], 255));
        }
    }
    img.export_image(path);
}


fn video_camera(base: &OrbitCamera, frame: u32) -> OrbitCamera {
    let t = frame as f32 / VIDEO_FRAMES as f32;
    let mut c = *base;
    c.yaw = base.yaw + t * TAU;
    c.distance = base.distance * (1.0 - 0.38 * (t * TAU).sin().max(0.0) - 0.15 * (t * TAU * 2.0).sin().max(0.0));
    c.pitch = base.pitch + 0.12 * (t * TAU * 2.0).sin();
    c
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let t0 = Instant::now();
    let world = scene::build();
    println!("Escena: {} bloques, {} faroles ({:.0} ms)", world.count_blocks(), world.lights.len(), t0.elapsed().as_millis());
    let scene = Scene::new(world);
    for m in &scene.materials {
        println!(
            "  material {:<17} albedo={:?} spec={:>5.1} transp={:.2} refl={:.2} ior={:.2}",
            m.name, m.albedo, m.specular, m.transparency, m.reflectivity, m.ior
        );
    }

    
    
    if let Some(i) = args.iter().position(|a| a == "--shot") {
        let num = |k: usize| args.get(i + k).and_then(|s| s.parse::<f32>().ok());
        let spp = num(1).unwrap_or(16.0) as u32;
        let mut cam = default_camera();
        cam.yaw = num(2).unwrap_or(cam.yaw);
        cam.pitch = num(3).unwrap_or(cam.pitch);
        cam.distance = num(4).unwrap_or(cam.distance);
        let mut acc = Accum::new(VIDEO_RES.0, VIDEO_RES.1);
        let t = Instant::now();
        for _ in 0..spp {
            render_pass(&scene, &cam.basis(), &mut acc);
        }
        println!("Render {}x{} con {} spp en {:.2} s", acc.w, acc.h, spp, t.elapsed().as_secs_f32());
        save_png("diorama.png", &acc);
        println!("Guardado diorama.png");
        return;
    }

    let (mut rl, thread) = raylib::init()
        .size(WIN_W, WIN_H)
        .title("El Sueño del Cazador - Diorama con Raytracing")
        .build();
    rl.set_target_fps(60);

    let mut low = Accum::new(LOW.0, LOW.1);
    let mut high = Accum::new(HIGH.0, HIGH.1);
    let mut video = Accum::new(VIDEO_RES.0, VIDEO_RES.1);
    let mk_tex = |rl: &mut RaylibHandle, w: usize, h: usize| {
        let img = Image::gen_image_color(w as i32, h as i32, Color::BLACK);
        rl.load_texture_from_image(&thread, &img).expect("textura")
    };
    let mut low_tex = mk_tex(&mut rl, LOW.0, LOW.1);
    let mut high_tex = mk_tex(&mut rl, HIGH.0, HIGH.1);
    let mut video_tex = mk_tex(&mut rl, VIDEO_RES.0, VIDEO_RES.1);

    let mut cam = default_camera();
    let mut auto_rotate = false;
    let mut show_help = true;
    let mut last_move = Instant::now();
    let mut video_frame: Option<u32> = None;
    let mut video_base = cam;
    let mut message = String::new();
    let mut render_ms = 0.0f32;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time().min(0.1);

        
        let mut moved = false;
        let speed = 1.4 * dt;
        let key = |k| rl.is_key_down(k);
        let (mut dyaw, mut dpitch, mut dzoom) = (0.0, 0.0, 0.0);
        if key(KeyboardKey::KEY_LEFT) || key(KeyboardKey::KEY_A) { dyaw -= speed; }
        if key(KeyboardKey::KEY_RIGHT) || key(KeyboardKey::KEY_D) { dyaw += speed; }
        if key(KeyboardKey::KEY_UP) || key(KeyboardKey::KEY_W) { dpitch += speed; }
        if key(KeyboardKey::KEY_DOWN) || key(KeyboardKey::KEY_S) { dpitch -= speed; }
        if key(KeyboardKey::KEY_Q) { dzoom += 1.2 * dt; }
        if key(KeyboardKey::KEY_E) { dzoom -= 1.2 * dt; }
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            let d = rl.get_mouse_delta();
            dyaw -= d.x * 0.006;
            dpitch += d.y * 0.006;
        }
        dzoom += rl.get_mouse_wheel_move() * 0.08;
        if rl.is_key_pressed(KeyboardKey::KEY_R) { auto_rotate = !auto_rotate; }
        if rl.is_key_pressed(KeyboardKey::KEY_H) { show_help = !show_help; }
        if rl.is_key_pressed(KeyboardKey::KEY_C) { cam = default_camera(); moved = true; }
        if auto_rotate { dyaw += 0.35 * dt; }

        if dyaw != 0.0 || dpitch != 0.0 || dzoom != 0.0 {
            cam.rotate(dyaw, dpitch);
            cam.zoom(dzoom);
            moved = true;
        }
        if moved {
            last_move = Instant::now();
            high.reset();
        }

        
        if rl.is_key_pressed(KeyboardKey::KEY_P) && video_frame.is_none() {
            let mut shot = Accum::new(VIDEO_RES.0, VIDEO_RES.1);
            for _ in 0..16 {
                render_pass(&scene, &cam.basis(), &mut shot);
            }
            save_png("diorama.png", &shot);
            message = "Captura guardada en diorama.png".into();
        }
        if rl.is_key_pressed(KeyboardKey::KEY_V) {
            if video_frame.is_some() {
                video_frame = None;
                message = "Video cancelado".into();
            } else {
                std::fs::create_dir_all("video_frames").ok();
                video_base = cam;
                video_frame = Some(0);
                auto_rotate = false;
            }
        }

        
        let t = Instant::now();
        let mut showing = 0; 
        if let Some(f) = video_frame {
            let c = video_camera(&video_base, f);
            video.reset();
            for _ in 0..VIDEO_SPP {
                render_pass(&scene, &c.basis(), &mut video);
            }
            save_png(&format!("video_frames/frame_{:04}.png", f + 1), &video);
            video_tex.update_texture(&video.rgba).ok();
            showing = 2;
            if f + 1 >= VIDEO_FRAMES {
                video_frame = None;
                message = "Video listo: ffmpeg -framerate 30 -i video_frames/frame_%04d.png -c:v libx264 -pix_fmt yuv420p diorama.mp4".into();
            } else {
                video_frame = Some(f + 1);
            }
        } else if moved || last_move.elapsed().as_secs_f32() < 0.2 {
            low.reset();
            render_pass(&scene, &cam.basis(), &mut low);
            low_tex.update_texture(&low.rgba).ok();
        } else {
            if high.samples < HIGH_MAX_SAMPLES {
                render_pass(&scene, &cam.basis(), &mut high);
                high_tex.update_texture(&high.rgba).ok();
            }
            showing = 1;
        }
        if showing != 1 || high.samples < HIGH_MAX_SAMPLES {
            render_ms = render_ms * 0.8 + t.elapsed().as_secs_f32() * 1000.0 * 0.2;
        }

        
        let fps = rl.get_fps();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        let (tex, w, h) = match showing {
            1 => (&high_tex, HIGH.0, HIGH.1),
            2 => (&video_tex, VIDEO_RES.0, VIDEO_RES.1),
            _ => (&low_tex, LOW.0, LOW.1),
        };
        d.draw_texture_pro(
            tex,
            Rectangle::new(0.0, 0.0, w as f32, h as f32),
            Rectangle::new(0.0, 0.0, WIN_W as f32, WIN_H as f32),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );

        let shadow_text = |d: &mut RaylibDrawHandle, s: &str, x: i32, y: i32, size: i32, c: Color| {
            d.draw_text(s, x + 1, y + 1, size, Color::new(0, 0, 0, 180));
            d.draw_text(s, x, y, size, c);
        };
        if let Some(f) = video_frame {
            shadow_text(&mut d, &format!("GRABANDO VIDEO  frame {}/{}  (V = cancelar)", f, VIDEO_FRAMES), 12, 12, 22, Color::RED);
        } else {
            let status = match showing {
                1 => format!("{}x{}  muestras {}/{}", HIGH.0, HIGH.1, high.samples, HIGH_MAX_SAMPLES),
                _ => format!("{}x{}  (moviendo)", LOW.0, LOW.1),
            };
            shadow_text(&mut d, &format!("FPS {}  |  render {:.0} ms  |  {}", fps, render_ms, status), 12, 12, 18, Color::RAYWHITE);
        }
        if show_help && video_frame.is_none() {
            let lines = [
                "A/D o flechas: rotar   W/S: inclinar   Mouse: arrastrar para rotar",
                "Rueda o Q/E: acercar / alejar   R: rotacion automatica   C: reiniciar camara",
                "P: captura HD (diorama.png)   V: grabar video (video_frames/)   H: ocultar ayuda",
            ];
            for (i, l) in lines.iter().enumerate() {
                shadow_text(&mut d, l, 12, WIN_H - 78 + i as i32 * 24, 18, Color::new(230, 230, 230, 255));
            }
        }
        if !message.is_empty() {
            shadow_text(&mut d, &message, 12, 38, 16, Color::YELLOW);
        }
    }
}
