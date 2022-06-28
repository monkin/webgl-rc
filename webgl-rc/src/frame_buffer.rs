use crate::{DepthBuffer, Gl, GlError, Settings, Texture};
use std::rc::Rc;
use web_sys::{WebGlFramebuffer, WebGlRenderingContext};

#[derive(Clone, Debug)]
struct FrameBufferInfo {
    gl: Gl,
    handle: WebGlFramebuffer,
}

impl Drop for FrameBufferInfo {
    fn drop(&mut self) {
        self.gl.context().delete_framebuffer(Some(&self.handle));
    }
}

#[derive(Clone, Debug)]
pub struct FrameBuffer {
    data: Rc<FrameBufferInfo>,
    color_buffer: Option<Texture>,
    depth_buffer: Option<DepthBuffer>,
}

impl PartialEq for FrameBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.data.handle == other.data.handle
    }
}

impl Eq for FrameBuffer {}

impl FrameBuffer {
    pub fn new(gl: Gl) -> Result<FrameBuffer, GlError> {
        Ok(FrameBuffer {
            data: Rc::new(FrameBufferInfo {
                handle: gl
                    .context()
                    .create_framebuffer()
                    .ok_or(GlError::FrameBufferError)?,
                gl,
            }),
            color_buffer: None,
            depth_buffer: None,
        })
    }
    pub fn set_color_buffer(&mut self, texture: Option<Texture>) -> &mut Self {
        self.color_buffer = texture.clone();
        self.data
            .gl
            .apply(Gl::settings().frame_buffer(self.clone()), || {
                self.data.gl.context().framebuffer_texture_2d(
                    WebGlRenderingContext::FRAMEBUFFER,
                    WebGlRenderingContext::COLOR_ATTACHMENT0,
                    WebGlRenderingContext::TEXTURE_2D,
                    self.color_buffer.as_ref().map(|buffer| buffer.handle()),
                    0,
                );
            });
        self
    }
    pub fn set_depth_buffer(&mut self, buffer: Option<DepthBuffer>) -> &mut Self {
        self.depth_buffer = buffer;
        self.data
            .gl
            .apply(Gl::settings().frame_buffer(self.clone()), || {
                self.data.gl.context().framebuffer_renderbuffer(
                    WebGlRenderingContext::FRAMEBUFFER,
                    WebGlRenderingContext::DEPTH_ATTACHMENT,
                    WebGlRenderingContext::RENDERBUFFER,
                    self.depth_buffer.as_ref().map(|buffer| buffer.handle()),
                );
            });
        self
    }
    pub fn color_buffer(&self) -> Option<Texture> {
        self.color_buffer.clone()
    }
    pub fn depth_buffer(&self) -> Option<DepthBuffer> {
        self.depth_buffer.clone()
    }

    pub(crate) fn handle(&self) -> &WebGlFramebuffer {
        &self.data.handle
    }
}
