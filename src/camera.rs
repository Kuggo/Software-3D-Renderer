use sdl2::render::{Texture, WindowCanvas};
use sdl2::video::{Window};
use sdl2::pixels::PixelFormatEnum;
use crate::utils::*;
use crate::renderer::Renderer;
use crate::geometry::{Scene, Transform};


/// Screen is responsible to show drawn pixels into a sdl2 window every frame.
pub struct Screen {
    pub width_pix: u32,
    pub height_pix: u32,
    pixel_size: u32,
    pixels_per_unit: f32,

    framebuffer: Vec<u32>,

    canvas: WindowCanvas,
    texture: Texture<'static>,
}
impl Screen {
    /// Creates a new Screen with the given width and height in pixels, and the size of each pixel in screen space units.
    /// The pixels_per_unit parameter determines how many pixels correspond to one unit in world space, which is used for converting world coordinates to screen coordinates.
    /// The title parameter sets the title of the window.
    pub fn new(sdl_ctx: &mut sdl2::Sdl, width_pix: u32, height_pix: u32, pixel_size: u32,
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
            framebuffer,
            canvas,
            texture,
        })
    }

    /// Outputs the contents of the framebuffer to the window.  
    /// Should be called at the end of each frame
    pub fn show(&mut self) {
        self.texture.update(
            None,
            bytemuck::cast_slice(&self.framebuffer),
            self.width_pix as usize * 4 ,
        ).unwrap();

        self.canvas.copy(&self.texture, None, None).unwrap();
        self.canvas.present();
    }

    /// Returns a reference to the SDL2 window associated with this screen.
    pub fn get_window(&self) -> &Window {
        self.canvas.window()
    }

    /// Returns the width of the screen in world space units, based on the pixels_per_unit setting.
    pub fn get_width_units(&self) -> f32 {
        self.width_pix as f32 / self.pixels_per_unit
    }

    /// Returns the height of the screen in world space units, based on the pixels_per_unit setting.
    pub fn get_height_units(&self) -> f32 {
        self.height_pix as f32 / self.pixels_per_unit
    }

    /// Returns the coordinates of the center of the screen in pixel coordinates.
    pub fn get_screen_center_pix(&self) -> (u32, u32) {
        let x = self.width_pix * self.pixel_size / 2;
        let y = self.height_pix * self.pixel_size / 2;
        (x, y)
    }

    /// Checks if the given pixel coordinates are within the bounds of the screen.
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x < self.width_pix as i32 && x >= 0 && y < self.height_pix as i32 && y >= 0
    }

    pub fn begin_frame(&mut self, color: Color) {
        self.framebuffer.fill(color.to_argb()); // black
    }
    
    /// Converts world space coordinates to pixel coordinates on the screen.
    /// The center of the screen corresponds to (0, 0) in world space.
    pub fn world_to_screen_coords(&self, coord: Vec2) -> Pixel {
        let screen_x = (self.width_pix >>1) as f32 + coord.x * (self.pixels_per_unit-0.1);
        let screen_y = (self.height_pix>>1) as f32 - coord.y * (self.pixels_per_unit-0.1);
        Pixel::new( screen_x as i32, screen_y as i32, )
    }

    /// Draws a pixel at the given coordinates (if they are valid) with the specified color.
    pub fn draw_pixel(&mut self, pixel: Pixel, color: &Color) {
        if self.in_bounds(pixel.x, pixel.y) {
            let x = pixel.x as usize;
            let y = pixel.y as usize;
            self.framebuffer[y * self.width_pix as usize + x] = color.to_argb();
        }
    }

    pub fn fast_draw_pixel(&mut self, idx: usize, color: &Color) {
        self.framebuffer[idx] = color.to_argb();
    }
}


/// The Camera represents the viewer's perspective in the 3D scene. 
/// It holds the position and orientation of the camera, as well as the field of view (FOV) for perspective projection.
pub struct Camera<'a> {
    pub scene: Scene<'a>,
    pub transform: Transform,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    fov: f32,
}
impl<'a> Camera<'a> {
    /// Creates a new Camera for a scene, with some initial transform, and field of view (FOV).
    pub fn new(scene: Scene<'a>, transform: Transform, fov: f32) -> Self {
        let camera = Camera {
            scene,
            transform,
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
            fov,
        };
        camera
    }

    /// Calculates the focal length (projection variable) based on the camera's field of view (FOV)
    /// and the width of the screen in world space units.
    pub fn get_focal_length(&self, screen_width: f32) -> f32 {
        let fov = self.fov.to_radians();
        (screen_width / 2.0) / (fov / 2.0).tan()
    }

    /// Adjusts the camera's field of view (FOV) by a given zoom amount, within a reasonable range.
    pub fn zoom(&mut self, zoom: f32) {
        self.fov = (self.fov + zoom).clamp(30.0, 160.0);
    }

    /// Returns the forward direction vector of the camera, based on its current rotation.
    pub fn forward(&self) -> Vec3 {
        self.transform.rot.rotate_vec3(Vec3::NZ_AXIS)
    }

    /// Returns the up direction vector of the camera, based on its current rotation.
    pub fn up(&self) -> Vec3 {
        self.transform.rot.rotate_vec3(Vec3::Y_AXIS)
    }

    /// Returns the right direction vector of the camera, based on its current rotation.
    pub fn right(&self) -> Vec3 {
        self.transform.rot.rotate_vec3(Vec3::X_AXIS)
    }

    /// Updates the camera's transform rotation based on its current yaw, pitch, and roll angles.
    /// This operation should be done frequently to avoid accumulation of float errors in the rotation quaternion.
    pub fn update_transform(&mut self) {
        let yaw_q = Quat::from_axis_angle(Vec3::Y_AXIS, self.yaw.to_radians());

        let right = yaw_q.rotate_vec3(Vec3::X_AXIS);
        let pitch_q = Quat::from_axis_angle(right, self.pitch.to_radians());

        let quat = pitch_q.mul(&yaw_q);

        let forward = quat.rotate_vec3(Vec3::Z_AXIS);
        let roll_q = Quat::from_axis_angle(forward, self.roll.to_radians());

        self.transform.rot = roll_q.mul(&quat).normalize();
    }

    /// Moves the camera in a direction relative to where it's currently facing.
    /// The movement is FPS style: meaning that the camera moves in the horizontal plane.
    pub fn move_rel_to_facing(&mut self, direction: Vec3) {
        let forward = self.transform.rot.rotate_vec3(Vec3::Z_AXIS);
        let flat_forward = Vec3::new(forward.x, 0.0, forward.z).normalize(); // project forward onto horizontal plane

        let right = flat_forward.cross(&Vec3::Y_AXIS);
        let horizontal = right * direction.x + flat_forward * direction.z;

        let vertical = Vec3::Y_AXIS * direction.y;

        self.transform.pos = self.transform.pos + horizontal + vertical;
    }

    /// Renders the current scene from the camera's perspective onto the given screen.
    /// Renderer is provided by the caller so its internal buffers can be reused across frames.
    pub fn draw_frame_to_screen<'b>(&self, screen: &'b mut Screen, renderer: &mut Renderer) -> &'b mut Screen {
        screen.begin_frame(Color::BLACK);
        renderer.render_scene_from_camera(self, screen);
        screen.show();
        screen
    }

}
