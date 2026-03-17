use std::cmp::PartialEq;
use std::ops;
use std::ops::Sub;
use crate::geometry::lerp;

pub const FP_TOLERANCE: f32 = 1e-6;
pub fn fp_equals(a: f32, b: f32) -> bool {
    (a - b).abs() < FP_TOLERANCE
}

#[derive(Debug, Copy, PartialEq, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
impl Vec2 {
    pub const ZERO: Vec2 = Vec2 {x: 0.0, y: 0.0};
    pub const IDENTITY: Vec2 = Vec2 {x: 1.0, y: 1.0};
    pub const X_AXIS: Vec2 = Vec2 {x: 1.0, y: 0.0};
    pub const Y_AXIS: Vec2 = Vec2 {x: 0.0, y: 1.0};
    pub const NX_AXIS: Vec2 = Vec2 {x: -1.0, y: 0.0};
    pub const NY_AXIS: Vec2 = Vec2 {x: 0.0, y: -1.0};

    /// Create a new Vec2 with the given x and y components.
    pub fn new(x: f32, y: f32) -> Self {
        Self {x, y}
    }

    /// Create a Vec2 from polar coordinates (length and angle in radians).
    pub fn from_polar_coords(len: f32, angle: f32) -> Self {
        Self::new(len * angle.cos(), len * angle.sin())
    }
    
    /// Checks if this Vec2 is approximately equal to another Vec2, 
    /// within a small tolerance to account for floating-point precision issues.  
    /// This tolerance is applied component-wise.
    pub fn equals_fp(&self, other: &Vec2) -> bool {
        (self.x - other.x).abs() < FP_TOLERANCE &&
            (self.y - other.y).abs() < FP_TOLERANCE
    }

    /// Calculate the dot product of this Vec2 with another Vec2.
    pub fn dot(&self, other: &Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }
    
    /// Calculate the 2D cross product (also known as the perp dot product) of this Vec2 with another Vec2.
    pub fn cross(&self, other: &Vec2) -> f32 {
        self.x * other.y - self.y * other.x
    }
    
    /// Add this Vec2 to another Vec2, returning the resulting Vec2.
    pub fn add(&self, other: &Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }

    /// Subtract another Vec2 from this Vec2, returning the resulting Vec2.
    pub fn sub(&self, other: &Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }

    /// Scale this Vec2 by a scalar value, returning the resulting Vec2.
    pub fn scale(&self, scalar: f32) -> Vec2 {
        Vec2::new(self.x * scalar, self.y * scalar)
    }

    /// Returns the Hadamard product (component-wise multiplication) of this Vec2 with another Vec2.
    pub fn scale_vec(&self, other: &Vec2) -> Vec2 {
        Vec2::new(
            self.x * other.x,
            self.y * other.y,
        )
    }
    
    /// Calculate the length (magnitude) of this Vec2.
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Calculate the Manhattan distance between this Vec2 and another Vec2.
    pub fn manhattan(&self, other: &Vec2) -> f32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
    
    /// Normalize this Vec2 to have a length of 1, returning the resulting Vec2.
    /// If the length of the Vec2 is 0, returns a zero vector to avoid division by zero.
    pub fn normalize(&self) -> Vec2 {
        let l = self.length();
        if l == 0.0 {
            return Vec2::new(0.0, 0.0);
        }
        Vec2::new(self.x / l, self.y / l)
    }

    /// Linearly interpolate between this Vec2 and another Vec2 by a factor of t.  
    /// t should be in the range [0.0, 1.0].
    pub fn lerp(a: &Vec2, b: &Vec2, t: f32) -> Vec2 {
        *a * (1.0 - t) + *b * t
    }
}

impl ops::Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}
impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}
impl ops::Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, scalar: f32) -> Vec2 {
        Vec2::new(self.x * scalar, self.y * scalar)
    }
}


#[derive(Debug, Copy, PartialEq, Clone)]
pub struct Pixel {
    pub x: i32,
    pub y: i32,
}
impl Pixel {
    pub const ZERO: Pixel = Pixel {x: 0, y: 0};
    pub const IDENTITY: Pixel = Pixel {x: 1, y: 1};

    /// Create a new Pixel with the given x and y coordinates.
    pub fn new(x: i32, y: i32) -> Self {
        Self {x, y}
    }

    pub fn from_vec2(vec: &Vec2) -> Pixel {
        Pixel::new(vec.x.round() as i32, vec.y.round() as i32)
    }

    /// Add this Pixel to another Pixel, returning the resulting Pixel.
    pub fn add(&self, other: &Pixel) -> Pixel {
        Pixel::new(self.x + other.x, self.y + other.y)
    }

