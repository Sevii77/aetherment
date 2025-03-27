pub struct Debug {
	new_uicolor_theme: bool,
	new_uicolor_index: u32,
}

impl Debug {
	pub fn new() -> Self {
		Self {
			new_uicolor_theme: true,
			new_uicolor_index: 1,
		}
	}
	
	pub fn draw(&mut self, ui: &mut renderer::Ui) {
		if ui.button("kill").clicked {
			panic!("the kill switch was pressed");
		}
		
		ui.add_space(16.0);
		ui.label("UiColor Replacements");
		for ((theme_color, index), [r, g, b]) in crate::service::uicolor::get_colors() {
			ui.horizontal(|ui| {ui.push_id(index, |ui| {
				if ui.button("x").clicked {
					crate::service::uicolor::remove_color(theme_color, index);
				}
				
				let mut clr = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];
				if ui.color_edit_rgb(format!("{} {index}", if theme_color {"theme"} else {"normal"}), &mut clr).changed {
					crate::service::uicolor::set_color(theme_color, index, [(clr[0] * 255.0) as u8, (clr[1] * 255.0) as u8, (clr[2] * 255.0) as u8]);
				}
			})});
		}
		
		ui.horizontal(|ui| {
			if ui.button("+").clicked {
				crate::service::uicolor::set_color(self.new_uicolor_theme, self.new_uicolor_index, [255, 255, 255]);
			}
			
			ui.checkbox("", &mut self.new_uicolor_theme);
			
			let mut val = self.new_uicolor_index.to_string();
			ui.input_text("Add Ui Color", &mut val);
			if let Ok(val) = u32::from_str_radix(&val, 10) {
				self.new_uicolor_index = val;
			}
		});
		
		ui.add_space(16.0);
		ui.debug();
	}
}