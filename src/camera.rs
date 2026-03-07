
use sdl2::render::{Texture, WindowCanvas};
use sdl2::video::{Window};
use std::f32::consts::PI;
use sdl2::pixels::PixelFormatEnum;
use crate::utils::*;

pub struct Screen {
    pub width_pix: u32,
    pub height_pix: u32,
    pixel_size: u8,

    pixels: Option<&'static mut [u32]>, // temporarily stores pixels during frame
    stride: usize,

    canvas: WindowCanvas,
    texture: Texture<'static>,
}

impl Screen {
    pub fn new(
        sdl_ctx: &mut sdl2::Sdl,
        width_pix: u32,
        height_pix: u32,
        pixel_size: u8,
        title: &str,
    ) -> Result<Self, String> {
        let video = sdl_ctx.video()?;

        let window = video
            .window(title, width_pix * pixel_size as u32, height_pix * pixel_size as u32)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())?;

        canvas.set_scale(pixel_size as f32, pixel_size as f32)?;

        sdl_ctx.mouse().show_cursor(false);

        // leak TextureCreator to make 'static
        let texture_creator = Box::leak(Box::new(canvas.texture_creator()));

        // create texture once
        let texture = texture_creator
            .create_texture_streaming(
                PixelFormatEnum::ARGB8888,
                width_pix,
                height_pix,
            )
            .map_err(|e| e.to_string())?;

        Ok(Self {
            width_pix,
            height_pix,
            pixel_size,
            pixels: None,
            stride: 0,
            canvas,
            texture,
        })
    }

    pub fn show(&mut self) {
        self.pixels = None; // release the slice
        self.canvas.copy(&self.texture, None, None).unwrap();
        self.canvas.present();
    }

    pub fn get_screen_center_pix(&self) -> (i32, i32) {
        let x = self.width_pix as i32 * self.pixel_size as i32 / 2;
        let y = self.height_pix as i32 * self.pixel_size as i32 / 2;
        (x, y)
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        -(self.width_pix as i32) / 2 <= x && x < (self.width_pix / 2) as i32 &&
            -(self.height_pix as i32) / 2 <= y && y < (self.height_pix / 2) as i32
    }

    pub fn begin_frame(&mut self) {
        let texture = &mut self.texture;
        texture.with_lock(None, |bytes, pitch| {
            let pixels: &mut [u32] = bytemuck::cast_slice_mut(bytes);
            self.stride = pitch / 4;

            // We temporarily store pixels in the struct
            // 'static because texture memory lives until unlock
            self.pixels = Some(unsafe { std::mem::transmute::<&mut [u32], &'static mut [u32]>(pixels) });
        }).unwrap();
    }

    pub fn draw_pixel(&mut self, x: i32, y: i32, color: u32) {
        let pixels = self.pixels.as_mut().unwrap();
        let x = (x + (self.width_pix as i32 / 2)) as usize;
        let y = (y + (self.height_pix as i32 / 2)) as usize;
        pixels[y * self.stride + x] = color;
    }

    pub fn draw_hline(&mut self, x0: i32, x1: i32, y: i32, color: u32) {
        let pixels = self.pixels.as_mut().unwrap();
        let x1 = (x1 + (self.width_pix as i32 / 2)) as usize;
        let x0 = (x0 + (self.width_pix as i32 / 2)) as usize;
        let y = (y + (self.height_pix as i32 / 2)) as usize;
        pixels[y * self.stride + x0..y * self.stride + x1].fill(color);
    }

}


pub struct Camera {
    pub screen: Screen,
    pixels_per_unit: u32,
    pub position: Vec3,
    lookat_direction: Vec3,
    up_vector: Vec3,
    fov: f32,
    focal_length: f32,
}

impl Camera {
    pub fn new(screen: Screen, position: Vec3, direction: Vec3, up_vector: Vec3, fov: f32, pixels_per_unit: u32) -> Self {
        let pixels_per_unit = pixels_per_unit;
        let front_direction = direction.normalize();
        let up_vector = up_vector.normalize();
        let mut camera = Camera {
            screen,
            pixels_per_unit,
            position,
            lookat_direction: front_direction,
            up_vector,
            fov: 1.0,
            focal_length: 0.0,
        };
        camera.set_fov(fov);
        camera
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov.to_radians();
        let width = self.screen.width_pix as f32 / self.pixels_per_unit as f32;
        self.focal_length = (width / 2.0) / (fov / 2.0).tan();
    }

    pub fn zoom(&mut self, zoom: f32) {
        let fov = (self.fov + zoom).clamp(30f32.to_radians(), 160f32.to_radians());
        self.set_fov(fov);
    }

    pub fn get_window(&self) -> &Window {
        self.screen.canvas.window()
    }

    pub fn get_direction(&self) -> Vec3 {
        self.lookat_direction
    }

    pub fn rotate_yaw(&mut self, angle: f32) {
        let yaw = angle / self.pixels_per_unit as f32;
        self.lookat_direction = self.lookat_direction.rotate_xz(-yaw);
        self.up_vector = self.up_vector.rotate_xz(-yaw);
    }

    pub fn rotate_pitch(&mut self, angle: f32) {
        let pitch = angle / self.pixels_per_unit as f32;
        let right_vec = self.lookat_direction.cross(&self.up_vector);
        self.lookat_direction = self.lookat_direction.rotate_around(&right_vec, pitch);
        self.up_vector = self.up_vector.rotate_around(&right_vec, pitch);
    }

    pub fn rotate_roll(&mut self, angle: f32) {
        self.up_vector = self.up_vector.rotate_around(&self.lookat_direction, angle);
    }

    pub fn move_rel_to_facing(&mut self, direction: Vec3) {
        let up_component = Vec3::new(0.0, direction.y, 0.0);
        let hori_component = direction.sub(&up_component);

        let sign = (((self.lookat_direction.x >= 0.0) as i32) * 2) - 1;
        let reference = Vec3::new(self.lookat_direction.x, 0.0, self.lookat_direction.z);
        let angle = sign as f32 * Vec3::Z_AXIS.angle(&reference);

        let hori_dir = hori_component.rotate_xz(angle);
        let mov_dir = hori_dir.add(&up_component);

        self.position = self.position.add(&mov_dir);
    }

    pub fn world_to_camera(&self, p: Vec3) -> Vec3 {
        p.sub(&self.position)
    }

    pub fn draw_frame(&mut self) {
        self.screen.begin_frame();

        for x in (-(self.screen.width_pix as i32) / 2)..(self.screen.width_pix as i32 / 2) {
            for y in (-(self.screen.height_pix as i32) / 2)..(self.screen.height_pix as i32 / 2) {
                let color = if (x+y) % 4 == 0 { Color::RED } else { Color::BLUE };
                self.screen.draw_pixel(x, y, color.to_argb());
            }
        }

        self.screen.draw_hline(-10, 10, 0, Color::GREEN.to_argb());

        self.screen.show();
    }
}


