use super::data_buffer::Writable;
use super::texture::Texture;
use super::types::{DataType, TypeMark};
use crate::uniforms::{IntoUniform, Value};

// f32
impl Writable for f32 {
    fn write(&self, output: &mut Vec<f32>) {
        output.push(*self);
    }
    fn stride() -> usize {
        return 1;
    }
}

impl TypeMark for f32 {
    fn data_type() -> DataType {
        DataType::Float
    }
}

impl IntoUniform for f32 {
    fn into_uniform(&self) -> Value {
        Value::Float(*self)
    }
}

// Texture

impl TypeMark for Texture {
    fn data_type() -> DataType {
        DataType::Sampler
    }
}

impl IntoUniform for Texture {
    fn into_uniform(&self) -> Value {
        Value::Texture(self.clone())
    }
}

// Option<Texture>

impl TypeMark for Option<Texture> {
    fn data_type() -> DataType {
        DataType::Sampler
    }
}

impl IntoUniform for Option<Texture> {
    fn into_uniform(&self) -> Value {
        self.as_ref()
            .map(|texture| Value::Texture(texture.clone()))
            .unwrap_or(Value::None)
    }
}

// Boolean

impl TypeMark for bool {
    fn data_type() -> DataType {
        DataType::Boolean
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl IntoUniform for bool {
    fn into_uniform(&self) -> Value {
        Value::Boolean(*self)
    }
}

// [f32;2]

impl TypeMark for [f32; 2] {
    fn data_type() -> DataType {
        DataType::Vec2
    }
}

impl IntoUniform for [f32; 2] {
    fn into_uniform(&self) -> Value {
        Value::Vec2(*self)
    }
}

impl Writable for [f32; 2] {
    fn write(&self, output: &mut Vec<f32>) {
        for v in self {
            output.push(*v);
        }
    }
    fn stride() -> usize {
        2
    }
}

// [f32;3]

impl TypeMark for [f32; 3] {
    fn data_type() -> DataType {
        DataType::Vec3
    }
}

impl IntoUniform for [f32; 3] {
    fn into_uniform(&self) -> Value {
        Value::Vec3(*self)
    }
}

impl Writable for [f32; 3] {
    fn write(&self, output: &mut Vec<f32>) {
        for v in self {
            output.push(*v);
        }
    }
    fn stride() -> usize {
        3
    }
}

// [f32;4]

impl TypeMark for [f32; 4] {
    fn data_type() -> DataType {
        DataType::Vec4
    }
}

impl IntoUniform for [f32; 4] {
    fn into_uniform(&self) -> Value {
        Value::Vec4(*self)
    }
}

impl Writable for [f32; 4] {
    fn write(&self, output: &mut Vec<f32>) {
        for v in self {
            output.push(*v);
        }
    }
    fn stride() -> usize {
        4
    }
}

// (f32, f32)

impl TypeMark for (f32, f32) {
    fn data_type() -> DataType {
        DataType::Vec2
    }
}

impl IntoUniform for (f32, f32) {
    fn into_uniform(&self) -> Value {
        Value::Vec2([self.0, self.1])
    }
}

impl Writable for (f32, f32) {
    fn write(&self, output: &mut Vec<f32>) {
        output.push(self.0);
        output.push(self.1);
    }
    fn stride() -> usize {
        2
    }
}

// (f32, f32, f32)

impl TypeMark for (f32, f32, f32) {
    fn data_type() -> DataType {
        DataType::Vec3
    }
}

impl IntoUniform for (f32, f32, f32) {
    fn into_uniform(&self) -> Value {
        Value::Vec3([self.0, self.1, self.2])
    }
}

impl Writable for (f32, f32, f32) {
    fn write(&self, output: &mut Vec<f32>) {
        output.push(self.0);
        output.push(self.1);
        output.push(self.2);
    }
    fn stride() -> usize {
        3
    }
}

// (f32, f32, f32, f32)

impl TypeMark for (f32, f32, f32, f32) {
    fn data_type() -> DataType {
        DataType::Vec4
    }
}

impl IntoUniform for (f32, f32, f32, f32) {
    fn into_uniform(&self) -> Value {
        Value::Vec4([self.0, self.1, self.2, self.3])
    }
}

impl Writable for (f32, f32, f32, f32) {
    fn write(&self, output: &mut Vec<f32>) {
        output.push(self.0);
        output.push(self.1);
        output.push(self.2);
        output.push(self.3);
    }
    fn stride() -> usize {
        4
    }
}
