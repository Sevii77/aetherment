use crate::{renderer::*, vertex, Vertex};

pub struct Mesh {
	visible: bool,
	matrix: glam::Mat4,
	material: &'static str,
	vertex_buffer: Buffer,
	index_buffer: Buffer,
	index_count: u32,
	shader_resources: Vec<ShaderResource>,
}

impl Mesh {
	pub fn new_buffer(renderer: &Renderer, vertices: Buffer, indices: Buffer) -> Self {
		Self {
			visible: true,
			matrix: glam::Mat4::IDENTITY,
			material: "3dmesh_lit",
			vertex_buffer: vertices,
			index_count: (indices.size() / 2) as u32,
			index_buffer: indices,
			shader_resources: vec![
				ShaderResource::Texture(renderer.create_texture_initialized(1, 1, TextureFormat::Rgba8Unorm, TextureUsage::TEXTURE_BINDING, &[255; 4])),
				ShaderResource::Sampler(renderer.create_sampler(SamplerAddress::Repeat, SamplerAddress::Repeat, SamplerFilter::Linear, SamplerFilter::Linear)),
				ShaderResource::Texture(renderer.create_texture_initialized(1, 1, TextureFormat::Rgba8Unorm, TextureUsage::TEXTURE_BINDING, &[128, 128, 255, 255])),
				ShaderResource::Sampler(renderer.create_sampler(SamplerAddress::Repeat, SamplerAddress::Repeat, SamplerFilter::Linear, SamplerFilter::Linear)),
			]
		}
	}
	pub fn new(renderer: &Renderer, vertices: &[Vertex], indices: &[u16]) -> Self {
		Self::new_buffer(renderer, create_vertex_buffer(renderer, vertices), create_index_buffer(renderer, indices))
	}
	
	pub fn new_test_cube(renderer: &Renderer) -> Self {
		let vertices = &mut [
			vertex(glam::vec3(-1.0, -1.0,  1.0), glam::vec3( 0.0,  0.0,  1.0), glam::vec4(1.0, 0.0, 0.0, 1.0), glam::vec2(0.0, 0.0)),
			vertex(glam::vec3( 1.0, -1.0,  1.0), glam::vec3( 0.0,  0.0,  1.0), glam::vec4(1.0, 0.0, 0.0, 1.0), glam::vec2(1.0, 0.0)),
			vertex(glam::vec3( 1.0,  1.0,  1.0), glam::vec3( 0.0,  0.0,  1.0), glam::vec4(1.0, 0.0, 0.0, 1.0), glam::vec2(1.0, 1.0)),
			vertex(glam::vec3(-1.0,  1.0,  1.0), glam::vec3( 0.0,  0.0,  1.0), glam::vec4(1.0, 0.0, 0.0, 1.0), glam::vec2(0.0, 1.0)),
			
			vertex(glam::vec3(-1.0,  1.0, -1.0), glam::vec3( 0.0,  0.0, -1.0), glam::vec4(0.0, 1.0, 0.0, 1.0), glam::vec2(1.0, 0.0)),
			vertex(glam::vec3( 1.0,  1.0, -1.0), glam::vec3( 0.0,  0.0, -1.0), glam::vec4(0.0, 1.0, 0.0, 1.0), glam::vec2(0.0, 0.0)),
			vertex(glam::vec3( 1.0, -1.0, -1.0), glam::vec3( 0.0,  0.0, -1.0), glam::vec4(0.0, 1.0, 0.0, 1.0), glam::vec2(0.0, 1.0)),
			vertex(glam::vec3(-1.0, -1.0, -1.0), glam::vec3( 0.0,  0.0, -1.0), glam::vec4(0.0, 1.0, 0.0, 1.0), glam::vec2(1.0, 1.0)),
			
			vertex(glam::vec3( 1.0, -1.0, -1.0), glam::vec3( 1.0,  0.0,  0.0), glam::vec4(0.0, 0.0, 1.0, 1.0), glam::vec2(0.0, 0.0)),
			vertex(glam::vec3( 1.0,  1.0, -1.0), glam::vec3( 1.0,  0.0,  0.0), glam::vec4(0.0, 0.0, 1.0, 1.0), glam::vec2(1.0, 0.0)),
			vertex(glam::vec3( 1.0,  1.0,  1.0), glam::vec3( 1.0,  0.0,  0.0), glam::vec4(0.0, 0.0, 1.0, 1.0), glam::vec2(1.0, 1.0)),
			vertex(glam::vec3( 1.0, -1.0,  1.0), glam::vec3( 1.0,  0.0,  0.0), glam::vec4(0.0, 0.0, 1.0, 1.0), glam::vec2(0.0, 1.0)),
			
			vertex(glam::vec3(-1.0, -1.0,  1.0), glam::vec3(-1.0,  0.0,  0.0), glam::vec4(1.0, 1.0, 0.0, 1.0), glam::vec2(1.0, 0.0)),
			vertex(glam::vec3(-1.0,  1.0,  1.0), glam::vec3(-1.0,  0.0,  0.0), glam::vec4(1.0, 1.0, 0.0, 1.0), glam::vec2(0.0, 0.0)),
			vertex(glam::vec3(-1.0,  1.0, -1.0), glam::vec3(-1.0,  0.0,  0.0), glam::vec4(1.0, 1.0, 0.0, 1.0), glam::vec2(0.0, 1.0)),
			vertex(glam::vec3(-1.0, -1.0, -1.0), glam::vec3(-1.0,  0.0,  0.0), glam::vec4(1.0, 1.0, 0.0, 1.0), glam::vec2(1.0, 1.0)),
			
			vertex(glam::vec3( 1.0,  1.0, -1.0), glam::vec3( 0.0,  1.0,  0.0), glam::vec4(0.0, 1.0, 1.0, 1.0), glam::vec2(1.0, 0.0)),
			vertex(glam::vec3(-1.0,  1.0, -1.0), glam::vec3( 0.0,  1.0,  0.0), glam::vec4(0.0, 1.0, 1.0, 1.0), glam::vec2(0.0, 0.0)),
			vertex(glam::vec3(-1.0,  1.0,  1.0), glam::vec3( 0.0,  1.0,  0.0), glam::vec4(0.0, 1.0, 1.0, 1.0), glam::vec2(0.0, 1.0)),
			vertex(glam::vec3( 1.0,  1.0,  1.0), glam::vec3( 0.0,  1.0,  0.0), glam::vec4(0.0, 1.0, 1.0, 1.0), glam::vec2(1.0, 1.0)),
			
			vertex(glam::vec3( 1.0, -1.0,  1.0), glam::vec3( 0.0, -1.0,  0.0), glam::vec4(1.0, 0.0, 1.0, 1.0), glam::vec2(0.0, 0.0)),
			vertex(glam::vec3(-1.0, -1.0,  1.0), glam::vec3( 0.0, -1.0,  0.0), glam::vec4(1.0, 0.0, 1.0, 1.0), glam::vec2(1.0, 0.0)),
			vertex(glam::vec3(-1.0, -1.0, -1.0), glam::vec3( 0.0, -1.0,  0.0), glam::vec4(1.0, 0.0, 1.0, 1.0), glam::vec2(1.0, 1.0)),
			vertex(glam::vec3( 1.0, -1.0, -1.0), glam::vec3( 0.0, -1.0,  0.0), glam::vec4(1.0, 0.0, 1.0, 1.0), glam::vec2(0.0, 1.0)),
		];
		
		let indices = &[
			0, 1, 2, 2, 3, 0,
			4, 5, 6, 6, 7, 4,
			8, 9, 10, 10, 11, 8,
			12, 13, 14, 14, 15, 12,
			16, 17, 18, 18, 19, 16,
			20, 21, 22, 22, 23, 20,
		];
		
		crate::calculate_tangents(vertices, indices);
		
		Self::new(renderer, vertices, indices)
	}
	
