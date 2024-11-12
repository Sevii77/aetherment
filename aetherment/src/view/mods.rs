use std::collections::HashMap;
use crate::modman::settings::Settings;

pub struct Mods {
	active_collection: String,
	selected_mod: String,
	selected_category_tab: String,
	gamma: u8,
	
	import_picker: Option<renderer::FilePicker>,
	new_preset_name: String,
	
	last_was_busy: bool,
	mods: Vec<String>,
	collections: HashMap<String, String>,
	// mod_settings: HashMap<String, HashMap<String, Settings>>,
	mod_settings: HashMap<String, Settings>,
	mod_settings_remote: HashMap<String, crate::remote::settings::Settings>,
}

impl Mods {
	pub fn new() -> Self {
		let mut s = Self {
			active_collection: String::new(),
			selected_mod: String::new(),
			selected_category_tab: String::new(),
			gamma: 50,
			
			import_picker: None,
			new_preset_name: String::new(),
			
			last_was_busy: false,
			mods: Vec::new(),
			collections: HashMap::new(),
			mod_settings: HashMap::new(),
			mod_settings_remote: HashMap::new(),
		};
		
		s.refresh();
		s
	}
	
	pub fn refresh(&mut self) {
		let backend = crate::backend();
		backend.load_mods();
		
		self.mods = backend.get_mods();
		self.mods.sort_unstable();
		
		self.collections = backend.get_collections().into_iter().map(|v| (v.id, v.name)).collect();
		// self.mod_settings = self.mods.iter().map(|m| (m.to_owned(), self.collections.iter().map(|(c, _)| (c.to_owned(), backend.get_mod_settings(m, c).unwrap())).collect())).collect();
		self.mod_settings = self.mods.iter().map(|m| (m.to_owned(), crate::modman::settings::Settings::open(backend.get_mod_meta(m).unwrap(), m))).collect();
		self.mod_settings_remote = self.mods.iter().map(|m| (m.to_owned(), crate::remote::settings::Settings::open(m))).collect();
		
		if !self.collections.iter().any(|(c, _)| c == &self.active_collection) {
			// self.active_collection = self.collections.iter().find(|_| true).map_or_else(|| String::new(), |(c, _)| c.clone());
			self.active_collection = backend.get_active_collection();
		}
	}
	
