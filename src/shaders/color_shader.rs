use crate::mesh::Mesh;
use crate::shader::{interpolate_mesh_attribute, VaryingAttributes, Shader};
use crate::utils::Color;


pub struct ColorVarying {
    pub color: Color,
}
impl VaryingAttributes for ColorVarying {
    fn calculate(mesh: &Mesh, indices: &[u32], weights: &[f32]) -> Self {
        let colors = mesh.colors.as_ref().unwrap();
        let color = interpolate_mesh_attribute(colors, indices, weights);
        Self { color }
    }
    
    fn validate_mesh(mesh: &Mesh) -> bool {
        mesh.colors.is_some()
    }
}


pub struct ColorShader;
impl Shader for ColorShader {
    type Input = ColorVarying;

    fn shade(&self, input: &ColorVarying) -> Color {
        input.color
    }
}
