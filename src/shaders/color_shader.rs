use crate::geometry::Mesh;
use crate::shader::{interpolate, PixelShaderInput, Shader};
use crate::utils::Color;


pub struct ColorInput {
    pub color: Color,
}
impl PixelShaderInput for ColorInput {
    fn interpolate(mesh: &Mesh, indices: &[u32], weights: &[f32]) -> Self {
        let colors = mesh.colors.as_ref().unwrap();
        let color = interpolate(colors, indices, weights);
        Self { color }
    }
}


pub struct ColorShader;
impl Shader for ColorShader {
    type Input = ColorInput;

    fn shade(&self, input: &ColorInput) -> Color {
        input.color
    }
}
