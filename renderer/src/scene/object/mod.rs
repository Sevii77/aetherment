use glam::{Mat4, Vec3, Quat, Vec4Swizzles};
use crate::renderer::{Buffer, ShaderResource};

pub mod mesh;
pub use mesh::Mesh;
pub mod skybox;
pub use skybox::Skybox;

pub trait Object {
	fn as_any(&self) -> &dyn std::any::Any;
	fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
	fn get_matrix(&self) -> &Mat4;
	fn get_matrix_mut(&mut self) -> &mut Mat4;
	fn get_material_id(&self) -> &str;
	fn get_vertex_buffer(&self) -> &Buffer;
	fn get_index_buffer(&self) -> &Buffer;
	fn get_index_count(&self) -> u32;
	fn get_shader_resources(&self) -> &[ShaderResource];
	fn get_shader_resources_mut(&mut self) -> &mut [ShaderResource];
	fn get_visible(&self) -> bool;
	fn get_visible_mut(&mut self) -> &mut bool;
	
	fn get_translation(&self) -> Vec3 {
		self.get_matrix().w_axis.xyz()
	}
	
	fn set_translation(&mut self, position: Vec3) {
		let matrix = self.get_matrix_mut();
		matrix.w_axis.x = position.x;
		matrix.w_axis.y = position.y;
		matrix.w_axis.z = position.z;
	}
	
	fn get_rotation(&self) -> Quat {
		self.get_matrix().to_scale_rotation_translation().1
	}
	
	fn set_rotation(&mut self, rotation: Quat) {
		let rotation = glam::Mat4::from_quat(rotation);
		let scale = self.get_scale();
		let matrix = self.get_matrix_mut();
		matrix.x_axis = rotation.x_axis * scale.x;
		matrix.y_axis = rotation.y_axis * scale.y;
		matrix.z_axis = rotation.z_axis * scale.z;
	}
	
	fn get_scale(&self) -> Vec3 {
		let matrix = self.get_matrix();
		let det = matrix.determinant();
		Vec3::new(
			matrix.x_axis.length() * f32::signum(det),
			matrix.y_axis.length(),
			matrix.z_axis.length(),
		)
	}
	
	fn set_scale(&mut self, scale: Vec3) {
		let old = self.get_scale();
		self.scale(scale / old);
	}
	
	fn scale(&mut self, scale: Vec3) {
		let matrix = self.get_matrix_mut();
		matrix.x_axis *= scale.x;
		matrix.y_axis *= scale.y;
		matrix.z_axis *= scale.z;
	}
}