

const FP_TOLERANCE: f32 = 0.0001;

#[derive(Debug, Copy, PartialEq, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
impl Vec2 {
    pub const ZERO: Vec2 = Vec2 {x: 0.0, y: 0.0};

    pub fn new(x: f32, y: f32) -> Self {
        Self {x, y}
    }

    pub fn equals_fp(&self, other: &Vec2) -> bool {
        (self.x - other.x).abs() < FP_TOLERANCE &&
            (self.y - other.y).abs() < FP_TOLERANCE
    }

    pub fn dot(&self, other: &Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn add(&self, other: &Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }

    pub fn sub(&self, other: &Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }

    pub fn scale(&self, scalar: f32) -> Vec2 {
        Vec2::new(self.x * scalar, self.y * scalar)
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Vec2 {
        let l = self.length();
        if l == 0.0 {
            return Vec2::new(0.0, 0.0);
        }
        Vec2::new(self.x / l, self.y / l)
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

    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 {x, y, z }
    }

    pub fn from_polar_coords(len: f32, pitch: f32, yaw: f32) -> Vec3 {
        Vec3::new(
            len * pitch.cos() * yaw.cos(),
            len * pitch.sin(),
            len * pitch.cos() * yaw.sin())
    }

    pub fn null(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
    }

    pub fn equals_fp(&self, other: &Vec3) -> bool {
        (self.x - other.x).abs() < FP_TOLERANCE &&
            (self.y - other.y).abs() < FP_TOLERANCE &&
            (self.z - other.z).abs() < FP_TOLERANCE

    }

    pub fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x)
    }

    pub fn add(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    pub fn sub(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    pub fn scale(&self, scalar: f32) -> Vec3 {
        Vec3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }

    /// Hadamard product (component-wise multiplication)
    pub fn scale_vec(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.x * other.x,
            self.y * other.y,
            self.z * other.z,
        )
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn manhattan(&self, other: &Vec3) -> f32 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }

    pub fn normalize(&self) -> Vec3 {
        let l = self.length();
        if l == 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }
        Vec3::new(self.x / l, self.y / l, self.z / l)
    }

    pub fn polar(&self) -> (f32, f32, f32) {
        let len = self.length();
        if len == 0.0 {
            return (0.0, 0.0, 0.0);
        }
        let pitch = (self.y / len).asin();
        let yaw = self.y.atan2(self.x);
        (len, pitch, yaw)
    }

    pub fn colinear(&self, other: &Vec3) -> bool {
        let l1 = self.length();
        let l2 = other.length();
        if l1 == 0.0 || l2 == 0.0 {
            return true;
        }
        let n1 = self.scale(1.0 / l1);
        let n2 = other.scale(1.0 / l2);
        n1.equals_fp(&n2)
    }

    pub fn angle(&self, other: &Vec3) -> f32 {
        if self.null() || other.null() {
            return 0.0;
        }
        (self.dot(&other) / (self.length() * other.length())).acos()
    }

    pub fn project_onto(&self, other: &Vec3) -> Vec3 {
        let l = other.length();
        if l == 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }
        other.scale(self.dot(other) / l)
    }

    pub fn orthogonal_component(&self, other: &Vec3) -> Vec3 {
        self.sub(&self.project_onto(&other))
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
            axis_sin_a2: axis.scale(s),
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

        let axis = other.axis_sin_a2.scale(self.cos_a2)
            .add(&self.axis_sin_a2.scale(other.cos_a2))
            .add(&self.axis_sin_a2.cross(&other.axis_sin_a2));

        Quat { cos_a2: angle, axis_sin_a2: axis }
    }

    /// The conjugate of a quaternion represents the inverse rotation.
    /// For unit quaternions, the conjugate is also the inverse.
    pub fn conjugate(&self) -> Quat {
        Quat {
            cos_a2: self.cos_a2,
            axis_sin_a2: self.axis_sin_a2.scale(-1.0),
        }
    }

    /// Apply the rotation of the quaternion to a 3D vector.
    pub fn rotate_vec3(&self, v: Vec3) -> Vec3 {
        let t = self.axis_sin_a2.cross(&v).scale(2.0);

        v.add( &t.scale(self.cos_a2) ).add( &self.axis_sin_a2.cross(&t) )
    }

    /// Normalize the quaternion to ensure it represents a valid rotation.
    /// This is important after multiple multiplications, as floating-point errors can accumulate.
    pub fn normalize(&self) -> Quat {
        let len = (self.cos_a2 * self.cos_a2 + self.axis_sin_a2.dot(&self.axis_sin_a2)).sqrt();
        if len == 0.0 { return Quat::IDENTITY; }

        Quat {
            cos_a2: self.cos_a2 / len,
            axis_sin_a2: self.axis_sin_a2.scale(1.0/len),
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


#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}
impl Color {
    pub const WHITE : Color = Color {r: 1.0, g: 1.0, b: 1.0, a: 1.0};
    pub const BLACK : Color = Color {r: 0.0, g: 0.0, b: 0.0, a: 1.0};
    pub const RED : Color = Color {r: 1.0, g: 0.0, b: 0.0, a: 1.0};
    pub const GREEN : Color = Color {r: 0.0, g: 1.0, b: 0.0, a: 1.0};
    pub const BLUE : Color = Color {r: 0.0, g: 0.0, b: 1.0, a: 1.0};

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0
        }
    }

    pub fn to_argb(self) -> u32 {
        ((255u32) << 24)
            | (((self.r * 255.0) as u32) << 16)
            | (((self.g * 255.0) as u32) << 8)
            | ((self.b * 255.0) as u32)
    }

    pub fn sdl_format(&self) -> sdl2::pixels::Color {
        sdl2::pixels::Color::RGBA(
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8
        )
    }

    pub fn from_sdl(color: sdl2::pixels::Color) -> Color {
        Color {
            r: color.r as f32 / 255.0,
            g: color.g as f32 / 255.0,
            b: color.b as f32 / 255.0,
            a: color.a as f32 / 255.0
        }
    }

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
}
