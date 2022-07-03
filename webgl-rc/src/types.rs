use std::fmt::Display;
use std::convert::TryFrom;
use web_sys::{
    WebGlRenderingContext as Context,
};
use super::gl::GlError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DataType {
    Boolean,
    Float,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
    Sampler,
}

impl DataType {
    pub(crate) fn size_in_floats(self) -> Option<usize> {
        match self {
            DataType::Boolean => None,
            DataType::Float => Some(1),
            DataType::Vec2 => Some(2),
            DataType::Vec3 => Some(3),
            DataType::Vec4 => Some(4),
            DataType::Mat2 => Some(4),
            DataType::Mat3 => Some(9),
            DataType::Mat4 => Some(16),
            DataType::Sampler => None,
        }
    }
}

impl From<DataType> for &str {
    fn from(value: DataType) -> &'static str {
        match value {
            DataType::Boolean => "bool",
            DataType::Float => "float",
            DataType::Vec2 => "vec2",
            DataType::Vec3 => "vec3",
            DataType::Vec4 => "vec4",
            DataType::Mat2 => "mat2",
            DataType::Mat3 => "mat3",
            DataType::Mat4 => "mat4",
            DataType::Sampler => "sampler2D",
        }
    }
}

impl Display for DataType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        formatter.write_str((*self).into())
    }
}

impl DataType {
    pub fn is_numeric(self) -> bool {
        self != DataType::Boolean && self != DataType::Sampler
    }
    pub fn is_vector(self) -> bool {
        self == DataType::Vec2 || self == DataType::Vec3 || self == DataType::Vec4
    }
    pub fn is_matrix(self) -> bool {
        self == DataType::Mat2 || self == DataType::Mat3 || self == DataType::Mat4
    }
}

impl TryFrom<u32> for DataType {
    type Error = GlError;
    fn try_from(value: u32) -> Result<DataType, GlError> {
        match value {
            Context::BOOL => Ok(DataType::Boolean),
            Context::FLOAT => Ok(DataType::Float),
            Context::FLOAT_VEC2 => Ok(DataType::Vec2),
            Context::FLOAT_VEC3 => Ok(DataType::Vec3),
            Context::FLOAT_VEC4 => Ok(DataType::Vec4),
            Context::FLOAT_MAT2 => Ok(DataType::Mat2),
            Context::FLOAT_MAT3 => Ok(DataType::Mat3),
            Context::FLOAT_MAT4 => Ok(DataType::Mat4),
            Context::SAMPLER_2D => Ok(DataType::Sampler),
            _ => Err(GlError::UnsupportedType(None))
        }
    }
}

impl From<DataType> for u32 {
    fn from(data_type: DataType) -> Self {
        match data_type {
            DataType::Boolean => Context::BOOL,
            DataType::Float => Context::FLOAT,
            DataType::Vec2 => Context::FLOAT_VEC2,
            DataType::Vec3 => Context::FLOAT_VEC3,
            DataType::Vec4 => Context::FLOAT_VEC4,
            DataType::Mat2 => Context::FLOAT_MAT2,
            DataType::Mat3 => Context::FLOAT_MAT3,
            DataType::Mat4 => Context::FLOAT_MAT4,
            DataType::Sampler => Context::SAMPLER_2D,
        }
    }
}

pub trait TypeMark {
    fn data_type() -> DataType;
}
