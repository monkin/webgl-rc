use crate::{DataType, IntoUniform, TypeMark, UniformValue, Writable};

use glm::{Vec1, Vec2, Vec3, Vec4};

impl TypeMark for Vec1 {
    fn data_type() -> DataType {
        DataType::Float
    }
}

impl TypeMark for Vec2 {
    fn data_type() -> DataType {
        DataType::Vec2
    }
}

impl TypeMark for Vec3 {
    fn data_type() -> DataType {
        DataType::Vec3
    }
}

impl TypeMark for Vec4 {
    fn data_type() -> DataType {
        DataType::Vec4
    }
}

impl IntoUniform for Vec1 {
    fn into_uniform(&self) -> UniformValue {
        self.x.into_uniform()
    }
}

impl IntoUniform for Vec2 {
    fn into_uniform(&self) -> UniformValue {
        (self.x, self.y).into_uniform()
    }
}

impl IntoUniform for Vec3 {
    fn into_uniform(&self) -> UniformValue {
        (self.x, self.y, self.z).into_uniform()
    }
}

impl IntoUniform for Vec4 {
    fn into_uniform(&self) -> UniformValue {
        (self.x, self.y, self.z, self.w).into_uniform()
    }
}

impl Writable for Vec1 {
    fn write(&self, output: &mut Vec<f32>) {
        output.push(self.x);
    }

    fn stride() -> usize {
        1
    }
}

impl Writable for Vec2 {
    fn write(&self, output: &mut Vec<f32>) {
        output.push(self.x);
        output.push(self.y);
    }

    fn stride() -> usize {
        2
    }
}

impl Writable for Vec3 {
    fn write(&self, output: &mut Vec<f32>) {
        output.push(self.x);
        output.push(self.y);
        output.push(self.z);
    }

    fn stride() -> usize {
        3
    }
}

impl Writable for Vec4 {
    fn write(&self, output: &mut Vec<f32>) {
        output.push(self.x);
        output.push(self.y);
        output.push(self.z);
        output.push(self.w);
    }

    fn stride() -> usize {
        4
    }
}

use glm::{Mat2, Mat3, Mat4};

impl TypeMark for Mat2 {
    fn data_type() -> DataType {
        DataType::Mat2
    }
}

impl TypeMark for Mat3 {
    fn data_type() -> DataType {
        DataType::Mat3
    }
}

impl TypeMark for Mat4 {
    fn data_type() -> DataType {
        DataType::Mat4
    }
}

impl IntoUniform for Mat2 {
    fn into_uniform(&self) -> UniformValue {
        UniformValue::Mat2([self.m11, self.m12, self.m21, self.m22])
    }
}

impl IntoUniform for Mat3 {
    fn into_uniform(&self) -> UniformValue {
        UniformValue::Mat3([
            self.m11, self.m12, self.m13, self.m21, self.m22, self.m23, self.m31, self.m32,
            self.m33,
        ])
    }
}

impl IntoUniform for Mat4 {
    fn into_uniform(&self) -> UniformValue {
        UniformValue::Mat4([
            self.m11, self.m12, self.m13, self.m14, self.m21, self.m22, self.m23, self.m24,
            self.m31, self.m32, self.m33, self.m34, self.m41, self.m42, self.m43, self.m44,
        ])
    }
}
