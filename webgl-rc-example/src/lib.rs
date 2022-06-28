#![allow(dead_code)]

use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use webgl_rc::{
    load_glsl, Attributes, BufferUsage, Gl, ItemsBuffer, PrimitiveType, Program, Settings, Uniforms,
};

mod utils;

#[derive(Clone, Copy, Uniforms)]
pub struct TriangleUniforms {
    pub time: f32,
}

#[derive(Clone, Copy, Attributes)]
pub struct TriangleAttributes {
    pub position: (f32, f32),
    pub color: (f32, f32, f32),
}

#[wasm_bindgen]
pub struct TriangleContext {
    gl: Gl,
    points: ItemsBuffer<TriangleAttributes>,
    program: Program,
}

#[wasm_bindgen]
pub fn create_context(canvas: &HtmlCanvasElement) -> TriangleContext {
    let gl = Gl::new(canvas).unwrap();
    let program = gl
        .program(load_glsl!("fragment.glsl"), load_glsl!("vertex.glsl"))
        .unwrap();
    let points = gl
        .items_buffer(
            &[
                TriangleAttributes {
                    position: (0.0, 1.0),
                    color: (1.0, 0.0, 0.0),
                },
                TriangleAttributes {
                    position: (1.0, -1.0),
                    color: (0.0, 1.0, 0.0),
                },
                TriangleAttributes {
                    position: (-1.0, -1.0),
                    color: (0.0, 0.0, 1.0),
                },
            ],
            BufferUsage::Static,
        )
        .unwrap();

    TriangleContext {
        gl,
        program,
        points,
    }
}

#[wasm_bindgen]
pub fn draw_triangle(context: &TriangleContext, width: i32, height: i32) {
    let gl = &context.gl;
    gl.apply(
        Gl::settings()
            .clear_color(1.0, 1.0, 1.0, 1.0)
            .viewport(0, 0, width, height),
        || {
            gl.clear_color_buffer();
            context.program.draw_arrays(
                PrimitiveType::Triangles,
                &TriangleUniforms { time: 0.0 },
                &context.points,
            );
        },
    );
}
