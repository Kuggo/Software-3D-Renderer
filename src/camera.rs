use sdl2::render::{Texture, WindowCanvas};
use sdl2::video::{Window};
use sdl2::pixels::PixelFormatEnum;
use crate::utils::*;
use crate::renderer::Renderer;
use crate::geometry::{Scene, Transform};

pub struct Screen {
    pub width_pix: u32,
    pub height_pix: u32,
    pixel_size: u8,
    pixels_per_unit: f32,

    stride: usize,
    framebuffer: Vec<u32>,

    canvas: WindowCanvas,
    texture: Texture<'static>,
}

impl Screen {
    pub fn new(sdl_ctx: &mut sdl2::Sdl, width_pix: u32, height_pix: u32, pixel_size: u8,
        pixels_per_unit: f32, title: &str,
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

        let framebuffer = vec![0; (width_pix * height_pix) as usize];

        Ok(Self {
            width_pix,
            height_pix,
            pixel_size,
            pixels_per_unit,
            stride: width_pix as usize,
            framebuffer,
            canvas,
            texture,
        })
    }

    pub fn show(&mut self) {
        self.texture.update(
            None,
            bytemuck::cast_slice(&self.framebuffer),
            self.stride * 4,
        ).unwrap();

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
        self.framebuffer.fill(0x00000000); // black
    }
    
    pub fn world_to_screen_coords(&self, coord: Vec2) -> (usize, usize) {
        let screen_x = (self.width_pix as i32 / 2) + (coord.x * self.pixels_per_unit) as i32;
        let screen_y = (self.height_pix as i32 / 2) - (coord.y * self.pixels_per_unit) as i32;
        (screen_x as usize, screen_y as usize)
    }

    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.width_pix as usize || y >= self.height_pix as usize {
            return;
        }

        self.framebuffer[y * self.stride + x] = color;
    }

    /*pub fn draw_hline(&mut self, x0: usize, x1: usize, y: usize, color: u32) {
        let (x0, x1) = self.world_to_screen_coords(x0, );
        let y = (y + (self.height_pix as i32 / 2)) as usize;
        let row = &mut self.framebuffer[y * self.stride .. (y+1) * self.stride];
        row[x0..x1].fill(color);
    }*/

}


pub struct Camera {
    pub screen: Screen,
    pub scene: Scene,
    pub transform: Transform,
    pub fov: f32,
}

impl Camera {
    pub fn new(screen: Screen, scene: Scene, transform: Transform, fov: f32) -> Self {
        let camera = Camera {
            screen,
            scene,
            transform,
            fov,
        };
        camera
    }

    pub fn get_focal_length(&mut self) -> f32 {
        let fov = self.fov.to_radians();
        let width = self.screen.width_pix as f32 / self.screen.pixels_per_unit as f32;
        let focal_length = (width / 2.0) / (fov / 2.0).tan();
        focal_length
    }

    pub fn zoom(&mut self, zoom: f32) {
        self.fov = (self.fov + zoom).clamp(30.0, 160.0);
    }

    pub fn get_window(&self) -> &Window {
        self.screen.canvas.window()
    }

    pub fn forward(&self) -> Vec3 {
        self.transform.rot.rotate_vec3(Vec3::NZ_AXIS)
    }

    pub fn up(&self) -> Vec3 {
        self.transform.rot.rotate_vec3(Vec3::Y_AXIS)
    }

    pub fn right(&self) -> Vec3 {
        self.transform.rot.rotate_vec3(Vec3::X_AXIS)
    }

    pub fn rotate(&mut self, yaw: f32, pitch: f32) {
        let yaw_q = Quat::from_axis_angle(Vec3::Y_AXIS, yaw);
        let pitch_q = Quat::from_axis_angle(self.right(), pitch);

        self.transform.rot = pitch_q.mul(&yaw_q).mul(&self.transform.rot).normalize();
    }

    pub fn roll(&mut self, angle: f32) {
        let forward = self.forward();
        let q = Quat::from_axis_angle(forward, angle);

        self.transform.rot = q.mul(&self.transform.rot).normalize();
    }

    pub fn move_rel_to_facing(&mut self, direction: Vec3) {
        let forward = self.transform.rot.rotate_vec3(Vec3::Z_AXIS);
        let flat_forward = Vec3::new(forward.x, 0.0, forward.z).normalize(); // project forward onto horizontal plane

        let right = flat_forward.cross(&Vec3::Y_AXIS);
        let horizontal = right.scale(direction.x).add( &flat_forward.scale(direction.z) );

        let vertical = Vec3::Y_AXIS.scale(direction.y);

        self.transform.pos = self.transform.pos.add( &horizontal.add(&vertical) );
    }

    pub fn world_to_camera(&self, p: Vec3) -> Vec3 {
        p.sub(&self.transform.pos)
    }

    pub fn draw_frame(&mut self) {
        self.screen.begin_frame();

        let mut renderer = Renderer::new();
        renderer.render_scene_from_camera(self);

        self.screen.show();
    }

}


