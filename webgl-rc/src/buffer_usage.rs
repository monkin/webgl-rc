use num_enum::{  IntoPrimitive,  TryFromPrimitive};
use web_sys::WebGlRenderingContext as Context;

#[repr(u32)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
pub enum BufferUsage {
    /// The data store contents will be modified once and used at most a few times.
    Stream = Context::STREAM_DRAW,
    /// The data store contents will be modified once and used many times.
    Static = Context::STATIC_DRAW,
    /// The data store contents will be modified repeatedly and used many times.
    Dynamic = Context::DYNAMIC_DRAW,
}
