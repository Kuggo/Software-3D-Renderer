use std::ops;
use crate::geometry::{Mesh, Primitive};
use crate::utils::{Color, Vec2, Vec3};


pub struct Material<'a> {
    pub shader: &'a dyn BaseShader,
}


pub trait PixelShaderInput {
    fn interpolate(mesh: &Mesh, indices: &[u32], weights: &[f32]) -> Self
    where Self: Sized;
}


pub trait BaseShader {
    fn shade(&self, mesh: &Mesh, indices: &[u32], weights: &[f32]) -> Color;
}
impl<T> BaseShader for T
where T: Shader, {
    fn shade(&self, mesh: &Mesh, indices: &[u32], weights: &[f32]) -> Color {
        let input = T::Input::interpolate(mesh, indices, weights);
        self.shade(&input)
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
