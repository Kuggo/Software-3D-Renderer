use std::ops;
use crate::geometry::{Mesh, Primitive};
use crate::utils::{Color, Vec2, Vec3};


pub struct Material<'a> {
    pub shader: &'a dyn BaseShader,
}


pub trait PixelShaderInput {
    fn interpolate(mesh: &Mesh, indices: &[u32], weights: &[f32]) -> Self
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
}
impl<T> BaseShader for T
where T: Shader, {
    fn shade(&self, mesh: &Mesh, indices: &[u32], weights: &[f32]) -> Color {
        let input = T::Input::interpolate(mesh, indices, weights);
        self.shade(&input)
    }

    fn validate_mesh(&self, mesh: &Mesh) -> bool {
        T::Input::validate_mesh(mesh)
    }
}

pub trait Shader {
    type Input: PixelShaderInput;

    fn shade(&self, input: &Self::Input) -> Color;
}



pub fn interpolate<T>(attribute: &[T], verts: &[u32], weights: &[f32]) -> T
where T: ops::Add<T, Output = T> + ops::Mul<f32, Output = T> + Copy, {
    let mut sum = attribute[verts[0] as usize] * weights[0];
    for i in 1..verts.len() {
        sum = sum + attribute[verts[i] as usize] * weights[i];
    }
    sum
}
