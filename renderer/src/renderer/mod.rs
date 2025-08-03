use std::collections::HashMap;
use glam::Mat4;

#[cfg(feature = "d3d11")] mod d3d11;
#[cfg(feature = "d3d11")] pub use d3d11::*;

#[cfg(feature = "wgpu")] mod wgpu;
#[cfg(feature = "wgpu")] pub use wgpu::*;

// ----------

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub(crate) struct Uniform {
	pub camera_view: Mat4,
	pub camera_view_inv: Mat4,
	pub camera_proj: Mat4,
	pub camera_proj_inv: Mat4,
	pub object: Mat4,
}

// ----------

// partially yoinked from wgpu
bitflags::bitflags! {
	#[repr(transparent)]
	#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
	pub struct TextureUsage: u32 {
		const COPY_SRC = 1 << 0;
		const COPY_DST = 1 << 1;
		const TEXTURE_BINDING = 1 << 2;
		const RENDER_TARGET = 1 << 4;
		const DEPTH_STENCIL = 1 << 5;
	}
}

// yoinked from wgpu, with some values removed
bitflags::bitflags! {
	#[repr(transparent)]
	#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
	pub struct BufferUsage: u32 {
		// const MAP_READ = 1 << 0;
		// const MAP_WRITE = 1 << 1;
		const COPY_SRC = 1 << 2;
		const COPY_DST = 1 << 3;
		const INDEX = 1 << 4;
		const VERTEX = 1 << 5;
		const UNIFORM = 1 << 6;
		// const STORAGE = 1 << 7;
		// const INDIRECT = 1 << 8;
		// const QUERY_RESOLVE = 1 << 9;
		// const BLAS_INPUT = 1 << 10;
		// const TLAS_INPUT = 1 << 11;
	}
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TextureFormat {
	Rgba8Unorm,
	Rgba8UnormSrgb,
	Depth32Float,
}

impl TextureFormat {
	pub(crate) fn bbp(&self) -> u32 {
		use TextureFormat::*;
		
		match self {
			Rgba8Unorm |
			Rgba8UnormSrgb |
			Depth32Float => 32,
		}
	}
}

// yoinked from wgpu
bitflags::bitflags! {
	#[repr(transparent)]
	#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
	pub struct MaterialBindStage: u32 {
		const NONE = 0;
		const VERTEX = 1 << 0;
		const FRAGMENT = 1 << 1;
		const COMPUTE = 1 << 2;
		const VERTEX_FRAGMENT = Self::VERTEX.bits() | Self::FRAGMENT.bits();
		const TASK = 1 << 3;
		const MESH = 1 << 4;
	}
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MaterialBindType {
	Buffer,
	Texture,
	Sampler,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct MaterialBind {
	pub stage: MaterialBindStage,
	pub typ: MaterialBindType,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SamplerAddress {
	ClampToEdge,
	Repeat,
	MirrorRepeat,
	ClampToBorder,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SamplerFilter {
	Nearest,
	Linear,
}

// ----------

pub trait Renderer {
	fn create_material(&self, shader: &str, binds: &[MaterialBind]) -> Box<dyn Material>;
	
	fn create_texture(&self, width: u32, height: u32, format: TextureFormat, usage: TextureUsage) -> Box<dyn Texture>;
	fn create_texture_initialized(&self, width: u32, height: u32, format: TextureFormat, usage: TextureUsage, data: &[u8]) -> Box<dyn Texture> {
		let texture = self.create_texture(width, height, format, usage | TextureUsage::COPY_DST);
		texture.set_data(data);
		texture
	}
	
	fn create_buffer(&self, size: usize, usage: BufferUsage) -> Box<dyn Buffer>;
	
	fn create_sampler(&self, address_u: SamplerAddress, address_v: SamplerAddress, min: SamplerFilter, mag: SamplerFilter) -> Box<dyn Sampler>;
	
	fn render(&self,
		clear_collor: &Option<[f32; 4]>,
		render_target: &Box<dyn Texture>,
		depth_buffer: &Box<dyn Texture>,
		materials: &HashMap<&'static str, Box<dyn Material>>,
		objects: &crate::ObjectBuffer,
		camera: &crate::scene::Camera);
	
	fn register_texture(&self, texture: &Box<dyn Texture>) -> u64;
	
	fn as_any(&self) -> &dyn std::any::Any;
	fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub trait Material {
	fn as_any(&self) -> &dyn std::any::Any;
}

pub trait Texture {
	fn as_any(&self) -> &dyn std::any::Any;
	fn set_data(&self, data: &[u8]);
}

pub trait Buffer {
	fn as_any(&self) -> &dyn std::any::Any;
	fn set_data(&self, data: &[u8]);
}

pub trait Sampler {
	fn as_any(&self) -> &dyn std::any::Any;
}

pub enum ShaderResource {
	Buffer(Box<dyn Buffer>),
	Texture(Box<dyn Texture>),
	Sampler(Box<dyn Sampler>),
}
