use crate::{Gl, GlError, Settings};
use std::rc::Rc;
use web_sys::{WebGlRenderbuffer, WebGlRenderingContext};

#[derive(Clone, Debug)]
struct DepthBufferInfo {
    gl: Gl,
    handle: WebGlRenderbuffer,
    width: u32,
    height: u32,
}

impl Drop for DepthBufferInfo {
    fn drop(&mut self) {
        self.gl.context().delete_renderbuffer(Some(&self.handle));
    }
}

#[derive(Clone, Debug)]
pub struct DepthBuffer {
    data: Rc<DepthBufferInfo>,
}

impl PartialEq for DepthBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.data.handle == other.data.handle
    }
}

impl Eq for DepthBuffer {}

impl DepthBuffer {
    pub fn new(gl: Gl, width: u32, height: u32) -> Result<DepthBuffer, GlError> {
        let handle = gl
            .context()
            .create_renderbuffer()
            .ok_or(GlError::DepthBufferError)?;
        let buffer = DepthBuffer {
            data: Rc::new(DepthBufferInfo {
                gl: gl.clone(),
                handle,
                width,
                height,
            }),
        };
        gl.apply(Gl::settings().depth_buffer(buffer.clone()), || {
            gl.context().renderbuffer_storage(
                WebGlRenderingContext::RENDERBUFFER,
                WebGlRenderingContext::DEPTH_COMPONENT16,
                width as i32,
                height as i32,
            )
        });
        Ok(buffer)
    }

    pub fn width(&self) -> u32 {
        self.data.width
    }
    pub fn height(&self) -> u32 {
        self.data.height
    }

    pub(crate) fn handle(&self) -> &WebGlRenderbuffer {
        &self.data.handle
    }
}