     /// Subtract another Pixel from this Pixel, returning the resulting Pixel.
    pub fn sub(&self, other: &Pixel) -> Pixel {
        Pixel::new(self.x - other.x, self.y - other.y)
    }

    /// Scale this Pixel by a scalar value, returning the resulting Pixel.
    pub fn scale(&self, scalar: f32) -> Pixel {
        Pixel::new((self.x as f32 * scalar).round() as i32, (self.y as f32 * scalar).round() as i32)
    }

    /// Returns the Hadamard product (component-wise multiplication) of this Pixel with a Vec2.
    pub fn scale_vec(&self, vec2: &Vec2) -> Pixel {
        Pixel::new((self.x as f32 * vec2.x).round() as i32, (self.y as f32 * vec2.y).round() as i32)
    }

    /// Calculate the Manhattan distance between this Pixel and another Pixel.
    pub fn manhattan(&self, other: &Pixel) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// Linearly interpolate between this Pixel and another Pixel by a factor of t.
    pub fn lerp(a: &Pixel, b: &Pixel, t: f32) -> Pixel {
        Pixel {
            x: (a.x as f32 + (b.x as f32 - a.x as f32) * t).round() as i32,
            y: (a.y as f32 + (b.y as f32 - a.y as f32) * t).round() as i32,
        }
    }

}


#[derive(Debug, Copy, PartialEq, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vec3 {
    pub const ZERO: Vec3 = Vec3 {x: 0.0, y: 0.0, z: 0.0};
    pub const IDENTITY: Vec3 = Vec3 {x: 1.0, y: 1.0, z: 1.0};
    pub const X_AXIS: Vec3 = Vec3 {x: 1.0, y: 0.0, z: 0.0};
    pub const Y_AXIS: Vec3 = Vec3 {x: 0.0, y: 1.0, z: 0.0};
    pub const Z_AXIS: Vec3 = Vec3 {x: 0.0, y: 0.0, z: 1.0};
    pub const NX_AXIS: Vec3 = Vec3 {x: -1.0, y: 0.0, z: 0.0};
    pub const NY_AXIS: Vec3 = Vec3 {x: 0.0, y: -1.0, z: 0.0};
    pub const NZ_AXIS: Vec3 = Vec3 {x: 0.0, y: 0.0, z: -1.0};

    /// Create a new Vec3 with the given x, y, and z components.
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 {x, y, z }
    }

    /// Create a Vec3 from polar coordinates (length, pitch, and yaw in radians).
    pub fn from_polar_coords(len: f32, pitch: f32, yaw: f32) -> Vec3 {
        Vec3::new(
            len * pitch.cos() * yaw.cos(),
            len * pitch.sin(),
            len * pitch.cos() * yaw.sin())
    }
    
    /// Checks if this Vec3 is approximately equal to another Vec3, 
    /// within a small tolerance to account for floating-point precision issues.  
    /// This tolerance is applied component-wise.
    pub fn equals_fp(&self, other: &Vec3) -> bool {
        (self.x - other.x).abs() < FP_TOLERANCE &&
            (self.y - other.y).abs() < FP_TOLERANCE &&
            (self.z - other.z).abs() < FP_TOLERANCE

    }

    /// Calculate the dot product of this Vec3 with another Vec3.
    pub fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Calculate the cross product of this Vec3 with another Vec3, returning the resulting Vec3.
    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x)
    }

    /// Returns the Hadamard product (component-wise multiplication) of this Vec3 with another Vec3.
    pub fn scale_vec(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.x * other.x,
            self.y * other.y,
            self.z * other.z,
        )
    }

    /// Calculate the length (magnitude) of this Vec3.
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Calculate the Manhattan distance between this Vec3 and another Vec3.
    pub fn manhattan(&self, other: &Vec3) -> f32 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }

    /// Normalize this Vec3 to have a length of 1, returning the resulting Vec3.
    /// If the length of the Vec3 is 0, returns a zero vector to avoid division by zero.
    pub fn normalize(&self) -> Vec3 {
        let l = self.length();
        if l == 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }
        Vec3::new(self.x / l, self.y / l, self.z / l)
    }

    /// Linearly interpolate between this Vec3 and another Vec3 by a factor of t.  
    /// t should be in the range [0.0, 1.0].
    pub fn lerp(a: &Vec3, b: &Vec3, t: f32) -> Vec3 {
        Vec3 {
            x: lerp(a.x, b.x, t),
            y: lerp(a.y, b.y, t),
            z: lerp(a.z, b.z, t),
        }
    }

    /// Convert this Vec3 to polar coordinates (length, pitch, and yaw in radians).
    pub fn polar(&self) -> (f32, f32, f32) {
        let len = self.length();
        if len == 0.0 {
            return (0.0, 0.0, 0.0);
        }
        let pitch = (self.y / len).asin();
        let yaw = self.y.atan2(self.x);
        (len, pitch, yaw)
    }

    /// Check if this Vec3 is collinear with another Vec3, meaning they lie on the same line.
    /// If one of them is a zero vector it returns true.
    pub fn collinear(&self, other: &Vec3) -> bool {
        let l1 = self.length();
        let l2 = other.length();
        if l1 == 0.0 || l2 == 0.0 {
            return true;
        }
        let n1 = *self * (1.0 / l1);
        let n2 = *other * (1.0 / l2);
        n1.equals_fp(&n2)
    }

    /// Calculate the angle in radians between this Vec3 and another Vec3.
    pub fn angle(&self, other: &Vec3) -> f32 {
        if *self == Self::ZERO || *other == Self::ZERO {
            return 0.0;
        }
        (self.dot(&other) / (self.length() * other.length())).acos()
    }

    /// Project this Vec3 onto another Vec3, returning the resulting Vec3.
    pub fn project_onto(&self, other: &Vec3) -> Vec3 {
        let l = other.length();
        if l == 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }
        *other * (self.dot(other) / l)
    }

    /// Returns the component of this Vec3 that is orthogonal (perpendicular) to another Vec3.
    pub fn orthogonal_component(&self, other: &Vec3) -> Vec3 {
        *self - self.project_onto(&other)
    }
}
impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}
impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}
impl ops::Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, scalar: f32) -> Vec3 {
        Vec3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
    
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Quat {
    pub cos_a2: f32,
    pub axis_sin_a2: Vec3,
}
impl Quat {
    pub const IDENTITY: Quat = Quat { cos_a2: 1.0, axis_sin_a2: Vec3::ZERO };