	pub fn draw(&mut self, ui: &mut renderer::Ui, install_progress: crate::modman::backend::InstallProgress, apply_progress: crate::modman::backend::ApplyProgress) {
		let backend = crate::backend();
		let config = crate::config();
		config.mark_for_changes();
		let is_busy = install_progress.is_busy() || apply_progress.is_busy();
		if self.last_was_busy && !is_busy {
			self.refresh();
		}
		self.last_was_busy = is_busy;
		
		ui.splitter("splitter", 0.3, |ui_left, ui_right| {
			{
				let ui = ui_left;
				if ui.button("Import Mods").clicked && self.import_picker.is_none() {
					self.import_picker = Some(renderer::FilePicker::new("Import Mods", &config.config.file_dialog_path, &[".aeth"], renderer::FilePickerMode::OpenFileMultiple));
				}
				
				if let Some(picker) = &mut self.import_picker {
					match picker.show(ui) {
						renderer::FilePickerStatus::Success(dir, paths) => {
							backend.install_mods_path(install_progress.clone(), paths);
							config.config.file_dialog_path = dir;
							self.import_picker = None;
						}
						
						renderer::FilePickerStatus::Canceled(dir) => {
							config.config.file_dialog_path = dir;
							self.import_picker = None;
						}
						
						_ => {}
					}
				}
				
				// if !is_busy && ui.button("Update Mods").clicked {
				// 	let progress = install_progress.clone();
				// 	std::thread::spawn(move || {
				// 		crate::remote::check_updates(progress);
				// 	});
				// }
				
				if !is_busy && ui.button("Reload Mods").clicked {
					self.refresh();
				}
				
				ui.add_space(16.0);
				
				let queue_size = backend.apply_queue_size();
				ui.enabled(!is_busy && queue_size > 0 , |ui| {
					ui.label(format!("{queue_size} mods have changes that might require an apply"));
					if ui.button("Apply").clicked {
						backend.finalize_apply(apply_progress.clone());
					}
				});
				
				// ui.combo("Active Collection", &self.collections[&self.active_collection], |ui| {
				
				ui.add_space(16.0);
				
				for m in &self.mods {
					if let Some(meta) = backend.get_mod_meta(m) {
						ui.push_id(m, |ui| {
							if ui.selectable(&meta.name, self.selected_mod == *m).clicked {
								self.selected_mod = m.clone();
							}
						});
					}
				}
			}
			
			ui_right.mark_next_splitter();
			
			{
				use crate::modman::{meta::OptionSettings, settings::Value::*, backend::SettingsType};
				
				let ui = ui_right;
				
				ui.horizontal(|ui| {
					let offset = ui.available_size()[0] - 150.0;
					ui.add_space(offset);
					ui.set_width(150.0);
					ui.combo("", self.collections.get(&self.active_collection).map_or("Invalid Collection", |v| v.as_str()), |ui| {
						for (id, name) in &self.collections {
							if ui.selectable(name, self.active_collection == *id).clicked {
								self.active_collection = id.clone();
							}
						}
					});
				});
				
				let Some(meta) = backend.get_mod_meta(&self.selected_mod) else {return};
				let Some(mod_settings) = self.mod_settings.get_mut(&self.selected_mod) else {return};
				// let Some(settings) = mod_settings.get_mut(&self.active_collection) else {return};
				let Some(remote_settings) = self.mod_settings_remote.get_mut(&self.selected_mod) else {return};
				
				let mut presets = mod_settings.presets.clone();
				let settings = mod_settings.get_collection(meta, &self.active_collection);
				
				let mut changed = false;
				
				let warnings = meta.requirements.iter().filter_map(|v| match v.get_status() {
					crate::modman::requirement::Status::Ok => None,
					crate::modman::requirement::Status::Warning(msg) => Some(msg),
				}).collect::<Vec<_>>();
				if warnings.len() > 0 {
					for msg in warnings {
						ui.label_frame(msg, [255, 0, 0, 255]);
					}
					ui.add_space(16.0);
				}
				
				ui.horizontal(|ui| {
					ui.label(&meta.name);
					ui.label(format!("({})", meta.version))
				});
				ui.add_space(16.0);
				
				if !meta.description.is_empty() {
					ui.label(&meta.description);
					ui.add_space(16.0);
				}
				
				if !remote_settings.origin.is_empty() && ui.checkbox("Auto Update", &mut remote_settings.auto_update).changed {
					remote_settings.save(&self.selected_mod);
				}
				
				let mut selected_preset = "Custom".to_string();
				'default: {
					for (name, value) in settings.iter() {
						if let Some(opt) = meta.options.options_iter().find(|v| v.name == *name) {
							if crate::modman::settings::Value::from_meta_option(opt) != *value {break 'default}
						}
					}
					
					selected_preset = "Default".to_owned();
				}
				
				let mut check_presets = |presets: &Vec<crate::modman::settings::Preset>| {
					'preset: for v in presets.iter() {
						for (name, value) in settings.iter() {
							match v.settings.get(name) {
								Some(v) => if v != value {continue 'preset},
								None => if crate::modman::settings::Value::from_meta_option(meta.options.options_iter().find(|v| v.name == *name).unwrap()) != *value {continue 'preset}
							}
						}
						
						selected_preset = v.name.to_owned();
					}
				};
				check_presets(&meta.presets);
				check_presets(&presets);
				
				ui.combo("Preset", &selected_preset, |ui| {
					let mut set_settings = |values: &HashMap<String, crate::modman::settings::Value>| {
						for (name, value) in settings.iter_mut() {
							*value = values.get(name).map_or_else(|| crate::modman::settings::Value::from_meta_option(meta.options.options_iter().find(|v| v.name == *name).unwrap()), |v| v.to_owned());
						}
						
						changed = true;
					};
					
					if meta.presets.len() == 0 {
						if ui.selectable("Default", "Default" == selected_preset).clicked {
							set_settings(&HashMap::new());
						}
					}
					
					for p in &meta.presets {
						if ui.selectable(&p.name, p.name == selected_preset).clicked {
							set_settings(&p.settings);
						}
					}
					
					let mut delete = None;
					for (i, p) in presets.iter().enumerate() {
						ui.horizontal(|ui| {
							ui.push_id(i, |ui| {
								let resp = ui.button("D");
								if resp.clicked {
									delete = Some(i);
								}
								if resp.hovered {
									ui.tooltip_text("Delete");
								}
								
								let resp = ui.button("C");
								if resp.clicked {
									if let Ok(json) = serde_json::to_vec(p) {
										log!("copied {}", base64::Engine::encode(&base64::prelude::BASE64_STANDARD_NO_PAD, &json));
										ui.set_clipboard(base64::Engine::encode(&base64::prelude::BASE64_STANDARD_NO_PAD, json));
									}
								}
								if resp.hovered {
									ui.tooltip_text("Copy to clipboard");
								}
								
								if ui.selectable(&p.name, p.name == selected_preset).clicked {
									set_settings(&p.settings);
								}
							});
						});
					}
					
					if let Some(delete) = delete {
						presets.remove(delete);
						changed = true;
					}
					
					ui.horizontal(|ui| {
						if ui.button("+").clicked && self.new_preset_name.len() > 0 && self.new_preset_name != "Custom" && self.new_preset_name != "Default" {
							let preset = crate::modman::settings::Preset {
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
						ui.input_text("", &mut self.new_preset_name);
					});
					
					if ui.button("Import preset from clipboard").clicked {'import: {
						let json = match base64::Engine::decode(&base64::prelude::BASE64_STANDARD_NO_PAD, ui.get_clipboard().trim().trim_end_matches(|v| v == '=')) {
							Ok(v) => v,
							Err(err) => {log!(err, "Error importing preset ({err:?})"); break 'import}
						};
						
						let preset = match serde_json::from_slice::<crate::modman::settings::Preset>(&json) {
							Ok(v) => v,
							Err(err) => {log!(err, "Error importing preset ({err:?})"); break 'import}
						};
						
						if preset.name.len() == 0 || preset.name == "Custom" || preset.name == "Default" {
							log!(err, "Error importing preset (Invalid name)");
							break 'import;
						}
						
						if let Some(existing) = presets.iter_mut().find(|v| v.name == preset.name) {
							*existing = preset;
						} else {
							presets.push(preset);
						}
						
						changed = true;
					}}
				});
				
				ui.add_space(16.0);
				
				let mut categories = meta.options.categories_iter().collect::<Vec<_>>();
				if !matches!(meta.options.0.get(0), Some(crate::modman::meta::OptionType::Category(_))) {
					categories.insert(0, "Main");
				}
				
				if categories.len() > 1 {
					ui.tabs(&categories, &mut self.selected_category_tab);
					if !categories.contains(&self.selected_category_tab.as_str()) {
						self.selected_category_tab = categories[0].to_string();
					}
				}
				
				let mut cur_category = "Main";
				for option_type in meta.options.0.iter() {
					let option = match option_type {
						crate::modman::meta::OptionType::Option(v) => v,
						crate::modman::meta::OptionType::Category(s) => {cur_category = s.as_ref(); continue},
					};
					
					if categories.len() > 1 && cur_category != self.selected_category_tab {continue}
					
					let setting_id = &option.name;
					let val = settings.get_mut(setting_id).unwrap();
					
					match val {
						SingleFiles(val) => {
							let OptionSettings::SingleFiles(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); continue};
							ui.horizontal(|ui| {
								ui.combo(&option.name, o.options.get(*val as usize).map_or("Invalid", |v| &v.name), |ui| {
									for (i, sub) in o.options.iter().enumerate() {
										ui.horizontal(|ui| {
											changed |= ui.selectable_value(&sub.name, val,i as u32).clicked;
											if !sub.description.is_empty() {
												ui.helptext(&sub.description);
											}
										});
									}
								});
								
								if !option.description.is_empty() {
									ui.helptext(&option.description);
								}
							});
						}
						
						MultiFiles(val) => {
							let OptionSettings::MultiFiles(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); continue};
							ui.horizontal(|ui| {
								ui.label(&option.name);
								if !option.description.is_empty() {
									ui.helptext(&option.description);
								}
							});
							
							ui.indent(|ui| {
								for (i, sub) in o.options.iter().enumerate() {
									ui.horizontal(|ui| {
										let mut toggled = *val & (1 << i) != 0;
										if ui.checkbox(&sub.name, &mut toggled).changed {
											*val ^= 1 << i;
											changed = true;
										}
										
										if !sub.description.is_empty() {
											ui.helptext(&sub.description);
										}
									});
								}
							});
						}
						
						Rgb(val) => {
							let OptionSettings::Rgb(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); continue};
							ui.horizontal(|ui| {
								changed |= ui.color_edit_rgb(&option.name, val).changed;
								for (i, v) in val.iter_mut().enumerate() {*v = v.clamp(o.min[i], o.max[i])}
								if !option.description.is_empty() {
									ui.helptext(&option.description);
								}
							});
						}
						
						Rgba(val) => {
							let OptionSettings::Rgba(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); continue};
							ui.horizontal(|ui| {
								changed |= ui.color_edit_rgba(&option.name, val).changed;
								for (i, v) in val.iter_mut().enumerate() {*v = v.clamp(o.min[i], o.max[i])}
								if !option.description.is_empty() {
									ui.helptext(&option.description);
								}
							});
						}
						
						Grayscale(val) => {
							let OptionSettings::Grayscale(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); continue};
							changed |= ui.slider(&option.name, val, o.min..=o.max).changed;
							*val = val.clamp(o.min, o.max);
							if !option.description.is_empty() {
								ui.helptext(&option.description);
							}
						}
						
						Opacity(val) => {
							let OptionSettings::Grayscale(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); continue};
							changed |= ui.slider(&option.name, val, o.min..=o.max).changed;
							*val = val.clamp(o.min, o.max);
							if !option.description.is_empty() {
								ui.helptext(&option.description);
							}
						}
						
						Mask(val) => {
							let OptionSettings::Grayscale(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); continue};
							changed |= ui.slider(&option.name, val, o.min..=o.max).changed;
							*val = val.clamp(o.min, o.max);
							if !option.description.is_empty() {
								ui.helptext(&option.description);
							}
						}
						
						Path(val) => {
							let OptionSettings::Path(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); continue};
							ui.horizontal(|ui| {
								ui.combo(&option.name, o.options.get(*val as usize).map_or("Invalid", |v| &v.0), |ui| {
									for (i, (name, _)) in o.options.iter().enumerate() {
										changed |= ui.selectable_value(name, val, i as u32).clicked;
									}
								});
								
								if !option.description.is_empty() {
									ui.helptext(&option.description);
								}
							});
						}
					}
				}
				
				// ui.enabled(!is_busy, |ui| {
				// 	if ui.button("Apply").clicked {
				// 		backend.apply_mod_settings(&self.selected_mod, &self.active_collection, SettingsType::Some(settings.clone()));
				// 		backend.finalize_apply(apply_progress.clone())
				// 	}
				// });
				
				ui.add_space(32.0);
				if let Some(style) = &meta.plugin_settings.dalamud {
					ui.horizontal(|ui| {
						if ui.button("Set Dalamud Style").clicked {
							let gamma = 1.2 - (self.gamma as f32 / 250.0);
							style.apply(&meta.name, meta, settings, gamma);
						}
						
						ui.set_width(150.0);
						ui.slider("Gamma", &mut self.gamma, 0..=100);
						ui.helptext("Set this to your game gamma setting");
					});
				}
				
				if changed {
					backend.apply_mod_settings(&self.selected_mod, &self.active_collection, SettingsType::Some(settings.clone()));
					mod_settings.presets = presets;
					mod_settings.save(&self.selected_mod);
				}
			}
		});
		
		_ = config.save();
	}
}
