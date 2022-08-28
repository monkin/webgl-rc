use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;
use web_sys::{WebGlBuffer, WebGlRenderingContext as Context, WebGlRenderingContext};
use crate::buffer_usage::BufferUsage;

use super::gl::{Gl, GlError};
use super::settings::Settings;
use super::types::DataType;

pub trait Writable: Copy {
    fn write(&self, output: &mut Vec<f32>);
    fn stride() -> usize;
}

#[derive(Debug, Clone)]
pub struct ArrayBufferData {
    pub(self) gl: Gl,
    pub(self) handle: WebGlBuffer,
    pub(self) length: Cell<usize>,
}

impl Drop for ArrayBufferData {
    fn drop(&mut self) {
        self.gl.context().delete_buffer(Some(&self.handle));
    }
}

#[derive(Debug, Clone)]
pub struct ArrayBuffer {
    pub(self) data: Rc<ArrayBufferData>,
}

impl PartialEq<ArrayBuffer> for ArrayBuffer {
    fn eq(&self, other: &ArrayBuffer) -> bool {
        self.data.handle == other.data.handle
    }
}

impl Eq for ArrayBuffer {}

impl ArrayBuffer {
    pub fn new<T: Writable>(
        gl: Gl,
        data: &[T],
        usage: BufferUsage,
    ) -> Result<ArrayBuffer, GlError> {
        let ref context: &WebGlRenderingContext = gl.context();
        let buffer = context
            .create_buffer()
            .ok_or(GlError::BufferAllocationError)?;

        let result = ArrayBuffer {
            data: Rc::new(ArrayBufferData {
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

    pub fn set_content<T: Writable>(&self, items: &[T], usage: BufferUsage) {
        let mut data: Vec<f32> = Vec::with_capacity(T::stride() * items.len());
        for i in items {
            i.write(&mut data);
        }

        self.data
            .gl
            .apply(Gl::settings().array_buffer(self.clone()), || {
                let bytes = unsafe {
                    std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4)
                };
                self.data.gl.context().buffer_data_with_u8_array(
                    Context::ARRAY_BUFFER,
                    &bytes,
                    usage.into(),
                );
            });

        self.data.length.set(items.len());
    }

    pub fn len(&self) -> usize {
        self.data.length.get()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Layout {
    pub name: &'static str,
    pub data_type: DataType,
}

pub trait Item: Writable {
    fn layout() -> Vec<Layout>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemsBuffer<T: Item> {
    pub(self) phantom: PhantomData<T>,
    pub(crate) buffer: ArrayBuffer,
}

impl<T: Item> ItemsBuffer<T> {
    pub fn new(gl: Gl, data: &[T], usage: BufferUsage) -> Result<ItemsBuffer<T>, GlError> {
        Ok(ItemsBuffer {
            phantom: Default::default(),
            buffer: ArrayBuffer::new(gl, data, usage)?,
        })
    }

    pub fn set_content(&self, items: &[T], usage: BufferUsage) {
        self.buffer.set_content(items, usage);
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}
