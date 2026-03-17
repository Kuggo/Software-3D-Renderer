use std::ops::Sub;
use std::rc::Rc;
use crate::mesh::Mesh;
use crate::shader::{Material, BaseShader};
use crate::utils::*;


pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}


#[derive(Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub d: f32,
}
impl Plane {
    pub const XY: Plane = Plane { normal: Vec3::Z_AXIS, d: 0.0 };
    pub const XZ: Plane = Plane { normal: Vec3::Y_AXIS, d: 0.0 };
    pub const YZ: Plane = Plane { normal: Vec3::X_AXIS, d: 0.0 };
    
    pub fn new(normal: Vec3, d: f32) -> Self {
        Self { normal, d }
    }
    pub fn distance(&self, p: Vec3) -> f32 {
        self.normal.dot(&p) + self.d
    }

    pub fn intersect_line_seg(&self, p1: Vec3, p2: Vec3) -> Vec3 {
        let d1 = self.distance(p1);
        let d2 = self.distance(p2);
        let t = d1 / (d1 - d2);
        
        p1 + (p2 - p1) * t
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
    pub mesh: Rc<Mesh>,
    pub material: Rc<Material>,
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


