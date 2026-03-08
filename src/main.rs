
use std::ops::Index;
use std::thread;
use std::time::{Duration, Instant};

use sdl2::keyboard::Scancode;
use sdl2::event::Event;
use sdl2::mouse::MouseButton;

mod utils;
use crate::utils::*;
mod camera;
mod geometry;
mod renderer;
pub use crate::camera::{Camera, Screen};
use crate::geometry::{Mesh, Object, Primitive, Scene, Transform, Triangle, Vertex};

// Data clump
struct Settings {
    mouse_sensitivity: f32,
    scroll_sensitivity: f32,
    zoom_sensitivity: f32,
    camera_speed: f32,
}


type Keys = [bool; 256];
impl Index<Key> for Keys {
    type Output = bool;

    fn index(&self, key: Key) -> &Self::Output {
        match key {
            _ => &self[key as usize],
        }
    }
}

enum Key {
    W,
    A,
    S,
    D,

    Space,
    Shift,
    Ctrl,
    Esc,

    MouseLeft,
    MouseRight,
    MouseMiddle,
}

impl Key {
    fn from_scancode(scancode: Scancode) -> Option<Key> {
        match scancode {
            Scancode::W => Some(Key::W),
            Scancode::A => Some(Key::A),
            Scancode::S => Some(Key::S),
            Scancode::D => Some(Key::D),
            Scancode::Space => Some(Key::Space),
            Scancode::LShift => Some(Key::Shift),
            Scancode::LCtrl => Some(Key::Ctrl),
            Scancode::Escape => Some(Key::Esc),
            _ => None,
        }
    }

    fn from_mouse(mouse_btn: MouseButton) -> Option<Key> {
        match mouse_btn {
            MouseButton::Left => Some(Key::MouseLeft),
            MouseButton::Right => Some(Key::MouseRight),
            MouseButton::Middle => Some(Key::MouseMiddle),
            _ => None,
        }
    }
}

/// Handles user inputs and updates the camera accordingly. Returns true if the program should stop.
fn user_inputs(sdl_ctx: &mut sdl2::Sdl, cfg: &Settings, camera: &mut Camera, key_states: &mut Keys, dt: f32) -> bool {
    let (center_x, center_y) = camera.screen.get_screen_center_pix();

    let mut events = sdl_ctx.event_pump().unwrap();
    for event in events.poll_iter() {
        match event {
            Event::Quit {..} => {
                return true;
            },

            Event::KeyDown { scancode, .. } => {
                let key = match Key::from_scancode(scancode.unwrap()) {
                    Some(key) => key,
                    None => continue,
                };
                match key {
                    Key::Esc => return true,
                    _ => {},
                }

                key_states[key as usize] = true;
            },

            Event::KeyUp { scancode, .. } => {
                let key = match Key::from_scancode(scancode.unwrap()) {
                    Some(key) => key,
                    None => continue,
                };

                key_states[key as usize] = false;
            },

            Event::MouseMotion { xrel, yrel, .. } => {
                let yaw = xrel as f32 * cfg.mouse_sensitivity;
                let pitch = yrel as f32 * cfg.mouse_sensitivity;

                camera.rotate(yaw, pitch);

                // setting the mouse to the center
                sdl_ctx.mouse().warp_mouse_in_window(camera.get_window(), center_x, center_y);
            },

            Event::MouseWheel { y, .. } => {
                if key_states[Key::Ctrl] {
                    let zoom = (y as f32) * cfg.zoom_sensitivity;
                    camera.zoom(zoom);
                }
                else {
                    let roll = (y as f32) * cfg.scroll_sensitivity;
                    camera.roll(roll);
                }
            },

            Event::MouseButtonDown {mouse_btn, .. } => {
                //clicks tells you how many clicks it was. Ex: 1 for single click, 2 for double click, etc.
                if let Some(key) = Key::from_mouse(mouse_btn) {
                    key_states[key as usize] = true;
                }
                // atm nothing is done to know the position of where mouse was clicked, because its always in the center
            },

            Event::MouseButtonUp {mouse_btn, .. } => {
                if let Some(key) = Key::from_mouse(mouse_btn) {
                    key_states[key as usize] = false;
                }
            },

            _ => {}
        }
    }

    let mov_x = (key_states[Key::A] as i32 - key_states[Key::D] as i32) as f32;
    let mov_y = (key_states[Key::Space] as i32 - key_states[Key::Shift] as i32) as f32;
    let mov_z = (key_states[Key::W] as i32 - key_states[Key::S] as i32) as f32;
    let mov = Vec3::new(mov_x, mov_y, mov_z).normalize().scale(dt * cfg.camera_speed);

    camera.move_rel_to_facing(mov);

    false
}



