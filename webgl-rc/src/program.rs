use core::convert::TryFrom;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryInto;
use std::rc::Rc;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlUniformLocation};

use super::data_buffer::{Item, ItemsBuffer};
use super::gl::Gl;
use super::gl::GlError;
use super::settings::Settings;
use super::texture::{Texture, TEXTURES_COUNT};
use super::types::DataType;
use crate::uniforms::{Uniforms, UniformValue};

#[repr(u32)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
pub enum PrimitiveType {
    Points = WebGlRenderingContext::POINTS,
    LineStrip = WebGlRenderingContext::LINE_STRIP,
    LineLoop = WebGlRenderingContext::LINE_LOOP,
    Lines = WebGlRenderingContext::LINES,
    TriangleStrip = WebGlRenderingContext::TRIANGLE_STRIP,
    TriangleFan = WebGlRenderingContext::TRIANGLE_FAN,
    Triangles = WebGlRenderingContext::TRIANGLES,
}

#[derive(Clone, Debug)]
struct AttributeInfo {
    name: String,
    location: u32,
    data_type: DataType,
}

#[derive(Clone, Debug)]
struct UniformInfo {
    name: String,
    location: WebGlUniformLocation,
    data_type: DataType,
}

#[derive(Clone, Debug)]
struct Shader {
    gl: Gl,
    handle: WebGlShader,
    source: String,
}

impl PartialEq for Shader {
    fn eq(&self, other: &Shader) -> bool {
        self.handle == other.handle
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        self.gl.context().delete_shader(Some(&self.handle));
    }
}

impl Shader {
    fn new(gl: Gl, source: &str, shader_type: u32) -> Result<Shader, GlError> {
        let ctx = gl.context();
        let handle = ctx
            .create_shader(shader_type)
            .ok_or_else(|| GlError::UnknownError(None))?;

        ctx.shader_source(&handle, source);
        ctx.compile_shader(&handle);

        let status = ctx
            .get_shader_parameter(&handle, WebGlRenderingContext::COMPILE_STATUS)
            .as_bool()
            .ok_or_else(|| GlError::UnknownError(None))?;

        if !status {
            return Err(GlError::ShaderCompilationError {
                source: source.into(),
                info: ctx.get_shader_info_log(&handle),
            });
        }

        return Ok(Shader {
            gl,
            handle,
            source: source.into(),
        });
    }
}

#[derive(Debug, Clone)]
struct ProgramData {
    gl: Gl,
    handle: WebGlProgram,
    vertex_shader: Shader,
    fragment_shader: Shader,
    attributes: Vec<AttributeInfo>,
    uniforms: Vec<UniformInfo>,
}

impl Drop for ProgramData {
    fn drop(&mut self) {
        let wgl = self.gl.context();
        wgl.delete_program(Some(&self.handle));
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    data: Rc<ProgramData>,
}

impl PartialEq for Program {
    fn eq(&self, other: &Program) -> bool {
        self.data.handle == other.data.handle
    }
}

impl Eq for Program {}

impl Program {
    fn collect_attributes(
        ctx: &WebGlRenderingContext,
        program: &WebGlProgram,
    ) -> Result<Vec<AttributeInfo>, GlError> {
        let attributes_count = ctx
            .get_program_parameter(&program, WebGlRenderingContext::ACTIVE_ATTRIBUTES)
            .as_f64()
            .ok_or_else(|| GlError::UnknownError(Some("Failed to get attributes count".into())))?
            as u32;

        let mut result = Vec::with_capacity(attributes_count as usize);

        for i in 0..attributes_count {
            let info = ctx.get_active_attrib(&program, i).ok_or_else(|| {
                GlError::UnknownError(Some("Failed to get attribute info".into()))
            })?;

            // Arrays are not supported
            if info.size() != 1 {
                return Err(GlError::UnsupportedType(Some(info.name())));
            }

            let location = ctx.get_attrib_location(&program, &info.name());
            result.push(AttributeInfo {
                name: info.name(),
                data_type: DataType::try_from(info.type_())
                    .map_err(|_| GlError::UnsupportedType(Some(info.name())))?,
                location: location.try_into().map_err(|_| {
                    GlError::UnknownError(Some("Negative attribute location".to_string()))
                })?,
            });
        }
        return Ok(result);
    }

