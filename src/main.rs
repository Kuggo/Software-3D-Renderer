
use std::ops::Index;
use std::thread;
use std::time::{Duration, Instant};

use sdl2::keyboard::Scancode;
use sdl2::event::Event;
use sdl2::mouse::MouseButton;

mod utils;
use crate::utils::*;
mod camera;
pub use crate::camera::{Camera, Screen};


// Aliases
type Keys = [bool; 256];
impl Index<Key> for Keys {
    type Output = bool;

    fn index(&self, key: Key) -> &Self::Output {
        match key {
            _ => &self[key as usize],
        }
    }
}


// Data clumps
struct Settings {
    mouse_sensitivity: f32,
    scroll_sensitivity: f32,
    zoom_sensitivity: f32,
    camera_speed: f32,
}



enum Key {
    W,
    A,
    S,
    D,

    /*Up,
    Down,
    Left,
    Right, */

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

    fn from_mouse(mouse_btn: sdl2::mouse::MouseButton) -> Option<Key> {
        match mouse_btn {
            MouseButton::Left => Some(Key::MouseLeft),
            MouseButton::Right => Some(Key::MouseRight),
            MouseButton::Middle => Some(Key::MouseMiddle),
            _ => None,
        }
    }
}


//Globals





// functions

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
                camera.rotate_yaw(yaw);

                let pitch = -yrel as f32 * cfg.mouse_sensitivity;
                camera.rotate_pitch(pitch);

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
                    camera.rotate_roll(roll);
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

fn main() -> Result<(), String> {
    const SCREEN_WIDTH_PIX: u32 = 64;
    const SCREEN_HEIGHT_PIX: u32 = 36;
    const PIXELS_PER_UNIT: u32 = 100;
    const PIXEL_SIZE: u8 = 40;
    let target_fps: f32 = 3.0;

    let camera_pos = Vec3::ZERO;
    let camera_dir = Vec3::NZ_AXIS;
    let camera_up = Vec3::Y_AXIS;
    let fov: f32 = 90.0;    // in degrees

    let mouse_sensitivity: f32 = 0.2;
    let scroll_sensitivity: f32 = 0.1;
    let zoom_sensitivity: f32 = 0.1;
    let camera_speed: f32 = 10.0;

    let mut sdl_ctx: sdl2::Sdl = sdl2::init()?;
    let screen = Screen::new(&mut sdl_ctx, SCREEN_WIDTH_PIX, SCREEN_HEIGHT_PIX, PIXEL_SIZE, "3D Renderer")?;
    let mut camera = Camera::new(screen, camera_pos, camera_dir, camera_up, fov, PIXELS_PER_UNIT);

    let mut key_states: Keys = [false; 256];

    let config = Settings {
        mouse_sensitivity,
        scroll_sensitivity,
        zoom_sensitivity,
        camera_speed
    };

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
        println!("FPS: {:.2}", 1.0 / dt);
        println!("Camera position: ({:.2}, {:.2}, {:.2})", camera.position.x, camera.position.y, camera.position.z);
    }
    Ok(())
}