//
fn get_cube_scene() -> Scene {
    let cube_vertices = [
        Vertex { pos: Vec3::new(-1.0, -1.0, -1.0) },
        Vertex { pos: Vec3::new(1.0, -1.0, -1.0) },
        Vertex { pos: Vec3::new(1.0, 1.0, -1.0) },
        Vertex { pos: Vec3::new(-1.0, 1.0, -1.0) },
        Vertex { pos: Vec3::new(-1.0, -1.0, 1.0) },
        Vertex { pos: Vec3::new(1.0, -1.0, 1.0) },
        Vertex { pos: Vec3::new(1.0, 1.0, 1.0) },
        Vertex { pos: Vec3::new(-1.0, 1.0, 1.0) },
    ];

    let triangles = [
        Triangle { a: cube_vertices[0], b: cube_vertices[1], c: cube_vertices[2] },
        Triangle { a: cube_vertices[0], b: cube_vertices[2], c: cube_vertices[3] },
        Triangle { a: cube_vertices[4], b: cube_vertices[5], c: cube_vertices[6] },
        Triangle { a: cube_vertices[4], b: cube_vertices[6], c: cube_vertices[7] },
        Triangle { a: cube_vertices[0], b: cube_vertices[1], c: cube_vertices[5] },
        Triangle { a: cube_vertices[0], b: cube_vertices[5], c: cube_vertices[4] },
        Triangle { a: cube_vertices[2], b: cube_vertices[3], c: cube_vertices[7] },
        Triangle { a: cube_vertices[2], b: cube_vertices[7], c: cube_vertices[6] },
        Triangle { a: cube_vertices[1], b: cube_vertices[2], c: cube_vertices[6] },
        Triangle { a: cube_vertices[1], b: cube_vertices[6], c: cube_vertices[5] },
        Triangle { a: cube_vertices[3], b: cube_vertices[0], c: cube_vertices[4] },
        Triangle { a: cube_vertices[3], b: cube_vertices[4], c: cube_vertices[7] },
    ];

    let cube = Object {
        transform: Transform::new(Vec3::Z_AXIS.scale(5.0), Quat::IDENTITY, Vec3::IDENTITY),
        mesh: Mesh { primitives: triangles.iter().map(|t| Primitive::Triangle(*t)).collect() },
    };

    let scene = Scene { objects: vec![cube] };
    scene
}


/// Initializes the SDL context, creates a window to which a camera outputs to
fn main() -> Result<(), String> {
    // initial states and setup constants
    const SCREEN_WIDTH_PIX: u32 = 128;
    const SCREEN_HEIGHT_PIX: u32 = 72;
    const PIXELS_PER_UNIT: f32 = 200.0;
    const PIXEL_SIZE: u8 = 10;
    let target_fps: f32 = 30.0;

    let camera_pos = Vec3::ZERO;
    let camera_rot = Quat::IDENTITY;
    let fov: f32 = 90.0;    // in degrees

    let mouse_sensitivity: f32 = 0.001;
    let scroll_sensitivity: f32 = 0.1;
    let zoom_sensitivity: f32 = 2.0;
    let camera_speed: f32 = 2.0;

    let scene = get_cube_scene();

    // Setup and Rendering loop
    let config = Settings { mouse_sensitivity, scroll_sensitivity, zoom_sensitivity, camera_speed };

    let mut sdl_ctx: sdl2::Sdl = sdl2::init()?;
    let screen = Screen::new(&mut sdl_ctx, SCREEN_WIDTH_PIX, SCREEN_HEIGHT_PIX, PIXEL_SIZE, PIXELS_PER_UNIT, "3D Renderer")?;
    let cam_transform = Transform::new(camera_pos, camera_rot, Vec3::IDENTITY);
    let mut camera = Camera::new(screen, scene, cam_transform, fov);

    let mut key_states: Keys = [false; 256];

    let target_dt = Duration::from_secs_f64(1.0 / target_fps as f64);
    let mut next_frame = Instant::now();
    let mut dt = target_dt.as_secs_f32();
    loop {
        let frame_start = Instant::now();

        // rendering
        camera.draw_frame();

        // input
        let stop = user_inputs(&mut sdl_ctx, &config, &mut camera, &mut key_states, dt);
        if stop { break; }

        next_frame += target_dt;

        let now = Instant::now();
        if next_frame > now {
            thread::sleep(next_frame - now);
        }
        else {    // frame took too long, resync
            next_frame = now;
        }

        dt = frame_start.elapsed().as_secs_f32();
        println!("FPS: {:.1}", 1.0 / dt);
        //println!("Camera Pos: ({:.2}, {:.2}, {:.2})", camera.transform.pos.x, camera.transform.pos.y, camera.transform.pos.z);
        let (p,y,r) = camera.transform.rot.to_euler();
        //println!("Camera Rot: pitch: {:.2}, yaw: {:.2}, roll: {:.2}", p.to_degrees(), y.to_degrees(), r.to_degrees());
        //let fl = camera.get_focal_length();
        //println!("Camera FOV: {:.2}, focal_length {:.2}", camera.fov, fl);
        //println!("-----------------------------")
    }
    Ok(())
}