    fn collect_uniforms(
        ctx: &WebGlRenderingContext,
        program: &WebGlProgram,
    ) -> Result<Vec<UniformInfo>, GlError> {
        let uniforms_count = ctx
            .get_program_parameter(&program, WebGlRenderingContext::ACTIVE_UNIFORMS)
            .as_f64()
            .ok_or_else(|| GlError::UnknownError(Some("Failed to get uniforms count".into())))?
            as u32;

        let mut result = Vec::with_capacity(uniforms_count as usize);

        for i in 0..uniforms_count {
            let info = ctx
                .get_active_uniform(&program, i)
                .ok_or_else(|| GlError::UnknownError(Some("Failed to get uniform info".into())))?;

            // Arrays are not supported
            if info.size() != 1 {
                return Err(GlError::UnsupportedType(Some(info.name())));
            }

            let location = ctx
                .get_uniform_location(&program, &info.name())
                .ok_or_else(|| {
                    GlError::UnknownError(Some("Failed to get uniform location".into()))
                })?;
            result.push(UniformInfo {
                name: info.name(),
                data_type: DataType::try_from(info.type_())
                    .map_err(|_| GlError::UnsupportedType(Some(info.name())))?,
                location,
            });
        }

        return Ok(result);
    }

    pub(crate) fn new(
        gl: Gl,
        fragment_shader_source: &str,
        vertex_shader_source: &str,
    ) -> Result<Self, GlError> {
        let ctx: &WebGlRenderingContext = gl.context();

        let vertex_shader = Shader::new(
            gl.clone(),
            vertex_shader_source,
            WebGlRenderingContext::VERTEX_SHADER,
        )?;
        let fragment_shader = Shader::new(
            gl.clone(),
            fragment_shader_source,
            WebGlRenderingContext::FRAGMENT_SHADER,
        )?;

        let program = ctx.create_program().unwrap();
        ctx.attach_shader(&program, &vertex_shader.handle);
        ctx.attach_shader(&program, &fragment_shader.handle);
        ctx.link_program(&program);

        let link_status = ctx
            .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
            .as_bool()
            .ok_or_else(|| GlError::UnknownError(Some("Failed to get linking status".into())))?;

        if !link_status {
            return Err(GlError::ProgramLinkingError {
                vertex: vertex_shader_source.into(),
                fragment: fragment_shader_source.into(),
                info: ctx.get_program_info_log(&program),
            });
        }

        return Ok(Program {
            data: Rc::new(ProgramData {
                gl: gl.clone(),
                handle: program.clone(),
                vertex_shader,
                fragment_shader,
                attributes: Program::collect_attributes(&ctx, &program)?,
                uniforms: Program::collect_uniforms(&ctx, &program)?,
            }),
        });
    }

    pub(crate) fn handle(&self) -> WebGlProgram {
        self.data.handle.clone()
    }

    pub(self) fn set_attributes<T: Item>(&self, buffer: &ItemsBuffer<T>) {
        let gl: &WebGlRenderingContext = self.data.gl.context();
        let mut offset: usize = 0;

        self.data.gl.apply(
            Gl::settings()
                .items_buffer((*buffer).clone())
                .program(self.clone()),
            || {
                for item in T::layout() {
                    (&self.data.attributes)
                        .iter()
                        .find(|i| i.name == item.name)
                        .map(|info| {
                            gl.vertex_attrib_pointer_with_i32(
                                info.location,
                                item.data_type.size_in_floats().unwrap().try_into().unwrap(),
                                WebGlRenderingContext::FLOAT,
                                false,
                                (T::stride() * 4).try_into().unwrap(),
                                (offset * 4).try_into().unwrap(),
                            );
                        });
                    offset += item.data_type.size_in_floats().unwrap();
                }
            },
        );
    }

    pub(self) fn enable_attributes<R, F: FnOnce() -> R>(&self, callback: F) -> R {
        let attributes: Vec<u32> = (&self.data.attributes).iter().map(|v| v.location).collect();
        self.data
            .gl
            .apply(Gl::settings().enabled_attributes(&attributes), callback)
    }

