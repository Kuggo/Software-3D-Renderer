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
mod shader;
mod shaders;

pub use crate::camera::{Camera, Screen};
use crate::geometry::{Mesh, Object, Primitive, Scene, Transform};
use crate::renderer::{CullMode, DepthTest, InterpMode, RenderMode, Renderer};
use crate::shader::{Material};
use crate::shaders::{ColorShader, PhongShader};

/// ControlSettings holds the various sensitivity and speed settings for the controls.
struct ControlSettings {
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
/// Events are already frame dependent, so dt should not be for most events.
/// However, for movement, dt is used to make movement frame rate independent.
fn user_inputs(sdl_ctx: &mut sdl2::Sdl, cfg: &ControlSettings, camera: &mut Camera,
               screen: &Screen, key_states: &mut Keys, dt: f32
) -> bool {
    let (center_x, center_y) = screen.get_screen_center_pix();

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
                let delta_yaw = xrel as f32 * cfg.mouse_sensitivity;
                let delta_pitch = yrel as f32 * cfg.mouse_sensitivity;

                camera.yaw += delta_yaw;
                camera.pitch = (camera.pitch + delta_pitch).clamp(-89.0, 89.0);

                // setting the mouse to the center
                sdl_ctx.mouse().warp_mouse_in_window(screen.get_window(), center_x as i32, center_y as i32);
            },

            Event::MouseWheel { y, .. } => {
                if key_states[Key::Ctrl] {
                    let zoom = (y as f32) * cfg.zoom_sensitivity;
                    camera.zoom(zoom);
                }
                else {
                    let roll = (y as f32) * cfg.scroll_sensitivity;
                    camera.roll += roll;
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
    let mov = Vec3::new(mov_x, mov_y, mov_z).normalize() * (dt * cfg.camera_speed);

    camera.move_rel_to_facing(mov);
    camera.update_transform();

    false
}



/// Creates a simple scene with a cube in the center.
/// The cube is made up of 12 triangles (2 for each face).
fn get_cube_scene<'a>() -> Scene<'a> {
    let triangles: &[[u32;3]] = &[
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 1, 5],
        [0, 5, 4],
        [2, 3, 7],
        [2, 7, 6],
        [1, 2, 6],
        [1, 6, 5],
        [3, 0, 4],
        [3, 4, 7],
    ];

    let positions = vec![
        Vec3::new(-1.0, -1.0, -1.0),
        Vec3::new(1.0, -1.0, -1.0),
        Vec3::new(1.0, 1.0, -1.0),
        Vec3::new(-1.0, 1.0, -1.0),
        Vec3::new(-1.0, -1.0, 1.0),
        Vec3::new(1.0, -1.0, 1.0),
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(-1.0, 1.0, 1.0),
    ];

    let colors = vec![
        Color::RED,
        Color::GREEN,
        Color::BLUE,
        Color::RED,
        Color::GREEN,
        Color::BLUE,
        Color::RED,
        Color::GREEN,
    ];

    let cube = Object {
        transform: Transform::new(Vec3::Z_AXIS * 2.5, Quat::from_axis_angle(Vec3::Y_AXIS, 35f32.to_radians()), Vec3::IDENTITY),
        mesh: Mesh {
            positions,
            colors: Some(colors),
            normals: None,
            uvs: None,
            primitives: triangles.iter().map(|&[a,b,c]| Primitive::Triangle(a, b, c)).collect()
        },
        material: &Material { shader: &ColorShader },
    };

    let scene = Scene { objects: vec![cube] };
    scene
}


fn get_tri_scene<'a>() -> Scene<'a> {
    let triangles: &[[u32;3]] = &[
        [0, 2, 1],
    ];

    let positions = vec![
        Vec3::new(-1.0, -1.0, -1.0),
        Vec3::new(1.0, -1.0, -1.0),
        Vec3::new(1.0, 1.0, -1.0),
    ];

    let colors = vec![
        Color::RED,
        Color::GREEN,
        Color::BLUE,
    ];

    let cube = Object {
        transform: Transform::new(Vec3::Z_AXIS * 2.5, Quat::from_axis_angle(Vec3::Y_AXIS, 35f32.to_radians()), Vec3::IDENTITY),
        mesh: Mesh {
            positions,
            colors: Some(colors),
            normals: None,
            uvs: None,
            primitives: triangles.iter().map(|&[a,b,c]| Primitive::Triangle(a, b, c)).collect()
        },
        material: &Material { shader: &ColorShader },
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
    const PIXEL_SIZE: u32 = 20;
    let target_fps: f32 = 30.0;

    let camera_pos = Vec3::ZERO;
    let fov: f32 = 90.0;    // in degrees

    let mouse_sensitivity: f32 = 0.05;
    let scroll_sensitivity: f32 = 5.0;
    let zoom_sensitivity: f32 = 2.0;
    let camera_speed: f32 = 2.0;

    let scene = get_cube_scene();

    // Setup and Rendering loop
    let config = ControlSettings { mouse_sensitivity, scroll_sensitivity, zoom_sensitivity, camera_speed };

    let mut sdl_ctx: sdl2::Sdl = sdl2::init()?;
    let mut screen = &mut Screen::new(&mut sdl_ctx, SCREEN_WIDTH_PIX, SCREEN_HEIGHT_PIX, PIXEL_SIZE, PIXELS_PER_UNIT, "3D Renderer")?;
    let cam_transform = Transform::new(camera_pos, Quat::IDENTITY, Vec3::IDENTITY);
    let mut camera = Camera::new(scene, cam_transform, fov);
    let mut renderer = Renderer::new(&screen, InterpMode::DepthCorrect, CullMode::Backface, RenderMode::Solid, DepthTest::Less);

    let mut key_states: Keys = [false; 256];

    let target_dt = Duration::from_secs_f64(1.0 / target_fps as f64);
    let mut next_frame = Instant::now();
    let mut last_print = next_frame;
    let mut dt = target_dt.as_secs_f32();
    loop {
        let frame_start = Instant::now();

        // rendering
        screen = camera.draw_frame_to_screen(screen, &mut renderer);

        // input
        let stop = user_inputs(&mut sdl_ctx, &config, &mut camera, &screen, &mut key_states, dt);
        if stop { break; }

        next_frame += target_dt;

        let now = Instant::now();
        if next_frame > now {
            thread::sleep(next_frame - now);
        }
        else {    // frame took too long, resync
            next_frame = now;
        }

        if last_print.elapsed().as_secs_f32() >= 1.0 {
            last_print = now;
            dt = frame_start.elapsed().as_secs_f32();
            println!("FPS: {:.1}", 1.0 / dt);
        }
    }
    Ok(())
}

