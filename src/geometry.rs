use crate::utils::*;


#[derive(Debug, Copy, PartialEq, Clone)]
pub struct Vertex {
    pub pos: Vec3,
    //pub normal: Vec3,
    //pub uv: Vec2,
}

#[derive(Debug, Copy, PartialEq, Clone)]
pub enum Primitive {
    Triangle(Triangle),
    //Line(Line),
    //Point(Point),
}
#[derive(Debug, Copy, PartialEq, Clone)]
pub struct Triangle {
    pub a: Vertex,
    pub b: Vertex,
    pub c: Vertex,
}


// Objects and Scene
pub struct Scene {
    pub objects: Vec<Object>,
}

pub struct Object {
    pub transform: Transform,
    pub mesh: Mesh,
}

pub struct Mesh {   // TODO make vertices be indirect
    pub primitives: Vec<Primitive>,
}

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,
}
impl Transform {
    pub const IDENTITY: Transform = Transform { pos: Vec3::ZERO, rot: Quat::IDENTITY, scale: Vec3::IDENTITY
    };

    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self { pos: position, rot: rotation, scale }
    }

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

    pub fn apply_to(&self, v: &Vec3) -> Vec3 {
        let v = v.scale_vec(&self.scale);
        let v = &self.rot.rotate_vec3(v);
        v.add(&self.pos)
    }

    pub fn combine_with(&self, local: &Transform) -> Transform {
        let scaled = local.pos.scale_vec(&self.scale);
        let rotated = self.rot.rotate_vec3(scaled);

        let pos = self.pos.add(&rotated);

        let rot = self.rot.mul(&local.rot);

        let scale = self.scale.scale_vec(&local.scale);

        Transform { pos, rot, scale }
    }
}


