use super::texture::Texture;

#[derive(Clone, Debug)]
pub enum UniformValue {
    None,
    Boolean(bool),
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat2([f32; 4]),
    Mat3([f32; 9]),
    Mat4([f32; 16]),
    Texture(Texture),
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: &'static str,
    pub value: UniformValue,
}

pub trait IntoUniform {
    fn into_uniform(&self) -> UniformValue;
}

pub trait Uniforms {
    fn uniforms(&self) -> Vec<Field>;
}
