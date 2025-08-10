use std::collections::HashMap;
use crate::renderer::*;

mod object;
pub use object::*;
mod camera;
pub use camera::*;

pub struct Scene {
	clear_color: Option<[f32; 4]>,
	render_target: Texture,
	depth_buffer: Texture,
	materials: HashMap<&'static str, Material>,
	objects: crate::ObjectBuffer,
}

impl Scene {
	pub fn new(renderer: &Renderer, render_width: u32, render_height: u32) -> Self {
		Self {
			clear_color: Some([0.0, 0.0, 0.0, 1.0]),
			render_target: renderer.create_texture(render_width, render_height, TextureFormat::Rgba8Unorm, TextureUsage::RENDER_TARGET | TextureUsage::TEXTURE_BINDING),
			depth_buffer: renderer.create_texture(render_width, render_height, TextureFormat::Depth32Float, TextureUsage::DEPTH_STENCIL),
			materials: HashMap::from([
				Skybox::create_material(renderer),
				Mesh::create_material(renderer),
			]),
			objects: Vec::new(),
		}
	}
	
	/// Resizes the render target.
	/// This will make the previous ones invalid!
	pub fn resize(&mut self, renderer: &Renderer, render_width: u32, render_height: u32) {
		self.render_target = renderer.create_texture(render_width, render_height, TextureFormat::Rgba8Unorm, TextureUsage::RENDER_TARGET | TextureUsage::TEXTURE_BINDING);
		self.depth_buffer = renderer.create_texture(render_width, render_height, TextureFormat::Depth32Float, TextureUsage::DEPTH_STENCIL);
	}
	
	pub fn set_clear_color(&mut self, color: Option<[f32; 4]>) {
		self.clear_color = color;
	}
	
	pub fn register_material(&mut self, id: &'static str, material: Material) {
		self.materials.insert(id, material);
	}
	
	pub fn add_object(&mut self, object: Box<dyn Object>) -> usize {
		for (i, obj) in self.objects.iter_mut().enumerate() {
			if obj.is_none() {
				*obj = Some(object);
				return i;
			}
		}
		
		self.objects.push(Some(object));
		self.objects.len() - 1
	}
	
	pub fn get_object(&self, id: usize) -> Option<&Box<dyn Object>> {
		let Some(obj) = self.objects.get(id) else {return None};
		obj.as_ref()
	}
	
	pub fn get_object_mut(&mut self, id: usize) -> Option<&mut Box<dyn Object>> {
		let Some(obj) = self.objects.get_mut(id) else {return None};
		obj.as_mut()
	}
	
	pub fn render(&self, renderer: &Renderer, camera: &Camera) {
		renderer.render(&self.clear_color, &self.render_target, &self.depth_buffer, &self.materials, &self.objects, camera);
	}
	
	pub fn get_render_target(&self) -> &Texture {
		&self.render_target
	}
}