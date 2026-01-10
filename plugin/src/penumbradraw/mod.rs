use std::collections::{HashMap, HashSet};

mod imgui_bindings;
mod imgui;

pub struct PenumbraDraw {
	mod_manager: aetherment::modman::manager::Manager,
	new_preset_name: String,
	selected_category_tab: usize,
}

impl PenumbraDraw {
	pub fn new(mod_manager: aetherment::modman::manager::Manager) -> Self {
		Self {
			mod_manager,
			new_preset_name: String::new(),
			selected_category_tab: 0,
		}
	}
	
	pub fn settings(&mut self, ui_scale: f32, mod_id: &str) -> bool {
		self.mod_manager.update_last_viewed();
		
		let Some(meta) = self.mod_manager.metas.get(mod_id) else {return false};
		let Some(mut mod_settings) = self.mod_manager.settings.get_mut(mod_id) else {return false};
		let Some(mut remote_settings) = self.mod_manager.settings_remote.get_mut(mod_id) else {return false};
		let collection_id = aetherment::modman::backend::penumbra_ipc::get_collection(aetherment::modman::backend::CollectionType::Current).id;
		let Some(collection) = self.mod_manager.collections.get(collection_id.as_str()) else {return false};
		let mut presets = mod_settings.presets.clone();
		let settings = mod_settings.get_collection(&meta, &collection);
		
		if !remote_settings.origin.is_empty() {
			imgui::dummy([0.0, 10.0 * ui_scale]);
			if imgui::checkbox(&mut remote_settings.auto_update, "Auto Update") {
				remote_settings.save(mod_id);
			}
		}
		
		if !self.mod_manager.aeth_mods.contains(mod_id) {return false};
		
		let mut changed = false;
		if meta.options.len() > 0 {
			// get active preset, yoinked from views::mods
			let mut selected_preset = "Custom".to_string();
			'default: {
				for (name, value) in settings.iter() {
					if let Some(opt) = meta.options.options_iter().find(|v| v.name == *name) {
						if aetherment::modman::settings::Value::from_meta_option(opt) != *value {break 'default}
					}
				}
				
				selected_preset = "Default".to_owned();
			}
			
			let mut check_presets = |presets: &Vec<aetherment::modman::settings::Preset>| {
				'preset: for v in presets.iter() {
					for (name, value) in settings.iter() {
						match v.settings.get(name) {
							Some(v) => if v != value {continue 'preset},
							None => if aetherment::modman::settings::Value::from_meta_option(meta.options.options_iter().find(|v| v.name == *name).unwrap()) != *value {continue 'preset}
						}
					}
					
					selected_preset = v.name.to_owned();
				}
			};
			check_presets(&meta.presets);
			check_presets(&presets);
			
			// preset dropdown
			imgui::combo(&selected_preset, "Preset", || {
				let mut set_settings = |values: &HashMap<String, aetherment::modman::settings::Value>| {
					for (name, value) in settings.iter_mut() {
						*value = values.get(name).map_or_else(|| aetherment::modman::settings::Value::from_meta_option(meta.options.options_iter().find(|v| v.name == *name).unwrap()), |v| v.to_owned());
					}
					
					changed = true;
				};
				
				// built in presets
				if meta.presets.len() == 0 {
					if imgui::selectable_label("Default" == selected_preset, "Default") {
						set_settings(&HashMap::new());
					}
				}
				
				for p in &meta.presets {
					if imgui::selectable_label(p.name == selected_preset, &p.name) {
						set_settings(&p.settings);
					}
				}
				
				// custom presets
				let mut delete = None;
				for (i, p) in presets.iter().enumerate() {
					imgui::push_id(i, || {
						if imgui::button("X") {
							delete = Some(i);
						}
						imgui::hover_text("Delete");
						
						imgui::same_line();
						if imgui::button("^") {
							_ = clipboard_win::set_clipboard_string(&p.sharable_string());
						}
						imgui::hover_text("Copy to clipboard");
						
						imgui::same_line();
						if imgui::selectable_label(p.name == selected_preset, &p.name) {
							set_settings(&p.settings);
						}
					});
				}
				
				if let Some(delete) = delete {
					presets.remove(delete);
					changed = true;
				}
				
				// new preset
				if imgui::button("+") && self.new_preset_name.len() > 0 && self.new_preset_name != "Custom" && self.new_preset_name != "Default" {
					let preset = aetherment::modman::settings::Preset {
						name: self.new_preset_name.clone(),
						settings: settings.iter().map(|(a, b)| (a.to_owned(), b.to_owned())).collect()
					};
					
					if let Some(existing) = presets.iter_mut().find(|v| v.name == preset.name) {
						*existing = preset;
					} else {
						presets.push(preset);
					}
					
					self.new_preset_name.clear();
					changed = true;
				}
				
				imgui::same_line();
				imgui::text_edit_singleline(&mut self.new_preset_name, "##newpreset");
				
				// import
				if imgui::button("Import preset from clipboard") {'import: {
					let Ok(clip) = clipboard_win::get_clipboard_string() else {break 'import};
					let Some(preset) = aetherment::modman::settings::Preset::from_sharable_string(&clip) else {break 'import};
					if preset.name.len() == 0 || preset.name == "Custom" || preset.name == "Default" {break 'import}
					
					if let Some(existing) = presets.iter_mut().find(|v| v.name == preset.name) {
						*existing = preset;
					} else {
						presets.push(preset);
					}
					
					changed = true;
				}}
			});
		}
		
		// categories
		imgui::dummy([0.0, 10.0 * ui_scale]);
		
		let mut categories = meta.options.categories_iter().collect::<Vec<_>>();
		if !matches!(meta.options.0.get(0), Some(aetherment::modman::meta::OptionType::Category(_))) {
			categories.insert(0, "Main");
		}
		
		if categories.len() > 1 {
			if self.selected_category_tab >= categories.len() {
				self.selected_category_tab = 0;
			}
			
			self.selected_category_tab = imgui::tabbar(&categories);
		}
		
		// build grouped options
		let mut grouped_options = HashSet::new();
		for option in meta.options.options_iter() {
			if let aetherment::modman::meta::OptionSettings::Grouped(group) = &option.settings {
				for sub in group.options.iter() {
					for val in sub.options.iter() {
						if let aetherment::modman::meta::ValueGroupedOptionEntryType::Option(val) = val {
							for name in val.options.iter() {
								grouped_options.insert(name.as_str());
							}
						}
					}
				}
			}
		}
		
		// generic settings
		let mut cur_category = "Main";
		for option_type in meta.options.iter() {
			let option = match option_type {
				aetherment::modman::meta::OptionType::Option(v) => v,
				aetherment::modman::meta::OptionType::Category(s) => {cur_category = s.as_ref(); continue},
			};
			
			if grouped_options.contains(option.name.as_str()) {continue}
			if categories.len() > 1 && cur_category != categories[self.selected_category_tab] {continue}
			
			changed |= draw_option(ui_scale, mod_id, &meta, settings, option, &option.name, &option.description);
		}
		
		if changed {
			let backend = aetherment::backend();
			backend.apply_mod_settings(mod_id, &collection_id, aetherment::modman::backend::SettingsType::Some(settings.clone()));
			mod_settings.presets = presets;
			mod_settings.save(mod_id);
			self.mod_manager.update_last_interacted();
		}
		
		true
	}
}

fn draw_description(_mod_id: &str, text: &str) {
	if text.starts_with("[md]") {
		// TODO: markdown support
		imgui::label(text);
	} else {
		imgui::label(text);
	}
}

fn draw_help(mod_id: &str, text: &str) {
	if text.is_empty() {return}
	
	imgui::same_line();
	imgui::label("(?)");
	imgui::hover(|| draw_description(mod_id, text));
}

fn draw_option(ui_scale: f32, mod_id: &str, meta: &aetherment::modman::meta::Meta, settings: &mut aetherment::modman::settings::CollectionSettings, option: &aetherment::modman::meta::Option, name: &str, desc: &str) -> bool {
	use aetherment::modman::{meta::OptionSettings, settings::Value::*};
	
	let mut changed = false;
	let setting_id = &option.name;
	let val = settings.get_mut(setting_id).unwrap();
	
	match val {
		Grouped(val) => {
			let OptionSettings::Grouped(o) = &option.settings else {imgui::label(format!("Invalid setting type for {setting_id}")); return false};
			let selected = o.options.get(*val as usize);
			
			imgui::combo(selected.map_or("Invalid", |v| &v.name), name, || {
				for (i, sub) in o.options.iter().enumerate() {
					changed |= imgui::selectable_value(val, i as u32, &sub.name);
					draw_help(mod_id, &sub.description);
				}
			});
			
			draw_help(mod_id, desc);
			
			if let Some(selected) = selected {
				for sub in selected.options.iter() {
					match sub {
						aetherment::modman::meta::ValueGroupedOptionEntryType::Category(v) =>
							_ = imgui::label(v),
						
						aetherment::modman::meta::ValueGroupedOptionEntryType::Option(v) => {
							if let Some(first) = v.options.first() {
								if let Some(opt) = meta.options.options_iter().find(|x| x.name == *first) {
									if draw_option(ui_scale, mod_id, meta, settings, opt, &v.name, &v.description) {
										let val = settings.get(first).unwrap().clone();
										for name in v.options.iter() {
											if let Some(opt) = meta.options.options_iter().find(|x| x.name == *name) {
												*settings.get_mut(&opt.name).unwrap() = val.clone();
											}
										}
										
										changed = true;
									}
								}
							}
						}
					}
				}
			}
		}
		
		SingleFiles(val) => {
			let OptionSettings::SingleFiles(o) = &option.settings else {imgui::label(format!("Invalid setting type for {setting_id}")); return false};
			imgui::combo(o.options.get(*val as usize).map_or("Invalid", |v| &v.name), name, || {
				for (i, sub) in o.options.iter().enumerate() {
					changed |= imgui::selectable_value(val, i as u32, &sub.name);
					draw_help(mod_id, &sub.description);
				}
			});
			
			draw_help(mod_id, &*desc);
		}
		
		MultiFiles(val) => {
			let OptionSettings::MultiFiles(o) = &option.settings else {imgui::label(format!("Invalid setting type for {setting_id}")); return false};
			
			imgui::label(name);
			draw_help(mod_id, &*desc);
			
			for (i, sub) in o.options.iter().enumerate() {
				imgui::dummy([10.0 * ui_scale, 0.0]);
				imgui::same_line();
				let mut toggled = *val & (1 << i) != 0;
				if imgui::checkbox(&mut toggled, &sub.name) {
					*val ^= 1 << i;
					changed = true;
				}
				
				draw_help(mod_id, &sub.description);
			}
		}
		
		Rgb(val) => {
			let OptionSettings::Rgb(o) = &option.settings else {imgui::label(format!("Invalid setting type for {setting_id}")); return false};
			
			changed |= imgui::color_edit(val, name);
			for (i, v) in val.iter_mut().enumerate() {*v = v.clamp(o.min[i], o.max[i])}
			
			draw_help(mod_id, &*desc);
		}
		
		Rgba(val) => {
			let OptionSettings::Rgba(o) = &option.settings else {imgui::label(format!("Invalid setting type for {setting_id}")); return false};
			
			changed |= imgui::color_edit(val, name);
			for (i, v) in val.iter_mut().enumerate() {*v = v.clamp(o.min[i], o.max[i])}
			
			draw_help(mod_id, &*desc);
		}
		
		Grayscale(val) => {
			let OptionSettings::Grayscale(o) = &option.settings else {imgui::label(format!("Invalid setting type for {setting_id}")); return false};
			
			changed |= imgui::slider(val, o.min..=o.max, name);
			*val = val.clamp(o.min, o.max);
			
			draw_help(mod_id, &*desc);
		}
		
		Opacity(val) => {
			let OptionSettings::Grayscale(o) = &option.settings else {imgui::label(format!("Invalid setting type for {setting_id}")); return false};
			
			changed |= imgui::slider(val, o.min..=o.max, name);
			*val = val.clamp(o.min, o.max);
			
			draw_help(mod_id, &*desc);
		}
		
		Mask(val) => {
			let OptionSettings::Grayscale(o) = &option.settings else {imgui::label(format!("Invalid setting type for {setting_id}")); return false};
			
			changed |= imgui::slider(val, o.min..=o.max, name);
			*val = val.clamp(o.min, o.max);
			
			draw_help(mod_id, &*desc);
		}
		
		Path(val) => {
			let OptionSettings::Path(o) = &option.settings else {imgui::label(format!("Invalid setting type for {setting_id}")); return false};
			
			imgui::combo(o.options.get(*val as usize).map_or("Invalid", |v| &v.0), name, || {
				for (i, (name, _)) in o.options.iter().enumerate() {
					changed |= imgui::selectable_value(val, i as u32, name);
				}
			});
			
			draw_help(mod_id, &*desc);
		}
	}
	
	changed
}