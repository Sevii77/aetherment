pub struct InteractableScene {
	scene: renderer::Scene,
	texture_id: egui::TextureId,
	width: usize,
	height: usize,
	
	pub camera_origin_default: glam::Vec3,
	pub camera_origin: glam::Vec3,
	pub camera_zoom_default: f32,
	pub camera_zoom: f32,
	pub camera_pitch: f32,
	pub camera_yaw: f32,
}

impl std::ops::Deref for InteractableScene {
	type Target = renderer::Scene;

	fn deref(&self) -> &Self::Target {
		&self.scene
	}
}

impl std::ops::DerefMut for InteractableScene {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.scene
	}
}

impl InteractableScene {
	pub fn new(renderer: &renderer::Renderer) -> Self {
		let scene = renderer::Scene::new(renderer, 32, 32);
		
		Self {
			texture_id: egui::TextureId::User(renderer.register_texture(scene.get_render_target())),
			scene,
			width: 32,
			height: 32,
			
			camera_origin_default: glam::Vec3::ZERO,
			camera_zoom_default: 4.0,
			camera_origin: glam::Vec3::ZERO,
			camera_zoom: 4.0,
			camera_pitch: -0.2,
			camera_yaw: 0.0,
		}
	}
	
	pub fn set_camera_defaults(&mut self, origin: glam::Vec3, distance: f32) {
		self.camera_origin_default = origin;
		self.camera_zoom_default = distance;
		self.camera_origin = origin;
		self.camera_zoom = distance;
	}
	
	pub fn render(&mut self, renderer: &renderer::Renderer, width: usize, height: usize, ui: &mut egui::Ui) {
		if (self.width != width || self.height != height) && width >= 16 && height >= 16 {
			self.width = width;
			self.height = height;
			self.scene.resize(renderer, width as u32, height as u32);
			self.texture_id = egui::TextureId::User(renderer.register_texture(self.scene.get_render_target()));
		}
		
		self.scene.render(renderer, &renderer::Camera {
			view: glam::Mat4::from_translation(self.camera_origin)
				.mul_mat4(&glam::Mat4::from_euler(glam::EulerRot::YXZ, self.camera_yaw, self.camera_pitch, 0.0))
				.mul_mat4(&glam::Mat4::from_translation(glam::vec3(0.0, 0.0, self.camera_zoom))),
			fov: 70.0,
			z_near: 0.001,
			z_far: 1000.0,
		});
		
		let (id, rect) = ui.allocate_space(egui::vec2(width as f32, height as f32));
		let resp = ui.interact(rect, id, egui::Sense::all());
		
		resp.context_menu(|ui| {
			if ui.button("Reset Camera").clicked() {
				self.camera_origin = self.camera_origin_default;
				self.camera_zoom = self.camera_zoom_default;
				self.camera_pitch = -0.2;
				self.camera_yaw = 0.0;
				
				ui.close_menu();
			}
		});
		
		let ctx = ui.ctx();
		if resp.dragged() {
			ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
			
			let drag = resp.drag_delta();
			if resp.dragged_by(egui::PointerButton::Primary) {
				self.camera_pitch -= drag.y / 100.0;
				self.camera_yaw -= drag.x / 100.0;
			} else if resp.dragged_by(egui::PointerButton::Middle) {
				self.camera_origin += glam::Quat::from_euler(glam::EulerRot::YXZ, self.camera_yaw, self.camera_pitch, 0.0) * glam::vec3(-drag.x, drag.y, 0.0) / 500.0 * self.camera_zoom;
			}
		} else if resp.hovered() {
			let zoom_speed_mul = (self.camera_zoom / 4.0).clamp(0.5, 10.0);
			let scroll = ctx.input(|v| v.smooth_scroll_delta);
			if scroll.y != 0.0 {
				ctx.set_cursor_icon(egui::CursorIcon::ResizeVertical);
				self.camera_zoom = (self.camera_zoom - scroll.y / 500.0 * zoom_speed_mul).max(0.01);
			} else if scroll.x != 0.0 {
				ctx.set_cursor_icon(egui::CursorIcon::ResizeVertical);
				self.camera_zoom = (self.camera_zoom - scroll.x / 100.0 * zoom_speed_mul).max(0.01);
			} else {
				ctx.set_cursor_icon(egui::CursorIcon::Grab);
			}
		}
		
		let mut drawscene = egui::Mesh::with_texture(self.texture_id);
		drawscene.add_rect_with_uv(
			rect,
			egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
			egui::Color32::WHITE);
		ui.painter().add(drawscene);
	}
}