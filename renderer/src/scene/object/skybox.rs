use crate::{renderer::*, vertex, Vertex};

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct SkyboxColor {
	points: [glam::Vec4; 8],
	colors: [glam::Vec4; 8],
}

pub struct Skybox {
	matrix: glam::Mat4,
	vertex_buffer: Box<dyn Buffer>,
	index_buffer: Box<dyn Buffer>,
	colors: [ShaderResource; 1],
}

impl Skybox {
	pub fn new(renderer: &Box<dyn Renderer>, colors: &[(f32, glam::Vec4)]) -> Self {
		const VERTICES: &[Vertex] = &[
			vertex(glam::vec3(-1.0, -1.0, 0.0), glam::Vec3::ZERO, glam::Vec4::ZERO, glam::Vec2::ZERO),
			vertex(glam::vec3( 1.0, -1.0, 0.0), glam::Vec3::ZERO, glam::Vec4::ZERO, glam::Vec2::X),
			vertex(glam::vec3(-1.0,  1.0, 0.0), glam::Vec3::ZERO, glam::Vec4::ZERO, glam::Vec2::Y),
			vertex(glam::vec3( 1.0,  1.0, 0.0), glam::Vec3::ZERO, glam::Vec4::ZERO, glam::Vec2::ONE),
		];
		
		const INDICES: &[u16] = &[0, 1, 2, 1, 3, 2];
		
		let mut colors_new = SkyboxColor::default();
		for (i, (point, color)) in colors.into_iter().enumerate() {
			colors_new.points[i] = glam::vec4(*point, 0.0, 0.0, 0.0); // alignment to 16 bytes
			colors_new.colors[i] = *color;
		}
		
		let vertex_buffer = renderer.create_buffer(size_of_val(VERTICES), BufferUsage::COPY_DST | BufferUsage::VERTEX);
		vertex_buffer.set_data(&bytemuck::cast_slice(VERTICES));
		
		let index_buffer = renderer.create_buffer(size_of_val(INDICES), BufferUsage::COPY_DST | BufferUsage::INDEX);
		index_buffer.set_data(&bytemuck::cast_slice(INDICES));
		
		let color_buffer = renderer.create_buffer(size_of::<SkyboxColor>(), BufferUsage::COPY_DST | BufferUsage::UNIFORM);
		color_buffer.set_data(&bytemuck::cast_slice(&[colors_new]));
		
		Self {
			matrix: glam::Mat4::IDENTITY,
			vertex_buffer,
			index_buffer,
			colors: [ShaderResource::Buffer(color_buffer)],
		}
	}
	
	pub fn simple(renderer: &Box<dyn Renderer>) -> Self {
		Self::new(renderer, &[
			(0.0, glam::vec4(0.0, 0.0, 0.0, 1.0)),
			(0.45, glam::vec4(0.1, 0.1, 0.1, 1.0)),
			(0.55, glam::vec4(0.4, 0.9, 1.0, 1.0)),
			(1.0, glam::vec4(0.1, 0.6, 1.0, 1.0)),
		])
	}
	
	pub(crate) fn create_material(renderer: &Box<dyn Renderer>) -> (&'static str, Box<dyn Material>) {
		(
			"skybox",
			renderer.create_material(include_str!("./skybox.wgsl"), &[
				MaterialBind {
					stage: MaterialBindStage::FRAGMENT,
					typ: MaterialBindType::Buffer,
				}
			])
		)
	}
}

impl super::Object for Skybox {
	fn get_matrix(&self) -> &glam::Mat4 {
		&self.matrix
	}
	
	fn get_matrix_mut(&mut self) -> &mut glam::Mat4 {
		&mut self.matrix
	}
	
	fn get_material_id(&self) -> &str {
		"skybox"
	}
	
	fn get_index_buffer(&self) -> &Box<dyn Buffer> {
		&self.index_buffer
	}
	
	fn get_vertex_buffer(&self) -> &Box<dyn Buffer> {
		&self.vertex_buffer
	}
	
	fn get_index_count(&self) -> u32 {
		6
	}
	
	fn get_shader_resources(&self) -> &[ShaderResource] {
		&self.colors
	}
	
	fn get_shader_resources_mut(&mut self) -> &mut [ShaderResource] {
		&mut self.colors
	}
}