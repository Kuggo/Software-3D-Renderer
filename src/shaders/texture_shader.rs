use std::any::Any;
use crate::geometry::Mesh;
use crate::shader::{interpolate, VaryingAttributes, Shader};
use crate::texture::Texture;
use crate::utils::{Color, Vec2};

pub struct TextureInput {
    pub uv: Vec2
}
impl VaryingAttributes for TextureInput {
    fn calculate(mesh: &Mesh, indices: &[u32], weights: &[f32]) -> Self {
        let uvs = mesh.uvs.as_ref().unwrap();
        let uv = interpolate(uvs, indices, weights);
        Self { uv }
    }

    fn validate_mesh(mesh: &Mesh) -> bool {
        mesh.uvs.is_some()
    }
}


#[derive(Debug, Clone)]
pub struct TextureShader {
    pub texture: Option<Texture<Color>>,
}
impl<'a> Shader for TextureShader {
    type Input = TextureInput;

    fn shade(&self, input: &TextureInput) -> Color {
        let texture = self.texture.as_ref().unwrap();
        texture.sample(input.uv.x, input.uv.y)
    }

    fn assign_uniforms(&mut self, uniforms: &dyn Any) -> bool {
        if let Some(texture) = uniforms.downcast_ref::<Texture<Color>>() {
            self.texture = Some(texture.clone());
            true
        }
        else {
            false
        }
    }
}