    /// A quaternion can be constructed from an axis around which objects are rotated and an angle (in radians)
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Quat {
        let axis = axis.normalize();
        let (s, c) = (angle * 0.5).sin_cos();

        Quat {
            cos_a2: c,
            axis_sin_a2: axis * s,
        }
    }

    /// Quaternion constructed from Euler angles (in radians).
    /// The order of rotations is pitch (X-axis), yaw (Y-axis), roll (Z-axis).
    /// This operation does NOT cause gimbal lock
    pub fn from_euler(pitch: f32, yaw: f32, roll: f32) -> Quat {
        let (sx, cx) = (pitch * 0.5).sin_cos();
        let (sy, cy) = (yaw * 0.5).sin_cos();
        let (sz, cz) = (roll * 0.5).sin_cos();

        let angle = cx * cy * cz + sx * sy * sz;

        let x = sx * cy * cz - cx * sy * sz;
        let y = cx * sy * cz + sx * cy * sz;
        let z = cx * cy * sz - sx * sy * cz;

        Quat {
            cos_a2: angle,
            axis_sin_a2: Vec3::new(x, y, z),
        }
    }

    /// Quaternion multiplication is composition of rotations.
    /// The resulting rotation is equivalent to applying `other` first, then `self`.
    pub fn mul(&self, other: &Quat) -> Quat {
        let angle = self.cos_a2 * other.cos_a2 - self.axis_sin_a2.dot(&other.axis_sin_a2);

        let axis = other.axis_sin_a2 * self.cos_a2 + 
            self.axis_sin_a2 * other.cos_a2 + 
            self.axis_sin_a2.cross(&other.axis_sin_a2);

        Quat { cos_a2: angle, axis_sin_a2: axis }
    }

    /// The conjugate of a quaternion represents the inverse rotation.
    /// For unit quaternions, the conjugate is also the inverse.
    pub fn conjugate(&self) -> Quat {
        Quat {
            cos_a2: self.cos_a2,
            axis_sin_a2: self.axis_sin_a2 * -1.0,
        }
    }

    /// Apply the rotation of the quaternion to a 3D vector.
    pub fn rotate_vec3(&self, v: Vec3) -> Vec3 {
        let t = self.axis_sin_a2.cross(&v) * 2.0;

        v + t * self.cos_a2 + self.axis_sin_a2.cross(&t) 
    }

