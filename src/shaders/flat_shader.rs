use crate::mesh::Mesh;
use crate::shader::{VaryingAttributes, Shader, interpolate_mesh_attribute};
use crate::utils::{Color, Vec3};


pub struct FlatVaryings {
    normal: Vec3,
}
impl VaryingAttributes for FlatVaryings {
    fn calculate(mesh: &Mesh, indices: &[u32], weights: &[f32]) -> Self {
        let positions = mesh.positions.as_slice();

        let normal = Vec3::cross(
            &(positions[indices[1] as usize] - positions[indices[0] as usize]),
            &(positions[indices[2] as usize] - positions[indices[0] as usize]),
        ).normalize();
        Self { normal, }
    }
}


pub struct FlatShader;
impl Shader for FlatShader {
    type Input = FlatVaryings;

    fn shade(&self, input: &FlatVaryings) -> Color {
        // For now, just return a color based on the normal and uv for testing.
        let gray_scale = ((input.normal.z * 0.5 + 0.5) * 255.0) as u8;
        Color::new(gray_scale, gray_scale, gray_scale, 255)
    }
}

