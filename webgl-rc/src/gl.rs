use js_sys::JsString;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    AngleInstancedArrays, ExtColorBufferHalfFloat, HtmlCanvasElement, OesTextureHalfFloat,
    OesTextureHalfFloatLinear, WebGlRenderingContext as Context,
};

use super::data_buffer::{BufferUsage, ItemsBuffer};
use super::program::Program;
use super::settings::{EmptySetting, Settings, SettingsCache};
use super::texture::{Texture, TextureContent, TextureFormat, TextureType};
use crate::{DepthBuffer, FrameBuffer};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GlError {
    UnknownError(Option<String>),
    ShaderCompilationError {
        source: String,
        info: Option<String>,
    },
    ProgramLinkingError {
        vertex: String,
        fragment: String,
        info: Option<String>,
    },
    ExtensionNotFound(String),
    UnsupportedType(Option<String>),
    BufferAllocationError,
    ReadPixelsError(Option<String>),
    WritePixelsError(Option<String>),
    InitTextureBufferError(Option<String>),
    InvalidBufferSize {
        expected: u32,
        received: u32,
    },
    DepthBufferError,
    FrameBufferError,
}

impl From<GlError> for js_sys::Error {
    fn from(error: GlError) -> Self {
        js_sys::Error::new(&format!("{:?}", error))
    }
}

impl From<GlError> for JsValue {
    fn from(error: GlError) -> Self {
        let js_error: js_sys::Error = error.into();
        js_error.into()
    }
}

impl Into<GlError> for js_sys::Error {
    fn into(self) -> GlError {
        GlError::UnknownError(Some(self.message().into()))
    }
}

impl Into<GlError> for JsValue {
    fn into(self) -> GlError {
        let error: js_sys::Error = self.into();
        error.into()
    }
}

#[derive(Debug)]
pub(self) struct GlInfo {
    pub(crate) context: Context,
    pub(self) settings_cache: RefCell<SettingsCache>,
    pub(self) ex_instanced_arrays: AngleInstancedArrays,
    pub(self) ex_color_buffer_half_float: ExtColorBufferHalfFloat,
    pub(self) ex_texture_half_float: OesTextureHalfFloat,
    pub(self) ex_texture_half_float_linear: OesTextureHalfFloatLinear,
}

#[derive(Clone, Debug)]
pub struct Gl {
    data: Rc<GlInfo>,
}

impl Gl {
    fn get_extension<Ex: JsCast>(context: &Context, name: &str) -> Result<Ex, GlError> {
        context
            .get_extension(name)
            .map_err(|_| GlError::ExtensionNotFound(name.into()))
            .map(|value| match value {
                Some(extension) => Ok(Ex::unchecked_from_js(extension.into())),
                None => Err(GlError::ExtensionNotFound(name.into())),
            })
            .unwrap_or_else(|error| Err(error))
    }

    pub fn new(canvas: &HtmlCanvasElement) -> Result<Gl, GlError> {
        let context = canvas
            .get_context("webgl")
            .map_err(|err| GlError::UnknownError(Some(JsString::from(err).into())))?
            .map(|context| Context::from(JsValue::from(context)))
            .ok_or_else(|| GlError::UnknownError(None))?;

        Ok(Gl {
            data: Rc::new(GlInfo {
                ex_instanced_arrays: Gl::get_extension(&context, "ANGLE_instanced_arrays")?,
                ex_color_buffer_half_float: Gl::get_extension(
                    &context,
                    "EXT_color_buffer_half_float",
                )?,
                ex_texture_half_float: Gl::get_extension(&context, "OES_texture_half_float")?,
                ex_texture_half_float_linear: Gl::get_extension(
                    &context,
                    "OES_texture_half_float_linear",
                )?,
                settings_cache: Default::default(),
                context,
            }),
        })
    }

    pub fn context(&self) -> &Context {
        &self.data.context
    }

    pub fn instanced_arrays(&self) -> &AngleInstancedArrays {
        &self.data.ex_instanced_arrays
    }

    pub fn settings() -> impl Settings {
        EmptySetting {}
    }

    pub fn apply<R>(&self, settings: impl Settings, callback: impl FnOnce() -> R) -> R {
        settings.apply(self, &self.data.settings_cache, callback)
    }

    pub fn program(&self, fragment: &str, vertex: &str) -> Result<Program, GlError> {
        Program::new(self.clone(), fragment, vertex)
    }

    pub fn items_buffer<I>(&self, data: &[I], usage: BufferUsage) -> Result<ItemsBuffer<I>, GlError>
    where
        I: super::data_buffer::Item,
    {
        ItemsBuffer::new(self.clone(), data, usage)
    }

    pub fn clear_color_buffer(&self) {
        self.context().clear(Context::COLOR_BUFFER_BIT);
    }

    pub fn clear_depth_buffer(&self) {
        self.context().clear(Context::DEPTH_BUFFER_BIT);
    }

    pub fn clear_buffers(&self) {
        self.context()
            .clear(Context::COLOR_BUFFER_BIT | Context::DEPTH_BUFFER_BIT);
    }

    pub fn texture(
        &self,
        width: u32,
        height: u32,
        data_type: TextureType,
        format: TextureFormat,
        data: TextureContent,
    ) -> Result<Texture, GlError> {
        Texture::new(self.clone(), width, height, data_type, format, data)
    }

    pub fn depth_buffer(&self, width: u32, height: u32) -> Result<DepthBuffer, GlError> {
        DepthBuffer::new(self.clone(), width, height)
    }

    pub fn frame_buffer(&self) -> Result<FrameBuffer, GlError> {
        FrameBuffer::new(self.clone())
    }

    pub fn frame_buffer_with_color(&self, texture: Texture) -> Result<FrameBuffer, GlError> {
        let mut result = FrameBuffer::new(self.clone())?;
        result.set_color_buffer(Some(texture));
        Ok(result)
    }

    pub fn frame_buffer_with_depth(
        &self,
        texture: Texture,
        depth_buffer: DepthBuffer,
    ) -> Result<FrameBuffer, GlError> {
        let mut result = FrameBuffer::new(self.clone())?;
        result.set_color_buffer(Some(texture));
        result.set_depth_buffer(Some(depth_buffer));
        Ok(result)
    }
}
