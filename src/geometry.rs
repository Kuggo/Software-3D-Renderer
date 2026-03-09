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

    /// Returns the perspective-correct linear interpolation between two vertices `a` and `b` by a factor `t`.
    /// This is used to interpolate vertex attributes in screen space after perspective projection, to avoid distortion.
    /// t should be in the range [0.0, 1.0].
    pub fn zlerp(a: &Vertex, b: &Vertex, t: f32) -> Vertex {
        let z = 1.0 / lerp(1.0/a.pos.z, 1.0/b.pos.z, t);

        let pos = Vec3::lerp(
            &a.pos.scale(1.0/a.pos.z), &b.pos.scale(1.0/a.pos.z), t
        ).scale(z);

        let color = Color::lerp(
            &a.color, &b.color, t
        ).scale(z);

        Vertex { pos, color }
    }
}


/// Primitives are the basic geometric shapes that make up a mesh.
/// Currently only triangles are supported, but lines and points could be added in the future.
#[derive(Debug, Copy, PartialEq, Clone)]
pub enum Primitive {
    Triangle(Triangle),
    Line(Vertex, Vertex),
    Point(Vertex),
}
#[derive(Debug, Copy, PartialEq, Clone)]
pub struct Triangle {
    pub a: Vertex,
    pub b: Vertex,
    pub c: Vertex,
}
impl Triangle {
    /// Creates a new Triangle with the given vertices.
    pub fn new(a: Vertex, b: Vertex, c: Vertex) -> Self {
        Self { a, b, c }
    }

    /// Computes the normal vector of the triangle using the cross product of its edges.
    pub fn get_normal(&self) -> Vec3 {
        let edge1 = self.b.pos.sub(&self.a.pos);
        let edge2 = self.c.pos.sub(&self.a.pos);
        edge1.cross(&edge2).normalize()
    }

    /// Computes the barycentric coordinates of a point `p` with respect to the triangle.
    pub fn barycentric_coords(&self, p: &Vec3) -> (f32, f32, f32) {
        let v0 = self.b.pos.sub(&self.a.pos);
        let v1 = self.c.pos.sub(&self.a.pos);
        let v2 = p.sub(&self.a.pos);

        let d00 = v0.dot(&v0);
        let d01 = v0.dot(&v1);
        let d11 = v1.dot(&v1);
        let d20 = v2.dot(&v0);
        let d21 = v2.dot(&v1);

        let denom = d00 * d11 - d01 * d01;
        if denom == 0.0 {
            return (0.0, 0.0, 0.0); // Degenerate triangle
        }

        let v = (d11 * d20 - d01 * d21) / denom;
        let w = (d00 * d21 - d01 * d20) / denom;
        let u = 1.0 - v - w;

        (u, v, w)
    }
}



// Objects and Scene

/// A Scene is a collection of objects.
pub struct Scene {
    pub objects: Vec<Object>,
}

/// An Object is an instance of a mesh with a specific transform (position, rotation, scale).
pub struct Object {
    pub transform: Transform,
    pub mesh: Mesh,
}

/// A Mesh is a collection of primitives (triangles, lines, points) that define an object's surface.
pub struct Mesh {
    pub primitives: Vec<Primitive>,
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
        let inv_pos = inv_rot.rotate_vec3( t.pos.scale(-1.0))
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
        let v = &self.rot.rotate_vec3(v);
        v.add(&self.pos)
    }

    /// Combines this transform with another local transform, returning a new transform
    /// that applies both transformations in sequence (`self`, `local`).
    pub fn combine_with(&self, local: &Transform) -> Transform {
        let scaled = local.pos.scale_vec(&self.scale);
        let rotated = self.rot.rotate_vec3(scaled);

        let pos = self.pos.add(&rotated);

        let rot = self.rot.mul(&local.rot);

        let scale = self.scale.scale_vec(&local.scale);

        Transform { pos, rot, scale }
    }
}


