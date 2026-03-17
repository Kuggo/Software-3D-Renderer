use crate::mesh::Mesh;
use crate::shader::{VaryingAttributes, Shader, interpolate_mesh_attribute};
use crate::utils::{Color, Vec3};


pub struct SmoothVaryings {
    normal: Vec3,
}
impl VaryingAttributes for SmoothVaryings {
    fn calculate(mesh: &Mesh, indices: &[u32], weights: &[f32]) -> Self {
        let normals = mesh.normals.as_ref().unwrap().as_slice();
        
        let normal = interpolate_mesh_attribute(normals, indices, weights).normalize();

        Self { normal, }
    }
}


pub struct SmoothShader;
impl Shader for SmoothShader {
    type Input = SmoothVaryings;

    fn shade(&self, input: &SmoothVaryings) -> Color {
        // For now, just return a color based on the normal and uv for testing.
        let gray_scale = ((input.normal.z * 0.5 + 0.5) * 255.0) as u8;
        Color::new(gray_scale, gray_scale, gray_scale, 255)
    }
}