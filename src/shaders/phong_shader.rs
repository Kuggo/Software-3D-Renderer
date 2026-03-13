use crate::geometry::Mesh;
use crate::shader::{PixelShaderInput, BaseShader, Shader, interpolate};
use crate::utils::{Color, Vec2, Vec3};

pub struct PhongInput {
    normal: Vec3,
    uv: Vec2,
}
impl PixelShaderInput for PhongInput {
    fn interpolate(mesh: &Mesh, indices: &[u32], weights: &[f32]) -> Self {
        let normals = mesh.normals.as_ref().unwrap();
        let uvs = mesh.uvs.as_ref().unwrap();

        let normal = interpolate(normals, indices, weights);
        let uv = interpolate(uvs, indices, weights);
        Self { normal, uv, }
    }
}


pub struct PhongShader;
impl Shader for PhongShader {
    type Input = PhongInput;

    fn shade(&self, input: &PhongInput) -> Color {
        // For now, just return a color based on the normal and uv for testing.
        let r = (input.normal.x.abs() * 255.0) as u8;
        let g = (input.normal.y.abs() * 255.0) as u8;
        let b = (input.normal.z.abs() * 255.0) as u8;
        Color::new(r, g, b, 255)
    }
}