    /// Normalize the quaternion to ensure it represents a valid rotation.
    /// This is important after multiple multiplications, as floating-point errors can accumulate.
    pub fn normalize(&self) -> Quat {
        let len = (self.cos_a2 * self.cos_a2 + self.axis_sin_a2.dot(&self.axis_sin_a2)).sqrt();
        if len == 0.0 { return Quat::IDENTITY; }

        Quat {
            cos_a2: self.cos_a2 / len,
            axis_sin_a2: self.axis_sin_a2 * (1.0/len),
        }
    }
    
    pub fn to_euler(&self) -> (f32, f32, f32) {
        let pitch = 
            (2.0 * (self.cos_a2 * self.axis_sin_a2.x - self.axis_sin_a2.y * self.axis_sin_a2.z)).asin();
        
        let yaw = 
            (self.cos_a2 * self.axis_sin_a2.y + self.axis_sin_a2.x * self.axis_sin_a2.z)
                .atan2(self.cos_a2 * self.cos_a2 - self.axis_sin_a2.x * self.axis_sin_a2.x - 
                    self.axis_sin_a2.y * self.axis_sin_a2.y + self.axis_sin_a2.z * self.axis_sin_a2.z);
        
        let roll = 
            (self.cos_a2 * self.axis_sin_a2.z + self.axis_sin_a2.x * self.axis_sin_a2.y)
                .atan2(self.cos_a2 * self.cos_a2 + self.axis_sin_a2.x * self.axis_sin_a2.x - 
                    self.axis_sin_a2.y * self.axis_sin_a2.y - self.axis_sin_a2.z * self.axis_sin_a2.z);
        
        (pitch, yaw, roll)
    }

}


/// A simple RGBA color struct with values in the range [0.0, 1.0].
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Color {
    pub const WHITE : Color = Color {r: 1.0, g: 1.0, b: 1.0, a: 1.0};
    pub const BLACK : Color = Color {r: 0.0, g: 0.0, b: 0.0, a: 1.0};
    pub const RED : Color = Color {r: 1.0, g: 0.0, b: 0.0, a: 1.0};
    pub const GREEN : Color = Color {r: 0.0, g: 1.0, b: 0.0, a: 1.0};
    pub const BLUE : Color = Color {r: 0.0, g: 0.0, b: 1.0, a: 1.0};

    /// Create a new color from RGBA values in the range [0, 255].
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0
        }
    }
    
    /// Create a new color from a gray scale value and an alpha value, both in the range [0, 255].
    pub fn from_gray_scale(gray: u8, a: u8) -> Color {
        let g = gray as f32 / 255.0;
        Color { r: g, g: g, b: g, a: a as f32 / 255.0 }
    }

    /// Create a new color from sdl2::pixels::Color, which has RGBA values in the range [0, 255].
    pub fn from_sdl(color: sdl2::pixels::Color) -> Color {
        Color {
            r: color.r as f32 / 255.0,
            g: color.g as f32 / 255.0,
            b: color.b as f32 / 255.0,
            a: color.a as f32 / 255.0
        }
    }

    /// Convert the color to a 32-bit ARGB format (0xAARRGGBB).  
    /// This format is used by SDL2 for texture manipulation.
    pub fn to_argb(self) -> u32 {
        ((255u32) << 24)
            | (((self.r * 255.0) as u32) << 16)
            | (((self.g * 255.0) as u32) << 8)
            | ((self.b * 255.0) as u32)
    }

    /// Alpha blend this color with another color, returning the resulting color.
    pub fn alpha_blend(&self, other: Color) -> Color {
        let c = self.a + other.a;
        let a1 = self.a / c;
        let a2 = other.a / c;
        Color {
            r: self.r * a1 + other.r * a2,
            g: self.g * a1 + other.g * a2,
            b: self.b * a1 + other.b * a2,
            a: c,
        }
    }

    /// Scale the color by a scalar value, returning the resulting color.
    /// This operation can result in saturated colors.
    pub fn scale(&self, p0: f32) -> Color {
        Color {
            r: self.r * p0,
            g: self.g * p0,
            b: self.b * p0,
            a: self.a * p0,
        }
    }
    
    /// Linearly interpolate between this color and another color by a factor of t.  
    /// t should be in the range [0.0, 1.0].
    pub fn lerp(a: &Color, b: &Color, t: f32) -> Color {
        *a * (1.0 - t) + *b * t
    }
}

impl ops::Add<Color> for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
            a: self.a + other.a,
        }
    }
}

impl ops::Mul<f32> for Color {
    type Output = Color;

    fn mul(self, scalar: f32) -> Color {
        Color {
            r: self.r * scalar,
            g: self.g * scalar,
            b: self.b * scalar,
            a: self.a * scalar,
        }
    }
}
