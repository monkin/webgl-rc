use std::cell::Cell;
use std::rc::Rc;

use js_sys::{Error, JsString, Uint8Array};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use web_sys::{
    HtmlImageElement, OesTextureHalfFloat, WebGlRenderingContext as Context, WebGlTexture,
};

use super::gl::Gl;
use super::gl::GlError;
use super::settings::Settings;

#[repr(i32)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
pub enum TextureFilter {
    Nearest = Context::NEAREST as i32,
    Linear = Context::LINEAR as i32,
}

impl Default for TextureFilter {
    fn default() -> Self {
        TextureFilter::Linear
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
pub enum TextureType {
    Byte = Context::UNSIGNED_BYTE,
    Float = Context::FLOAT,
    HalfFloat = OesTextureHalfFloat::HALF_FLOAT_OES,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
pub enum TextureFormat {
    Alpha = Context::ALPHA,
    Luminance = Context::LUMINANCE,
    LuminanceAlpha = Context::LUMINANCE_ALPHA,
    Rgb = Context::RGB,
    Rgba = Context::RGBA,
}

impl TextureFormat {
    pub fn channels(self) -> u32 {
        match self {
            TextureFormat::Alpha => 1,
            TextureFormat::Luminance => 1,
            TextureFormat::LuminanceAlpha => 2,
            TextureFormat::Rgb => 3,
            TextureFormat::Rgba => 4,
        }
    }
}

#[derive(Debug)]
pub enum TextureContent {
    None,
    Image(HtmlImageElement),
    Bytes(Vec<u8>),
}

pub const TEXTURES_COUNT: u32 = 16;

#[derive(Debug)]
struct TextureInfo {
    gl: Gl,
    handle: WebGlTexture,
    width: u32,
    height: u32,
    data_type: TextureType,
    format: TextureFormat,
    filter: Cell<TextureFilter>,
}

impl PartialEq<TextureInfo> for TextureInfo {
    fn eq(&self, other: &TextureInfo) -> bool {
        self.handle == other.handle
    }
}

impl Eq for TextureInfo {}

impl Drop for TextureInfo {
    fn drop(&mut self) {
        self.gl.context().delete_texture(Some(&self.handle))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Texture {
    data: Rc<TextureInfo>,
}

impl Texture {
    pub fn new(
        gl: Gl,
        width: u32,
        height: u32,
        data_type: TextureType,
        format: TextureFormat,
        data: TextureContent,
    ) -> Result<Texture, GlError> {
        let handle = gl
            .context()
            .create_texture()
            .ok_or_else(|| GlError::UnknownError(Some("Texture creation failed".into())))?;

        let result = Texture {
            data: Rc::new(TextureInfo {
                gl: gl.clone(),
                handle: handle.clone(),
                filter: Default::default(),
                width,
                height,
                data_type,
                format,
            }),
        };

        gl.apply(
            Gl::settings().active_texture(0).texture(0, result.clone()),
            || {
                gl.context().tex_parameteri(
                    Context::TEXTURE_2D,
                    Context::TEXTURE_WRAP_S,
                    Context::CLAMP_TO_EDGE as i32,
                );
                gl.context().tex_parameteri(
                    Context::TEXTURE_2D,
                    Context::TEXTURE_WRAP_T,
                    Context::CLAMP_TO_EDGE as i32,
                );
                gl.context().tex_parameteri(
                    Context::TEXTURE_2D,
                    Context::TEXTURE_MAG_FILTER,
                    TextureFilter::default().into(),
                );
                gl.context().tex_parameteri(
                    Context::TEXTURE_2D,
                    Context::TEXTURE_MIN_FILTER,
                    TextureFilter::default().into(),
                );
            },
        );

        match data {
            TextureContent::None => result.init_buffer()?,
            TextureContent::Image(image) => result.write_image(&image)?,
            TextureContent::Bytes(bytes) => result.write_bytes(&bytes)?,
        }

        Ok(result)
    }

    pub fn gl(&self) -> Gl {
        self.data.gl.clone()
    }

    pub fn width(&self) -> u32 {
        self.data.width
    }
    pub fn height(&self) -> u32 {
        self.data.height
    }
    pub fn data_type(&self) -> TextureType {
        self.data.data_type
    }
    pub fn format(&self) -> TextureFormat {
        self.data.format
    }

    pub(crate) fn handle(&self) -> &WebGlTexture {
        &self.data.handle
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    pub fn filter(&self) -> TextureFilter {
        self.data.filter.get()
    }

    pub fn set_filter(&self, filter: TextureFilter) {
        if self.filter() != filter {
            let ref gl = self.data.gl;
            let context = gl.context();
            gl.apply(
                Gl::settings().texture(0, self.clone()).active_texture(0),
                || {
                    context.tex_parameteri(
                        Context::TEXTURE_2D,
                        Context::TEXTURE_MAG_FILTER,
                        filter.into(),
                    );
                    context.tex_parameteri(
                        Context::TEXTURE_2D,
                        Context::TEXTURE_MIN_FILTER,
                        filter.into(),
                    );
                    self.data.filter.set(filter);
                },
            );
        }
    }

    pub fn write_image(&self, image: &HtmlImageElement) -> Result<(), GlError> {
        let gl = self.gl();
        let format: u32 = self.format().into();

        gl.apply(
            Gl::settings().active_texture(0).texture(0, self.clone()),
            || {
                gl.context()
                    .tex_image_2d_with_u32_and_u32_and_image(
                        Context::TEXTURE_2D,
                        0,
                        format as i32,
                        format,
                        self.data_type().into(),
                        image,
                    )
                    .map_err(|e| GlError::WritePixelsError(Some(JsString::from(e).into())))
            },
        )?;

        Ok(())
    }

    pub fn write_bytes(&self, bytes: &Vec<u8>) -> Result<(), GlError> {
        let gl = self.gl();
        let format: u32 = self.format().into();

        gl.apply(
            Gl::settings().active_texture(0).texture(0, self.clone()),
            || {
                gl.context()
                    .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                        Context::TEXTURE_2D,
                        0,
                        format as i32,
                        self.width() as i32,
                        self.height() as i32,
                        0,
                        format,
                        self.data_type().into(),
                        Some(bytes),
                    )
                    .map_err(|e| GlError::WritePixelsError(Some(JsString::from(e).into())))
            },
        )?;

        Ok(())
    }

    fn init_buffer(&self) -> Result<(), GlError> {
        let gl = self.gl();
        let format: u32 = self.format().into();

        gl.apply(
            Gl::settings().active_texture(0).texture(0, self.clone()),
            || {
                gl.context()
                    .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                        Context::TEXTURE_2D,
                        0,
                        format as i32,
                        self.width() as i32,
                        self.height() as i32,
                        0,
                        format,
                        self.data_type().into(),
                        None,
                    )
                    .map_err(|e| GlError::InitTextureBufferError(Some(JsString::from(e).into())))
            },
        )?;

        Ok(())
    }

    /// Read RGBA 8-bit data into vector
    pub fn read_pixels_into_array(&self, array: &mut [u8]) -> Result<(), GlError> {
        let size = self.width() * self.height() * 4;
        if array.len() as u32 != size {
            Err(GlError::InvalidBufferSize {
                expected: size,
                received: array.len() as u32,
            })
        } else {
            let gl = self.gl();

            gl.apply(
                Gl::settings().frame_buffer(gl.frame_buffer_with_color(self.clone())?),
                || {
                    gl.context()
                        .read_pixels_with_opt_u8_array(
                            0,
                            0,
                            self.width() as i32,
                            self.height() as i32,
                            TextureFormat::Rgba.into(),
                            TextureType::Byte.into(),
                            Some(array),
                        )
                        .map_err(|error_object| {
                            let error: Error = error_object.into();
                            GlError::ReadPixelsError(Some(error.message().into()))
                        })
                },
            )?;
            Ok(())
        }
    }

    /// Read RGBA 8-bit data into UInt8Array
    pub fn read_pixels_into_buffer(&self, buffer: &Uint8Array) -> Result<(), GlError> {
        if self.data_type() != TextureType::Byte {
            Err(GlError::ReadPixelsError(Some(format!(
                "Invalid texture data type {:?}",
                self.data_type()
            ))))
        } else if buffer.length() != self.width() * self.height() * self.format().channels() {
            Err(GlError::InvalidBufferSize {
                expected: self.width() * self.height() * self.format().channels(),
                received: buffer.length(),
            })
        } else {
            let gl = self.gl();

            gl.apply(
                Gl::settings().frame_buffer(gl.frame_buffer_with_color(self.clone())?),
                || {
                    gl.context()
                        .read_pixels_with_opt_array_buffer_view(
                            0,
                            0,
                            self.width() as i32,
                            self.height() as i32,
                            TextureFormat::Rgba.into(),
                            TextureType::Byte.into(),
                            Some(buffer),
                        )
                        .map_err(|error_object| {
                            let error: Error = error_object.into();
                            GlError::ReadPixelsError(Some(error.message().into()))
                        })
                },
            )?;
            Ok(())
        }
    }

    pub fn read_pixels_array(&self) -> Result<Vec<u8>, GlError> {
        let mut result = Vec::with_capacity((self.width() * self.height() * 4) as usize);
        self.read_pixels_into_array(&mut result)?;
        return Ok(result);
    }

    pub fn read_pixels_buffer(&self) -> Result<Uint8Array, GlError> {
        let result = Uint8Array::new_with_length(self.width() * self.height() * 4);
        self.read_pixels_into_buffer(&result)?;
        return Ok(result);
    }

    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) -> Result<(), GlError> {
        let gl = self.gl();

        gl.apply(
            Gl::settings()
                .clear_color(r, g, b, a)
                .viewport(0, 0, self.width() as i32, self.height() as i32)
                .frame_buffer(gl.frame_buffer_with_color(self.clone())?),
            || gl.clear_color_buffer(),
        );

        Ok(())
    }
}
