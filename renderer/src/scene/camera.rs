use glam::{Mat4, Quat, Vec3, Vec4Swizzles};

pub struct Camera {
	pub view: Mat4,
	pub fov: f32,
	pub z_near: f32,
	pub z_far: f32,
}

impl Camera {
	pub const IDENTITY: Self = Self {
		view: Mat4::IDENTITY,
		fov: 70.0,
		z_near: 0.001,
		z_far: 1000.0,
	};
	
	pub fn new(position: Vec3, rotation: Quat) -> Self {
		Self {
			view: Mat4::from_translation(position).mul_mat4(&glam::Mat4::from_quat(rotation)),
			fov: 70.0,
			z_near: 0.001,
			z_far: 1000.0,
		}
	}
	
	pub fn get_translation(&self) -> Vec3 {
		self.view.w_axis.xyz()
	}
	
	pub fn set_translation(&mut self, position: Vec3) {
		self.view.w_axis.x = position.x;
		self.view.w_axis.y = position.y;
		self.view.w_axis.z = position.z;
	}
	
	pub fn get_rotation(&self) -> Quat {
		self.view.to_scale_rotation_translation().1
	}
	
	pub fn set_rotation(&mut self, rotation: Quat) {
		let rotation = glam::Mat4::from_quat(rotation);
		let scale = self.get_scale();
		self.view.x_axis = rotation.x_axis * scale.x;
		self.view.y_axis = rotation.y_axis * scale.y;
		self.view.z_axis = rotation.z_axis * scale.z;
	}
	
	pub(crate) fn proj_matrix(&self, aspect: f32) -> Mat4 {
		Mat4::perspective_rh(
			self.fov.to_radians(),
			aspect,
			self.z_near,
			self.z_far,
		)
	}
	
	fn get_scale(&self) -> Vec3 {
		let det = self.view.determinant();
		Vec3::new(
			self.view.x_axis.length() * f32::signum(det),
			self.view.y_axis.length(),
			self.view.z_axis.length(),
		)
	}
}