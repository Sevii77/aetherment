use crate::ui_ext::UiExt;

pub struct Debug {
	scene: Option<(renderer::Scene, egui::TextureId)>,
	
	new_uicolor_theme: bool,
	new_uicolor_index: u32,
	
	userspace_loaders: bool,
}

impl Debug {
	pub fn new() -> Self {
		Self {
			scene: None,
			
			new_uicolor_theme: true,
			new_uicolor_index: 1,
			
			userspace_loaders: false,
		}
	}
}

impl super::View for Debug {
	fn title(&self) -> &'static str {
		"Debug"
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, renderer: &crate::Renderer) {
		ui.collapsing("3D Renderer", |ui| {
			let (scene, texture_id) = self.scene.get_or_insert_with(|| {
				let mut scene = renderer::Scene::new(renderer, 512, 512);
				scene.set_clear_color(Some([0.0; 4]));
				
				let albedo = image::ImageReader::new(std::io::Cursor::new(include_bytes!("../../debug_albedo.png"))).with_guessed_format().unwrap().decode().unwrap().into_bytes();
				let normal = image::ImageReader::new(std::io::Cursor::new(include_bytes!("../../debug_normal.png"))).with_guessed_format().unwrap().decode().unwrap().into_bytes();
				let normal2 = image::ImageReader::new(std::io::Cursor::new(include_bytes!("../../debug_normal2.png"))).with_guessed_format().unwrap().decode().unwrap().into_bytes();
				
				let id = scene.add_object(Box::new(renderer::Mesh::new_test_cube(renderer)));
				let obj = scene.get_object_mut(id).unwrap();
				obj.get_shader_resources_mut()[0] = renderer::renderer::ShaderResource::Texture(
					renderer.create_texture_initialized(64, 64, renderer::renderer::TextureFormat::Rgba8UnormSrgb, renderer::renderer::TextureUsage::TEXTURE_BINDING, &albedo));
				obj.get_shader_resources_mut()[2] = renderer::renderer::ShaderResource::Texture(
					renderer.create_texture_initialized(64, 64, renderer::renderer::TextureFormat::Rgba8Unorm, renderer::renderer::TextureUsage::TEXTURE_BINDING, &normal2));
				
				let id = scene.add_object(Box::new(renderer::Mesh::new_test_cube(renderer)));
				let obj = scene.get_object_mut(id).unwrap();
				obj.get_shader_resources_mut()[0] = renderer::renderer::ShaderResource::Texture(
					renderer.create_texture_initialized(64, 64, renderer::renderer::TextureFormat::Rgba8UnormSrgb, renderer::renderer::TextureUsage::TEXTURE_BINDING, &albedo));
				obj.get_shader_resources_mut()[2] = renderer::renderer::ShaderResource::Texture(
					renderer.create_texture_initialized(64, 64, renderer::renderer::TextureFormat::Rgba8Unorm, renderer::renderer::TextureUsage::TEXTURE_BINDING, &normal));
				
				let texture_id = egui::TextureId::User(renderer.register_texture(scene.get_render_target()));
				(scene, texture_id)
			});
			
			let time = ui.ctx().input(|v| v.time) as f32;
			
			let obj = scene.get_object_mut(0).unwrap();
			obj.set_rotation(glam::Quat::from_euler(glam::EulerRot::YXZ, -time, -time * 2.0, -time * 0.5));
			
			let obj = scene.get_object_mut(1).unwrap();
			obj.set_translation(glam::vec3(time.sin() * 3.0, (time + 1.0).sin() * 0.5, time.cos() * 3.0));
			obj.set_rotation(glam::Quat::from_euler(glam::EulerRot::YXZ, time * 0.7, time * 0.5, time * 0.3));
			obj.set_scale(glam::vec3(0.5, 0.5, 0.5));
			
			scene.render(renderer, &renderer::Camera::new(glam::vec3(0.0, 2.0, 8.0), glam::Quat::from_euler(glam::EulerRot::YXZ, 0.0, -0.2, 0.0)));
			
			let mut drawscene = egui::Mesh::with_texture(*texture_id);
			let (_, rect) = ui.allocate_space(egui::vec2(512.0, 512.0));
			drawscene.add_rect_with_uv(
				rect,
				egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
				egui::Color32::WHITE);
			ui.painter().add(drawscene);
			
			ui.ctx().request_repaint();
		});
		
		ui.collapsing("UiColor Replacements", |ui| {
			for ((theme_color, index), [r, g, b]) in crate::service::uicolor::get_colors() {
				ui.horizontal(|ui| {ui.push_id(index, |ui| {
					if ui.button("x").clicked() {
						crate::service::uicolor::remove_color(theme_color, index);
					}
					
					let mut clr = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];
					if ui.color_edit(&mut clr).changed() {
						crate::service::uicolor::set_color(theme_color, index, [(clr[0] * 255.0) as u8, (clr[1] * 255.0) as u8, (clr[2] * 255.0) as u8]);
					}
					
					ui.label(format!("{} {index}", if theme_color {"theme"} else {"normal"}))
				})});
			}
			
			ui.horizontal(|ui| {
				if ui.button("+").clicked() {
					crate::service::uicolor::set_color(self.new_uicolor_theme, self.new_uicolor_index, [255, 255, 255]);
				}
				
				ui.checkbox(&mut self.new_uicolor_theme, "");
				
				let mut val = self.new_uicolor_index.to_string();
				ui.text_edit_singleline(&mut val);
				if let Ok(val) = u32::from_str_radix(&val, 10) {
					self.new_uicolor_index = val;
				}
				
				ui.label("Add Ui Color");
			});
		});
		
		ui.collapsing("Ui Settings", |ui|
			ui.ctx().clone().settings_ui(ui));
		
		ui.collapsing("Ui Inspection", |ui|
			ui.ctx().clone().inspection_ui(ui));
		
		ui.collapsing("Loaders", |ui| {
			ui.checkbox(&mut self.userspace_loaders, "Show userspace");
			
			let draw_loaders = |ui: &mut egui::Ui| {
				let loaders = ui.ctx().loaders();
				ui.label("Texture");
				ui.indent("texture", |ui| {
					for loader in loaders.texture.lock().iter() {
						ui.label(loader.id());
					}
				});
				
				ui.label("Image");
				ui.indent("image", |ui| {
					for loader in loaders.image.lock().iter() {
						ui.label(loader.id());
					}
				});
				
				ui.label("Byte");
				ui.indent("byte", |ui| {
					for loader in loaders.bytes.lock().iter() {
						ui.label(loader.id());
					}
				});
			};
			
			if self.userspace_loaders {
				ui.userspace_loaders(draw_loaders);
			} else {
				draw_loaders(ui);
			}
		});
	}
}