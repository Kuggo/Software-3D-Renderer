use std::ops;
use crate::utils::FP_TOLERANCE;

#[derive(Debug, Clone, Copy)]
pub enum TextureFilter {
    Nearest,
    Bilinear,
}

#[derive(Debug, Clone, Copy)]
pub enum TextureWrap {
    Repeat,
    ClampToEdge,
}


/// Represent a texture, which can be used in shaders to sample colors or other attributes based on UV coordinates.
#[derive(Debug, Clone)]
pub struct Texture<T> {
    pub data: Vec<T>,
    pub width: u32,
    pub height: u32,
    pub filter: TextureFilter,
    pub wrap: TextureWrap,
}
impl<T> Texture<T>
where T: Copy + ops::Add<T, Output = T> + ops::Mul<f32, Output = T>, {
    pub fn new(data: Vec<T>, width: u32, height: u32, filter: TextureFilter, wrap: TextureWrap) -> Self {
        assert_eq!(data.len(), (width * height) as usize, "Texture data length does not match width * height");
        Self { data, width, height, filter, wrap }
    }

    pub fn sample(&self, u: f32, v: f32) -> T {
        const MAX: f32 = 1.0 - FP_TOLERANCE;

        let (u, v) = (u * MAX, v * MAX);    // attempt to map [0, 1] -> [0, MAX] ~ [0, 1)

        let (u, v) = match self.wrap {
            TextureWrap::Repeat => (u.fract(), v.fract()),
            TextureWrap::ClampToEdge => (u.clamp(0.0, MAX), v.clamp(0.0, MAX)),
        };

        match self.filter {
            TextureFilter::Nearest => {
                let x = (u * self.width as f32) as u32;
                let y = (v * self.height as f32) as u32;
                self.data[(y * self.width + x) as usize]
            },
            TextureFilter::Bilinear => {
                let x_float = u * self.width as f32 - 0.5;
                let y_float = v * self.height as f32 - 0.5;
                let x = x_float.floor() as u32;
                let y = y_float.floor() as u32;

                let c00 = self.data[( y    * self.width + x  ) as usize];
                let c10 = self.data[( y    * self.width + x+1) as usize];
                let c01 = self.data[((y+1) * self.width + x  ) as usize];
                let c11 = self.data[((y+1) * self.width + x+1) as usize];

                let sx = x_float - x as f32;
                let sy = y_float - y as f32;
                let c0 = c00 * (1.0 - sx) + c10 * sx;
                let c1 = c01 * (1.0 - sx) + c11 * sx;
                c0 * (1.0 - sy) + c1 * sy
            },
        }
    }
}

