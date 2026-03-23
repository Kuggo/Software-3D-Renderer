use std::any::Any;
use std::ops;
use std::rc::Rc;
use crate::mesh::Mesh;
use crate::utils::{Color, Vec2, Vec3};


pub struct Material {
    pub shader: Box<dyn BaseShader>,
}


pub trait VaryingAttributes {
    fn calculate(mesh: &Mesh, indices: &[u32], weights: &[f32]) -> Self
    where Self: Sized;

    fn validate_mesh(mesh: &Mesh) -> bool {
        // Default implementation: check if the mesh has the required attributes for this shader input.
        // This can be overridden by specific shader inputs if they require additional attributes.
        true
    }
}


pub trait BaseShader {
    fn shade(&self, mesh: &Mesh, indices: &[u32], weights: &[f32]) -> Color;

    fn validate_mesh(&self, mesh: &Mesh) -> bool;

    fn assign_uniforms(&mut self, uniforms: &dyn Any) -> bool;
}
impl<T> BaseShader for T
where T: Shader, {
    fn shade(&self, mesh: &Mesh, indices: &[u32], weights: &[f32]) -> Color {
        let input = T::Input::calculate(mesh, indices, weights);
        self.shade(&input)
    }

    fn validate_mesh(&self, mesh: &Mesh) -> bool {
        T::Input::validate_mesh(mesh)
    }

    fn assign_uniforms(&mut self, uniforms: &dyn Any) -> bool {
        if let Some(unif) = uniforms.downcast_ref::<T::Uniforms>() {
            T::assign_uniforms(self, unif);
            true
        }
        else {
            false
        }

    }
}

pub trait Shader {
    type Input: VaryingAttributes;
    type Uniforms: 'static;

    fn shade(&self, input: &Self::Input) -> Color;

    fn assign_uniforms(&mut self, uniforms: &Self::Uniforms) {
        // Default implementation does nothing, but specific shaders can override this to assign uniform values.
    }
}



pub fn interpolate_mesh_attribute<T>(attribute: &[T], verts: &[u32], weights: &[f32]) -> T
where T: ops::Add<T, Output = T> + ops::Mul<f32, Output = T> + Copy, {
    let mut sum = attribute[verts[0] as usize] * weights[0];
    for i in 1..verts.len() {
        sum = sum + attribute[verts[i] as usize] * weights[i];
    }
    sum
}

pub fn interpolate<T>(attribute: &[T], weights: &[f32]) -> T
where T: ops::Add<T, Output = T> + ops::Mul<f32, Output = T> + Copy, {
    let mut sum = attribute[0] * weights[0];
    for i in 1..weights.len() {
        sum = sum + attribute[i] * weights[i];
    }
    sum
}
