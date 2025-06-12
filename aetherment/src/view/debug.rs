use crate::ui_ext::UiExt;

pub struct Debug {
	new_uicolor_theme: bool,
	new_uicolor_index: u32,
	
	userspace_loaders: bool,
}

impl Debug {
	pub fn new() -> Self {
		Self {
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
	
	fn ui(&mut self, ui: &mut egui::Ui) {
		ui.heading("UiColor Replacements");
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
		
		ui.add_space(16.0);
		ui.heading("Ui Settings");
		ui.ctx().clone().settings_ui(ui);
		
		ui.add_space(16.0);
		ui.heading("Ui Inspection");
		ui.ctx().clone().inspection_ui(ui);
		
		ui.add_space(16.0);
		ui.heading("Loaders");
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
	}
}