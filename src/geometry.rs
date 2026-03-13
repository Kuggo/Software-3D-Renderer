use crate::shader::{Material, BaseShader};
use crate::utils::*;


pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}


/// Vertex containing attributes used to render primitives.
#[derive(Debug, Copy, PartialEq, Clone)]
pub struct Vertex {
    pub pos: Vec3,
    pub color: Color,
    //pub normal: Vec3,
    //pub uv: Vec2,
}
impl Vertex {
    /// Creates a new Vertex with the given attributes.
    pub fn new(pos: Vec3, color: Color) -> Self {
        Self { pos, color }
    }

    /// Returns the linear interpolation between two vertices `a` and `b` by a factor `t`.
    /// t should be in the range [0.0, 1.0].
    pub fn lerp(a: &Vertex, b: &Vertex, t: f32) -> Vertex {
        let pos = Vec3::lerp(&a.pos, &b.pos, t);
        let color = Color::lerp(&a.color, &b.color, t);
        Vertex { pos, color }
    }
}


/// Primitives are the basic geometric shapes that make up a mesh.
/// Currently only triangles are supported, but lines and points could be added in the future.
#[derive(Debug, Copy, PartialEq, Clone)]
pub enum Primitive {
    Point(u32),     // Single vertex (point)
    Line(u32, u32), // Two vertices (line)
    Triangle(u32, u32, u32), // Three vertices (triangle)
}



// Objects and Scene

/// A Scene is a collection of objects.
pub struct Scene<'a> {
    pub objects: Vec<Object<'a>>,
}

/// An Object is an instance of a mesh with a specific transform (position, rotation, scale).
pub struct Object<'a> {
    pub transform: Transform,
    pub mesh: Mesh,
    pub material: &'a Material<'a>,
}

/// A Mesh is a collection of primitives (triangles, lines, points) that define an object's surface,
/// and their attributes
pub struct Mesh {
    pub primitives: Vec<Primitive>,
    pub positions: Vec<Vec3>,
    pub colors: Option<Vec<Color>>,
    pub normals: Option<Vec<Vec3>>,
    pub uvs: Option<Vec<Vec2>>,
}

/// A Transform represents the position, rotation, and scale of an object in 3D space.
/// It can be chained together to combine multiple transformations.
/// It is applied to vertices.
#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,
}
impl Transform {
    pub const IDENTITY: Transform = Transform { pos: Vec3::ZERO, rot: Quat::IDENTITY, scale: Vec3::IDENTITY
    };

    /// Creates a new Transform with the given position, rotation, and scale.
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self { pos: position, rot: rotation, scale }
    }

    /// Returns the inverse of a transform, which can be used to reverse its effects on vertices.
    pub fn inverse(t: &Transform) -> Transform {
        let inv_rot = t.rot.conjugate();

        let inv_scale = Vec3::new(
            1.0 / t.scale.x,
            1.0 / t.scale.y,
            1.0 / t.scale.z,
        );
        let inv_pos = inv_rot.rotate_vec3( t.pos * -1.0)
            .scale_vec( &inv_scale );

        Transform {
            pos: inv_pos,
            rot: inv_rot,
            scale: inv_scale,
        }
    }

    /// Applies the transform to a given vertex position, returning the transformed position.
    pub fn apply_to(&self, v: &Vec3) -> Vec3 {
        let v = v.scale_vec(&self.scale);
        let v = self.rot.rotate_vec3(v);
        v + self.pos
    }

    /// Combines this transform with another local transform, returning a new transform
    /// that applies both transformations in sequence (`self`, `local`).
    pub fn combine_with(&self, local: &Transform) -> Transform {
        let scaled = local.pos.scale_vec(&self.scale);
        let rotated = self.rot.rotate_vec3(scaled);

        let pos = self.pos + rotated;

        let rot = self.rot.mul(&local.rot);

        let scale = self.scale.scale_vec(&local.scale);

        Transform { pos, rot, scale }
    }
}