    pub(self) fn set_uniforms<R, F: FnOnce() -> R>(
        &self,
        uniforms: &impl Uniforms,
        callback: F,
    ) -> R {
        let items = uniforms.uniforms();
        let info = &self.data.uniforms;
        let gl = &self.data.gl;
        let context: &WebGlRenderingContext = gl.context();

        let mut textures: Vec<Texture> = Vec::with_capacity(TEXTURES_COUNT.try_into().unwrap());

        gl.apply(Gl::settings().program(self.clone()), || {
            for i in items.iter() {
                info.iter().find(|info| info.name == i.name).map(|info| {
                    let location = Some(&info.location);
                    match &i.value {
                        UniformValue::None => match info.data_type {
                            DataType::Boolean => context.uniform1i(location, 0),
                            DataType::Float => context.uniform1f(location, 0.0),
                            DataType::Vec2 => {
                                context.uniform2f(location, 0.0, 0.0);
                            }
                            DataType::Vec3 => {
                                context.uniform3f(location, 0.0, 0.0, 0.0);
                            }
                            DataType::Vec4 => {
                                context.uniform4f(location, 0.0, 0.0, 0.0, 0.0);
                            }
                            DataType::Mat2 => {
                                let mat = [0.0; 4];
                                context.uniform_matrix2fv_with_f32_array(location, false, &mat)
                            }
                            DataType::Mat3 => {
                                let mat = [0.0; 9];
                                context.uniform_matrix3fv_with_f32_array(location, false, &mat)
                            }
                            DataType::Mat4 => {
                                let mat = [0.0; 16];
                                context.uniform_matrix4fv_with_f32_array(location, false, &mat)
                            }
                            DataType::Sampler => {
                                context.uniform1i(location, -1);
                            }
                        },
                        UniformValue::Boolean(value) => {
                            context.uniform1i(location, if *value { 1 } else { 0 })
                        }
                        UniformValue::Float(value) => context.uniform1f(location, *value),
                        UniformValue::Vec2(value) => context.uniform2fv_with_f32_array(location, value),
                        UniformValue::Vec3(value) => context.uniform3fv_with_f32_array(location, value),
                        UniformValue::Vec4(value) => context.uniform4fv_with_f32_array(location, value),
                        UniformValue::Mat2(value) => {
                            context.uniform_matrix2fv_with_f32_array(location, false, value)
                        }
                        UniformValue::Mat3(value) => {
                            context.uniform_matrix3fv_with_f32_array(location, false, value)
                        }
                        UniformValue::Mat4(value) => {
                            context.uniform_matrix4fv_with_f32_array(location, false, value)
                        }
                        UniformValue::Texture(value) => {
                            context.uniform1i(location, textures.len().try_into().unwrap());
                            textures.push(value.clone())
                        }
                    }
                });
            }
        });

        gl.apply(Gl::settings().texture_list(textures), callback)
    }

    pub fn draw_arrays<T: Item, U: Uniforms>(
        &self,
        primitive_type: PrimitiveType,
        uniforms: &U,
        attributes: &ItemsBuffer<T>,
    ) {
        let gl = &self.data.gl;
        gl.apply(Gl::settings().program(self.clone()), || {
            self.enable_attributes(|| {
                self.set_uniforms(uniforms, || {
                    self.set_attributes(attributes);
                    gl.context().draw_arrays(
                        primitive_type.into(),
                        0,
                        attributes.len().try_into().unwrap(),
                    )
                });
            });
        });
    }

    pub fn draw_instances<T: Item, I: Item, U: Uniforms>(
        &self,
        primitive_type: PrimitiveType,
        uniforms: &U,
        attributes: &ItemsBuffer<T>,
        instances: &ItemsBuffer<I>,
    ) {
        let gl = &self.data.gl;
        gl.apply(Gl::settings().program(self.clone()), || {
            self.enable_attributes(|| {
                self.set_uniforms(uniforms, || {
                    self.set_attributes(attributes);
                    self.set_attributes(instances);
                    gl.instanced_arrays().draw_arrays_instanced_angle(
                        primitive_type.into(),
                        0,
                        attributes.len().try_into().unwrap(),
                        instances.len().try_into().unwrap(),
                    );
                });
            });
        });
    }

    pub fn vertex_source(&self) -> &String {
        &self.data.vertex_shader.source
    }

    pub fn fragment_source(&self) -> &String {
        &self.data.fragment_shader.source
    }
}