	pub(crate) fn create_material(renderer: &Renderer) -> (&'static str, Material) {
		(
			"3dmesh_lit",
			renderer.create_material(include_str!("./mesh.wgsl"), &[
				MaterialBind {
					stage: MaterialBindStage::FRAGMENT,
					typ: MaterialBindType::Texture,
				},
				MaterialBind {
					stage: MaterialBindStage::FRAGMENT,
					typ: MaterialBindType::Sampler,
				},
				MaterialBind {
					stage: MaterialBindStage::FRAGMENT,
					typ: MaterialBindType::Texture,
				},
				MaterialBind {
					stage: MaterialBindStage::FRAGMENT,
					typ: MaterialBindType::Sampler,
				},
			])
		)
	}
	
	pub fn set_vertices(&mut self, renderer: &Renderer, vertices: &[Vertex]) {
		self.vertex_buffer = create_vertex_buffer(renderer, vertices);
	}
	
	pub fn set_indices(&mut self, renderer: &Renderer, indices: &[u16]) {
		self.index_buffer = create_index_buffer(renderer, indices);
		self.index_count = indices.len() as u32;
	}
}

impl super::Object for Mesh {
	fn as_any(&self) -> &dyn std::any::Any {
		self
	}
	
	fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
		self
	}
	
	fn get_matrix(&self) -> &glam::Mat4 {
		&self.matrix
	}
	
	fn get_matrix_mut(&mut self) -> &mut glam::Mat4 {
		&mut self.matrix
	}
	
	fn get_material_id(&self) -> &str {
		&self.material
	}
	
	fn get_index_buffer(&self) -> &Buffer {
		&self.index_buffer
	}
	
	fn get_vertex_buffer(&self) -> &Buffer {
		&self.vertex_buffer
	}
	
	fn get_index_count(&self) -> u32 {
		self.index_count
	}
	
	fn get_shader_resources(&self) -> &[ShaderResource] {
		&self.shader_resources
	}
	
	fn get_shader_resources_mut(&mut self) -> &mut [ShaderResource] {
		&mut self.shader_resources
	}
	
	fn get_visible(&self) -> bool {
		self.visible
	}
	
	fn get_visible_mut(&mut self) -> &mut bool {
		&mut self.visible
	}
}

pub fn create_vertex_buffer(renderer: &Renderer, vertices: &[Vertex]) -> Buffer {
	let vertex_buffer = renderer.create_buffer(size_of_val(vertices), BufferUsage::COPY_DST | BufferUsage::VERTEX);
	vertex_buffer.set_data(bytemuck::cast_slice(vertices));
	vertex_buffer
}

pub fn create_index_buffer(renderer: &Renderer, indices: &[u16]) -> Buffer {
	let index_buffer;
	if indices.len() % 2 == 0 {
		index_buffer = renderer.create_buffer(size_of_val(indices), BufferUsage::COPY_DST | BufferUsage::INDEX);
		index_buffer.set_data(bytemuck::cast_slice(indices));
	} else {
		// needs to be 4byte aligned, we will intentionally cause the buffer bounds to overflow
		// this doesnt matter since the last u16 arent actually used in the rendering pipeline
		// (atleast i think this wont cause issues?)
		index_buffer = renderer.create_buffer(size_of_val(indices) + 2, BufferUsage::COPY_DST | BufferUsage::INDEX);
		index_buffer.set_data(unsafe{std::slice::from_raw_parts(indices.as_ptr() as _, indices.len() * 2 + 2)});
	}
	
	index_buffer
}