use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::cell::RefCell;
use std::convert::TryInto;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;
use web_sys::{AngleInstancedArrays, WebGlRenderingContext as Context};

use super::data_buffer::{ArrayBuffer, Item, ItemsBuffer};
use super::gl::Gl;
use super::program::Program;
use super::texture::Texture;
use super::texture::TextureFilter;
use crate::depth_buffer::DepthBuffer;
use crate::{ElementsBuffer, FrameBuffer};

#[repr(u32)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
pub enum BlendFunction {
    Zero = Context::ZERO,
    One = Context::ONE,
    SrcColor = Context::SRC_COLOR,
    OneMinusSrcColor = Context::ONE_MINUS_SRC_COLOR,
    DstColor = Context::DST_COLOR,
    OneMinusDstColor = Context::ONE_MINUS_DST_COLOR,
    SrcAlpha = Context::SRC_ALPHA,
    OneMinusSrcAlpha = Context::ONE_MINUS_SRC_ALPHA,
    DstAlpha = Context::DST_ALPHA,
    OneMinusDstAlpha = Context::ONE_MINUS_DST_ALPHA,
    SrcAlphaSaturate = Context::SRC_ALPHA_SATURATE,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
pub enum CullFace {
    Front = Context::FRONT,
    Back = Context::BACK,
    FrontAndBack = Context::FRONT_AND_BACK,
}

impl Default for CullFace {
    fn default() -> Self {
        CullFace::Back
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
pub enum BlendEquation {
    Add = Context::FUNC_ADD,
    Sub = Context::FUNC_SUBTRACT,
    RSub = Context::FUNC_REVERSE_SUBTRACT,
}

impl Default for BlendEquation {
    fn default() -> Self {
        BlendEquation::Add
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
pub enum DepthFunction {
    Never = Context::NEVER,
    Less = Context::LESS,
    Equal = Context::EQUAL,
    LEqual = Context::LEQUAL,
    Greater = Context::GREATER,
    NotEqual = Context::NOTEQUAL,
    GEqual = Context::GEQUAL,
    Always = Context::ALWAYS,
}

impl Default for DepthFunction {
    fn default() -> Self {
        DepthFunction::Less
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ColorMask(bool, bool, bool, bool);

impl Default for ColorMask {
    fn default() -> Self {
        Self(true, true, true, true)
    }
}

#[derive(Clone, Debug, Default)]
pub struct SettingsCache {
    blend: BlendSetting,
    depth: DepthTestSetting,
    array_buffer: ArrayBufferSetting,
    element_buffer: ElementBufferSetting,
    active_texture: ActiveTextureSetting,
    textures: [Option<Texture>; 16],
    enabled_attributes: EnabledAttributesSetting,
    instanced_attributes: InstancedAttributesSetting,
    program: ProgramSetting,
    clear_color: ClearColorSetting,
    clear_depth: ClearDepthSetting,
    viewport: ViewportSetting,
    depth_buffer: DepthBufferSetting,
    frame_buffer: FrameBufferSetting,
    blend_equation: BlendEquationSetting,
    blend_function: BlendFunctionSetting,
    depth_function: DepthFunction,
    cull_face: CullFace,
    color_mask: ColorMask,
}

pub trait Settings
where
    Self: PartialEq,
    Self: Debug,
    Self: Clone,
{
    fn apply<R, F: FnOnce() -> R>(&self, gl: &Gl, cache: &RefCell<SettingsCache>, callback: F)
        -> R;

    fn depth_test(self, value: bool) -> ComposedSetting<Self, DepthTestSetting> {
        ComposedSetting(self, DepthTestSetting(value))
    }

    fn blend(self, value: bool) -> ComposedSetting<Self, BlendSetting> {
        ComposedSetting(self, BlendSetting(value))
    }

    fn blend_equation(
        self,
        color: BlendEquation,
        alpha: BlendEquation,
    ) -> ComposedSetting<Self, BlendEquationSetting> {
        ComposedSetting(self, BlendEquationSetting { color, alpha })
    }

    fn blend_function(
        self,
        src_rgb: BlendFunction,
        dst_rgb: BlendFunction,
        src_alpha: BlendFunction,
        dst_alpha: BlendFunction,
    ) -> ComposedSetting<Self, BlendFunctionSetting> {
        ComposedSetting(
            self,
            BlendFunctionSetting {
                src_rgb,
                dst_rgb,
                src_alpha,
                dst_alpha,
            },
        )
    }

    fn depth_function(self, function: DepthFunction) -> ComposedSetting<Self, DepthFunction> {
        ComposedSetting(self, function)
    }

    fn active_texture(self, index: u32) -> ComposedSetting<Self, ActiveTextureSetting> {
        ComposedSetting(self, ActiveTextureSetting(index))
    }

    fn texture(self, index: u32, texture: Texture) -> ComposedSetting<Self, TextureSetting> {
        ComposedSetting(
            self,
            TextureSetting {
                index,
                texture: Some(texture),
            },
        )
    }

    fn texture_list<T: IntoIterator<Item = Texture>>(
        self,
        textures: T,
    ) -> ComposedSetting<Self, TextureListSetting> {
        let mut setting: [Option<Texture>; 16] = Default::default();
        for (i, texture) in textures.into_iter().enumerate() {
            setting[i] = Some(texture);
        }
        ComposedSetting(self, TextureListSetting { textures: setting })
    }

    fn texture_filter(
        self,
        texture: Texture,
        filter: TextureFilter,
    ) -> ComposedSetting<Self, TextureFilterSetting> {
        ComposedSetting(self, TextureFilterSetting { texture, filter })
    }

    fn array_buffer(self, array_buffer: ArrayBuffer) -> ComposedSetting<Self, ArrayBufferSetting> {
        ComposedSetting(self, ArrayBufferSetting(Some(array_buffer)))
    }

    fn items_buffer<T: Item>(
        self,
        array_buffer: ItemsBuffer<T>,
    ) -> ComposedSetting<Self, ArrayBufferSetting> {
        ComposedSetting(self, ArrayBufferSetting(Some(array_buffer.buffer)))
    }

    fn element_buffer(
        self,
        element_buffer: ElementsBuffer,
    ) -> ComposedSetting<Self, ElementBufferSetting> {
        ComposedSetting(self, ElementBufferSetting(Some(element_buffer)))
    }

    fn enabled_attributes(
        self,
        attributes: &[u32],
    ) -> ComposedSetting<Self, EnabledAttributesSetting> {
        ComposedSetting(
            self,
            EnabledAttributesSetting {
                items: attributes.into(),
            },
        )
    }

    fn program(self, program: Program) -> ComposedSetting<Self, ProgramSetting> {
        ComposedSetting(
            self,
            ProgramSetting {
                program: Some(program),
            },
        )
    }

    fn clear_color(
        self,
        r: f32,
        g: f32,
        b: f32,
        alpha: f32,
    ) -> ComposedSetting<Self, ClearColorSetting> {
        ComposedSetting(
            self,
            ClearColorSetting {
                color: [r, g, b, alpha],
            },
        )
    }

    fn clear_depth(self, value: f32) -> ComposedSetting<Self, ClearDepthSetting> {
        ComposedSetting(self, ClearDepthSetting { value })
    }

    fn viewport(
        self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> ComposedSetting<Self, ViewportSetting> {
        ComposedSetting(
            self,
            ViewportSetting {
                x,
                y,
                width,
                height,
            },
        )
    }

    fn depth_buffer(self, buffer: DepthBuffer) -> ComposedSetting<Self, DepthBufferSetting> {
        ComposedSetting(
            self,
            DepthBufferSetting {
                buffer: Some(buffer),
            },
        )
    }

    fn frame_buffer(self, buffer: FrameBuffer) -> ComposedSetting<Self, FrameBufferSetting> {
        ComposedSetting(
            self,
            FrameBufferSetting {
                buffer: Some(buffer),
            },
        )
    }

    fn cull_face(self, cull_face: CullFace) -> ComposedSetting<Self, CullFace> {
        ComposedSetting(self, cull_face)
    }

    fn color_mask(self, r: bool, g: bool, b: bool, a: bool) -> ComposedSetting<Self, ColorMask> {
        ComposedSetting(self, ColorMask(r, g, b, a))
    }
}

pub trait CachedSettings {
    fn set(gl: &Gl, value: &Self);
    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self;
    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self);
}

impl<T> Settings for T
where
    T: PartialEq,
    T: Debug,
    T: Clone,
    T: CachedSettings,
{
    fn apply<R, F: FnOnce() -> R>(
        &self,
        gl: &Gl,
        cache: &RefCell<SettingsCache>,
        callback: F,
    ) -> R {
        let old_value = Self::read_cached(&cache.borrow());
        return if self == &old_value {
            callback()
        } else {
            Self::write_cached(&mut cache.borrow_mut(), self);
            Self::set(gl, self);
            let result = callback();
            Self::set(gl, &old_value);
            Self::write_cached(&mut cache.borrow_mut(), &old_value);
            result
        };
    }
}

#[derive(Default, PartialEq, Debug, Clone)]
pub struct EmptySetting {}

impl Settings for EmptySetting {
    fn apply<R, F: FnOnce() -> R>(&self, _: &Gl, _: &RefCell<SettingsCache>, callback: F) -> R {
        callback()
    }
}

#[derive(Default, PartialEq, Debug, Clone)]
pub struct ComposedSetting<S1: Settings, S2: Settings>(S1, S2);

impl<S1: Settings, S2: Settings> Settings for ComposedSetting<S1, S2> {
    fn apply<R, F: FnOnce() -> R>(
        &self,
        gl: &Gl,
        cache: &RefCell<SettingsCache>,
        callback: F,
    ) -> R {
        self.0
            .apply(gl, cache, || self.1.apply(gl, cache, || callback()))
    }
}
#[derive(Default, PartialEq, Debug, Clone, Copy)]
pub struct ClearColorSetting {
    color: [f32; 4],
}

impl CachedSettings for ClearColorSetting {
    fn set(gl: &Gl, value: &Self) {
        gl.context().clear_color(
            value.color[0],
            value.color[1],
            value.color[2],
            value.color[3],
        );
    }

    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.clear_color
    }

    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.clear_color = *value;
    }
}

#[derive(Default, PartialEq, Debug, Clone, Copy)]
pub struct ClearDepthSetting {
    value: f32,
}

impl CachedSettings for ClearDepthSetting {
    fn set(gl: &Gl, value: &Self) {
        gl.context().clear_depth(value.value);
    }

    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.clear_depth
    }

    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.clear_depth = *value;
    }
}

#[derive(Default, PartialEq, Debug, Clone, Copy)]
pub struct ViewportSetting {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl CachedSettings for ViewportSetting {
    fn set(gl: &Gl, value: &Self) {
        gl.context()
            .viewport(value.x, value.y, value.width, value.height);
    }

    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.viewport
    }

    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.viewport = *value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActiveTextureSetting(u32);

impl CachedSettings for ActiveTextureSetting {
    fn set(gl: &Gl, value: &Self) {
        gl.context().active_texture(value.0 + Context::TEXTURE0);
    }
    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.active_texture
    }
    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.active_texture = *value;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ArrayBufferSetting(Option<ArrayBuffer>);

impl CachedSettings for ArrayBufferSetting {
    fn set(gl: &Gl, value: &Self) {
        gl.context().bind_buffer(
            Context::ARRAY_BUFFER,
            value.0.as_ref().map(|v| v.handle()).as_ref(),
        );
    }
    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.array_buffer.clone()
    }
    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.array_buffer = value.clone();
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ElementBufferSetting(Option<ElementsBuffer>);

impl CachedSettings for ElementBufferSetting {
    fn set(gl: &Gl, value: &Self) {
        gl.context().bind_buffer(
            Context::ELEMENT_ARRAY_BUFFER,
            value.0.as_ref().map(|v| v.handle()).as_ref(),
        );
    }
    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.element_buffer.clone()
    }
    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.element_buffer = value.clone();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlendSetting(bool);

impl CachedSettings for BlendSetting {
    fn set(gl: &Gl, value: &Self) {
        if value.0 {
            gl.context().enable(Context::BLEND)
        } else {
            gl.context().disable(Context::BLEND)
        }
    }
    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.blend
    }
    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.blend = *value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DepthTestSetting(bool);

impl CachedSettings for DepthTestSetting {
    fn set(gl: &Gl, value: &Self) {
        if value.0 {
            gl.context().enable(Context::DEPTH_TEST)
        } else {
            gl.context().disable(Context::DEPTH_TEST)
        }
    }
    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.depth
    }
    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.depth = *value;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextureSetting {
    index: u32,
    texture: Option<Texture>,
}

impl TextureSetting {
    pub(self) fn set_texture(gl: &Gl, index: u32, texture: Option<&Texture>) {
        gl.apply(Gl::settings().active_texture(index), || {
            gl.context()
                .bind_texture(Context::TEXTURE_2D, texture.map(|texture| texture.handle()));
        })
    }
}

impl Settings for TextureSetting {
    fn apply<R, F: FnOnce() -> R>(
        &self,
        gl: &Gl,
        cache: &RefCell<SettingsCache>,
        callback: F,
    ) -> R {
        let previous = cache.borrow().textures[self.index as usize].clone();
        cache.borrow_mut().textures[self.index as usize] = self.texture.clone();
        Self::set_texture(gl, self.index, self.texture.as_ref());
        let result = callback();
        Self::set_texture(gl, self.index, previous.as_ref());
        cache.borrow_mut().textures[self.index as usize] = previous;
        return result;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextureListSetting {
    textures: [Option<Texture>; 16],
}

impl TextureListSetting {
    pub(self) fn set_textures(
        gl: &Gl,
        current: &[Option<Texture>; 16],
        target: &[Option<Texture>; 16],
    ) {
        for i in 0..16 {
            if current[i] != target[i] {
                TextureSetting::set_texture(gl, i.try_into().unwrap(), target[i].as_ref());
            }
        }
    }
}

impl Settings for TextureListSetting {
    fn apply<R, F: FnOnce() -> R>(
        &self,
        gl: &Gl,
        cache: &RefCell<SettingsCache>,
        callback: F,
    ) -> R {
        let previous = cache.borrow().textures.clone();

        cache.borrow_mut().textures = self.textures.clone();
        TextureListSetting::set_textures(gl, &previous, &self.textures);

        let result = callback();

        TextureListSetting::set_textures(gl, &self.textures, &previous);
        cache.borrow_mut().textures = previous;

        return result;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureFilterSetting {
    texture: Texture,
    filter: TextureFilter,
}

impl Settings for TextureFilterSetting {
    fn apply<R, F: FnOnce() -> R>(&self, _: &Gl, _: &RefCell<SettingsCache>, callback: F) -> R {
        let previous = self.texture.filter();
        let current = self.filter;
        self.texture.set_filter(current);
        let result = callback();
        self.texture.set_filter(previous);
        return result;
    }
}

fn array_diff<'a, T: PartialEq>(v1: &'a Vec<T>, v2: &'a Vec<T>) -> impl Iterator<Item = &'a T> {
    v1.iter().filter(move |i| !v2.contains(i))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnabledAttributesSetting {
    items: Vec<u32>,
}

impl Settings for EnabledAttributesSetting {
    fn apply<R, F: FnOnce() -> R>(
        &self,
        gl: &Gl,
        cache: &RefCell<SettingsCache>,
        callback: F,
    ) -> R {
        let context: &Context = gl.context();
        // get old value
        let previous = { cache.borrow().enabled_attributes.clone() };

        // set current value
        {
            cache.borrow_mut().enabled_attributes = self.clone();
        }

        // disable extra attributes
        array_diff(&previous.items, &self.items).for_each(|i| {
            context.disable_vertex_attrib_array(*i);
        });

        // enable disabled attributes
        array_diff(&self.items, &previous.items).for_each(|i| {
            context.enable_vertex_attrib_array(*i);
        });

        // do the stuff
        let result = callback();

        // rollback changes
        array_diff(&previous.items, &self.items).for_each(|i| {
            context.enable_vertex_attrib_array(*i);
        });

        array_diff(&self.items, &previous.items).for_each(|i| {
            context.disable_vertex_attrib_array(*i);
        });

        {
            cache.borrow_mut().enabled_attributes = previous;
        }

        return result;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstancedAttributesSetting {
    items: Vec<u32>,
}

impl Settings for InstancedAttributesSetting {
    fn apply<R, F: FnOnce() -> R>(
        &self,
        gl: &Gl,
        cache: &RefCell<SettingsCache>,
        callback: F,
    ) -> R {
        let context: &AngleInstancedArrays = gl.instanced_arrays();
        // get old value
        let previous = { cache.borrow().instanced_attributes.clone() };

        // set current value
        {
            cache.borrow_mut().instanced_attributes = self.clone();
        }

        // disable instancing
        array_diff(&previous.items, &self.items).for_each(|i| {
            context.vertex_attrib_divisor_angle(*i, 0);
        });

        // enable instancing
        array_diff(&self.items, &previous.items).for_each(|i| {
            context.vertex_attrib_divisor_angle(*i, 1);
        });

        // do the stuff
        let result = callback();

        // rollback changes
        array_diff(&previous.items, &self.items).for_each(|i| {
            context.vertex_attrib_divisor_angle(*i, 1);
        });

        array_diff(&self.items, &previous.items).for_each(|i| {
            context.vertex_attrib_divisor_angle(*i, 0);
        });

        {
            cache.borrow_mut().instanced_attributes = previous;
        }

        return result;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProgramSetting {
    program: Option<Program>,
}

impl CachedSettings for ProgramSetting {
    fn set(gl: &Gl, value: &Self) {
        gl.context().use_program(
            value
                .program
                .as_ref()
                .map(|program| program.handle())
                .as_ref(),
        );
    }

    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.program.clone()
    }

    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.program = value.clone();
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DepthBufferSetting {
    buffer: Option<DepthBuffer>,
}

impl CachedSettings for DepthBufferSetting {
    fn set(gl: &Gl, value: &Self) {
        gl.context().bind_renderbuffer(
            Context::RENDERBUFFER,
            value.buffer.as_ref().map(|v| v.handle()),
        );
    }

    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.depth_buffer.clone()
    }

    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.depth_buffer = value.clone();
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FrameBufferSetting {
    buffer: Option<FrameBuffer>,
}

impl CachedSettings for FrameBufferSetting {
    fn set(gl: &Gl, value: &Self) {
        gl.context().bind_framebuffer(
            Context::FRAMEBUFFER,
            value.buffer.as_ref().map(|v| v.handle()),
        );
    }

    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.frame_buffer.clone()
    }

    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.frame_buffer = value.clone();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlendEquationSetting {
    color: BlendEquation,
    alpha: BlendEquation,
}

impl CachedSettings for BlendEquationSetting {
    fn set(gl: &Gl, value: &Self) {
        gl.context()
            .blend_equation_separate(value.color.into(), value.alpha.into());
    }

    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.blend_equation
    }

    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.blend_equation = *value;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlendFunctionSetting {
    src_rgb: BlendFunction,
    dst_rgb: BlendFunction,
    src_alpha: BlendFunction,
    dst_alpha: BlendFunction,
}

impl Default for BlendFunctionSetting {
    fn default() -> Self {
        BlendFunctionSetting {
            src_rgb: BlendFunction::One,
            dst_rgb: BlendFunction::Zero,
            src_alpha: BlendFunction::One,
            dst_alpha: BlendFunction::Zero,
        }
    }
}

impl CachedSettings for BlendFunctionSetting {
    fn set(gl: &Gl, value: &Self) {
        gl.context().blend_func_separate(
            value.src_rgb.into(),
            value.dst_rgb.into(),
            value.src_alpha.into(),
            value.dst_alpha.into(),
        );
    }

    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.blend_function
    }

    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.blend_function = *value;
    }
}

impl CachedSettings for DepthFunction {
    fn set(gl: &Gl, value: &Self) {
        gl.context().depth_func((*value).into());
    }

    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.depth_function
    }

    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.depth_function = *value;
    }
}

impl CachedSettings for CullFace {
    fn set(gl: &Gl, value: &Self) {
        gl.context().cull_face((*value).into());
    }

    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.cull_face
    }

    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.cull_face = *value;
    }
}

impl CachedSettings for ColorMask {
    fn set(gl: &Gl, value: &Self) {
        gl.context().color_mask(value.0, value.1, value.2, value.3);
    }

    fn read_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.color_mask
    }

    fn write_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.color_mask = *value;
    }
}
