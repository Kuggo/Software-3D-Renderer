use crate::mesh::Mesh;
use crate::shader::{VaryingAttributes, Shader, interpolate};
use crate::utils::{Color, Vec3};


pub struct GouraudVaryings {
    color: Color,
}
impl VaryingAttributes for GouraudVaryings {
    fn calculate(mesh: &Mesh, indices: &[u32], weights: &[f32]) -> Self {
        let normals = mesh.normals.as_ref().unwrap().as_slice();
        
        fn to_gray(normal: Vec3) -> Color {
            let gray_scale = ((normal.z * -0.5 + 0.5) * 255.0) as u8;
            Color::new(gray_scale, gray_scale, gray_scale, 255)
        }
        
        let color = interpolate(&[
            to_gray(normals[indices[0] as usize]),
            to_gray(normals[indices[1] as usize]),
            to_gray(normals[indices[2] as usize]),
        ], weights);

        Self { color, }
    }
}


pub struct GouraudShader;
impl Shader for GouraudShader {
    type Input = GouraudVaryings;

    fn shade(&self, input: &GouraudVaryings) -> Color {
        input.color
    }
}