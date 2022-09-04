use crate::settings::Settings;
use crate::{BufferUsage, Gl, GlError};
use std::cell::Cell;
use std::rc::Rc;
use web_sys::{WebGlBuffer, WebGlRenderingContext};

#[derive(Debug, Clone)]
pub struct ElementBufferData {
    pub(self) gl: Gl,
    pub(self) handle: WebGlBuffer,
    pub(self) length: Cell<usize>,
}

impl Drop for ElementBufferData {
    fn drop(&mut self) {
        self.gl.context().delete_buffer(Some(&self.handle));
    }
}

#[derive(Debug, Clone)]
pub struct ElementsBuffer {
    pub(self) data: Rc<ElementBufferData>,
}

impl PartialEq<ElementsBuffer> for ElementsBuffer {
    fn eq(&self, other: &ElementsBuffer) -> bool {
        self.data.handle == other.data.handle
    }
}

impl Eq for ElementsBuffer {}

impl ElementsBuffer {
    pub fn new(gl: Gl, data: &[u32], usage: BufferUsage) -> Result<ElementsBuffer, GlError> {
        let ref context: &WebGlRenderingContext = gl.context();
        let buffer = context
            .create_buffer()
            .ok_or(GlError::BufferAllocationError)?;

        let result = ElementsBuffer {
            data: Rc::new(ElementBufferData {
                gl: gl.clone(),
                handle: buffer,
                length: Default::default(),
            }),
        };

        result.set_content(data, usage);

        return Ok(result);
    }

    pub(crate) fn handle(&self) -> WebGlBuffer {
        self.data.handle.clone()
    }

    pub fn set_content(&self, data: &[u32], usage: BufferUsage) {
        self.data
            .gl
            .apply(Gl::settings().element_buffer(self.clone()), || {
                let bytes = unsafe {
                    std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4)
                };
                self.data.gl.context().buffer_data_with_u8_array(
                    WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
                    &bytes,
                    usage.into(),
                );
            });

        self.data.length.set(data.len());
    }

    pub fn len(&self) -> usize {
        self.data.length.get()
    }
}
