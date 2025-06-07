use std::collections::{HashMap, HashSet};
use crate::{modman::settings::Settings, ui_ext::UiExt};

pub struct Mods {
	install_progress: crate::modman::backend::InstallProgress,
	apply_progress: crate::modman::backend::ApplyProgress,
	
	active_collection: String,
	selected_mod: String,
	selected_category_tab: String,
	gamma: u8,
	
	import_picker: Option<egui_file::FileDialog>,
	new_preset_name: String,
	
	last_was_busy: bool,
	mods: Vec<String>,
	collections: HashMap<String, String>,
	// mod_settings: HashMap<String, HashMap<String, Settings>>,
	mod_settings: HashMap<String, Settings>,
	mod_settings_remote: HashMap<String, crate::remote::settings::Settings>,
}

impl Mods {
	pub fn new(install_progress: crate::modman::backend::InstallProgress, apply_progress: crate::modman::backend::ApplyProgress) -> Self {
		let mut s = Self {
			install_progress,
			apply_progress,
			
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
}

impl super::View for Mods {
	fn title(&self) -> &'static str {
		"Mods"
	}

	fn ui(&mut self, ui: &mut egui::Ui) {
		let backend = crate::backend();
		let config = crate::config();
		config.mark_for_changes();
		let is_busy = self.install_progress.is_busy() || self.apply_progress.is_busy();
		if self.last_was_busy && !is_busy {
			self.refresh();
		}
		self.last_was_busy = is_busy;
		
		crate::ui_ext::Splitter::new("splitter", crate::ui_ext::SplitterAxis::Horizontal)
			.default_pos(0.3)
			.show(ui, |ui_left, ui_right| {
			let h = ui_left.available_height() - 20.0;
			egui::ScrollArea::vertical().id_salt("left").auto_shrink(false).max_height(h).show(ui_left, |ui| {
				ui.combo(self.collections.get(&self.active_collection).map_or("Invalid Collection", |v| v.as_str()), "", |ui| {
					for (id, name) in &self.collections {
						if ui.selectable_label(self.active_collection == *id, name).clicked {
							self.active_collection = id.clone();
						}
					}
				});
				
				if ui.button("Import Mods").clicked && self.import_picker.is_none() {
					let mut dialog = egui_file::FileDialog::open_file(Some(config.config.file_dialog_path.clone()))
						.title("Import Mods")
						.multi_select(true)
						.filename_filter(Box::new(|name| name.ends_with(".aeth")));
					dialog.open();
					self.import_picker = Some(dialog);
				}
				
				if let Some(picker) = &mut self.import_picker {
					match picker.show(&ui.ctx()).state() {
						egui_file::State::Selected => {
							let progress = self.install_progress.clone();
							let paths = picker.selection().into_iter().map(|v| v.to_path_buf()).collect();
							std::thread::spawn(move || {
								crate::backend().install_mods_path(progress, paths);
							});
							
							config.config.file_dialog_path = picker.directory().to_path_buf();
							self.import_picker = None;
						}
						
						egui_file::State::Cancelled => {
							config.config.file_dialog_path = picker.directory().to_path_buf();
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
				ui.add_enabled_ui(!is_busy && queue_size > 0 , |ui| {
					ui.label(format!("{queue_size} mods have changes that might require an apply"));
					if ui.button("Apply").clicked {
						backend.finalize_apply(self.apply_progress.clone());
					}
				});
				
				// ui.combo("Active Collection", &self.collections[&self.active_collection], |ui| {
				
				ui.add_space(16.0);
				
				for m in &self.mods {
					if let Some(meta) = backend.get_mod_meta(m) {
						ui.push_id(m, |ui| {
							if ui.selectable_label(self.selected_mod == *m, &meta.name).clicked {
								self.selected_mod = m.clone();
							}
						});
					}
				}
			});
			
			if ui_left.add(egui::Button::new(egui::RichText::new("Support me on Buy Me a Coffee").color(egui::Color32::from_rgb(0, 0, 0))).fill(egui::Color32::from_rgb(254, 210, 0))).clicked {
				ui_left.ctx().open_url(egui::OpenUrl::new_tab("https://buymeacoffee.com/sevii77"));
			}
			
			egui::ScrollArea::vertical().id_salt("right").auto_shrink(false).show(ui_right, |ui| {
				use crate::modman::{meta::OptionSettings, backend::SettingsType};
				
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
						ui.label(egui::RichText::new(msg).background_color(egui::Color32::from_rgb(255, 0, 0)));
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
				
				if !remote_settings.origin.is_empty() && ui.checkbox(&mut remote_settings.auto_update, "Auto Update").changed {
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
				
				ui.combo(&selected_preset, "Preset", |ui| {
					let mut set_settings = |values: &HashMap<String, crate::modman::settings::Value>| {
						for (name, value) in settings.iter_mut() {
							*value = values.get(name).map_or_else(|| crate::modman::settings::Value::from_meta_option(meta.options.options_iter().find(|v| v.name == *name).unwrap()), |v| v.to_owned());
						}
						
						changed = true;
					};
					
					if meta.presets.len() == 0 {
						if ui.selectable_label("Default" == selected_preset, "Default").clicked {
							set_settings(&HashMap::new());
						}
					}
					
					for p in &meta.presets {
						if ui.selectable_label(p.name == selected_preset, &p.name).clicked {
							set_settings(&p.settings);
						}
					}
					
					let mut delete = None;
					for (i, p) in presets.iter().enumerate() {
						ui.horizontal(|ui| {
							ui.push_id(i, |ui| {
								let resp = ui.button("🗑");
								if resp.clicked {
									delete = Some(i);
								}
								resp.on_hover_text("Delete");
								
								let resp = ui.button("📋");
								if resp.clicked {
									if let Ok(json) = serde_json::to_vec(p) {
										log!("copied {}", base64::Engine::encode(&base64::prelude::BASE64_STANDARD_NO_PAD, &json));
										ui.set_clipboard(base64::Engine::encode(&base64::prelude::BASE64_STANDARD_NO_PAD, json));
									}
								}
								resp.on_hover_text("Copy to clipboard");
								
								if ui.selectable_label(p.name == selected_preset, &p.name).clicked {
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
						ui.text_edit_singleline(&mut self.new_preset_name)
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
					ui.horizontal(|ui| {
						for cat in categories.iter() {
							ui.selectable_value(&mut self.selected_category_tab, cat.to_string(), *cat);
						}
					});
					
					if !categories.contains(&self.selected_category_tab.as_str()) {
						self.selected_category_tab = categories[0].to_string();
					}
				}
				
				let mut grouped_options = HashSet::new();
				for option in meta.options.options_iter() {
					if let OptionSettings::Grouped(group) = &option.settings {
						for sub in group.options.iter() {
							for val in sub.options.iter() {
								if let crate::modman::meta::ValueGroupedOptionEntryType::Option(val) = val {
									for name in val.options.iter() {
										grouped_options.insert(name.as_str());
									}
								}
							}
						}
					}
				}
				
				let mut cur_category = "Main";
				for option_type in meta.options.iter() {
					let option = match option_type {
						crate::modman::meta::OptionType::Option(v) => v,
						crate::modman::meta::OptionType::Category(s) => {cur_category = s.as_ref(); continue},
					};
					
					if grouped_options.contains(option.name.as_str()) {continue}
					if categories.len() > 1 && cur_category != self.selected_category_tab {continue}
					
					changed |= draw_option(ui, meta, settings, option, &option.name, &option.description);
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
						
						ui.slider(&mut self.gamma, 0..=100, "Gamma");
						ui.helptext("Set this to your game gamma setting");
					});
				}
				
				if changed {
					backend.apply_mod_settings(&self.selected_mod, &self.active_collection, SettingsType::Some(settings.clone()));
					mod_settings.presets = presets;
					mod_settings.save(&self.selected_mod);
				}
			});});
		
		_ = config.save();
	}
}

fn draw_option(ui: &mut egui::Ui, meta: &crate::modman::meta::Meta, settings: &mut crate::modman::settings::CollectionSettings, option: &crate::modman::meta::Option, name: &str, desc: &str) -> bool {
	use crate::modman::{meta::OptionSettings, settings::Value::*};
	
	let mut changed = false;
	let setting_id = &option.name;
	let val = settings.get_mut(setting_id).unwrap();
	
	match val {
		Grouped(val) => {
			let OptionSettings::Grouped(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); return false};
			let selected = o.options.get(*val as usize);
			ui.horizontal(|ui| {
				ui.combo(selected.map_or("Invalid", |v| &v.name), name, |ui| {
					for (i, sub) in o.options.iter().enumerate() {
						ui.horizontal(|ui| {
							changed |= ui.selectable_value(val, i as u32, &sub.name).clicked;
							if !sub.description.is_empty() {
								ui.helptext(&sub.description);
							}
						});
					}
				});
				
				if !desc.is_empty() {
					ui.helptext(&*desc);
				}
			});
			
			if let Some(selected) = selected {
				for sub in selected.options.iter() {
					match sub {
						crate::modman::meta::ValueGroupedOptionEntryType::Category(v) => _ = ui.label(v),
						
						crate::modman::meta::ValueGroupedOptionEntryType::Option(v) => {
							if let Some(first) = v.options.first() {
								if let Some(opt) = meta.options.options_iter().find(|x| x.name == *first) {
									if draw_option(ui, meta, settings, opt, &v.name, &v.description) {
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
			let OptionSettings::SingleFiles(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); return false};
			ui.horizontal(|ui| {
				ui.combo(o.options.get(*val as usize).map_or("Invalid", |v| &v.name), name, |ui| {
					for (i, sub) in o.options.iter().enumerate() {
						ui.horizontal(|ui| {
							changed |= ui.selectable_value(val, i as u32, &sub.name).clicked;
							if !sub.description.is_empty() {
								ui.helptext(&sub.description);
							}
						});
					}
				});
				
				if !desc.is_empty() {
					ui.helptext(&*desc);
				}
			});
		}
		
		MultiFiles(val) => {
			let OptionSettings::MultiFiles(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); return false};
			ui.horizontal(|ui| {
				ui.label(name);
				if !desc.is_empty() {
					ui.helptext(&*desc);
				}
			});
			
			ui.indent("", |ui| {
				for (i, sub) in o.options.iter().enumerate() {
					ui.horizontal(|ui| {
						let mut toggled = *val & (1 << i) != 0;
						if ui.checkbox(&mut toggled, &sub.name).changed {
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
			let OptionSettings::Rgb(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); return false};
			ui.horizontal(|ui| {
				changed |= ui.color_edit_button_rgb(val).changed;
				ui.label(name);
				for (i, v) in val.iter_mut().enumerate() {*v = v.clamp(o.min[i], o.max[i])}
				if !desc.is_empty() {
					ui.helptext(&*desc);
				}
			});
		}
		
		Rgba(val) => {
			let OptionSettings::Rgba(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); return false};
			ui.horizontal(|ui| {
				// changed |= ui.color_edit_rgba(name, val).changed;
				changed |= ui.color_edit_button_rgba_unmultiplied(val).changed;
				ui.label(name);
				for (i, v) in val.iter_mut().enumerate() {*v = v.clamp(o.min[i], o.max[i])}
				if !desc.is_empty() {
					ui.helptext(&*desc);
				}
			});
		}
		
		Grayscale(val) => {
			let OptionSettings::Grayscale(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); return false};
			changed |= ui.slider(val, o.min..=o.max, name).changed;
			*val = val.clamp(o.min, o.max);
			if !desc.is_empty() {
				ui.helptext(&*desc);
			}
		}
		
		Opacity(val) => {
			let OptionSettings::Grayscale(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); return false};
			changed |= ui.slider(val, o.min..=o.max, name).changed;
			*val = val.clamp(o.min, o.max);
			if !desc.is_empty() {
				ui.helptext(&*desc);
			}
		}
		
		Mask(val) => {
			let OptionSettings::Grayscale(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); return false};
			changed |= ui.slider(val, o.min..=o.max, name).changed;
			*val = val.clamp(o.min, o.max);
			if !desc.is_empty() {
				ui.helptext(&*desc);
			}
		}
		
		Path(val) => {
			let OptionSettings::Path(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); return false};
			ui.horizontal(|ui| {
				ui.combo(o.options.get(*val as usize).map_or("Invalid", |v| &v.0), name, |ui| {
					for (i, (name, _)) in o.options.iter().enumerate() {
						changed |= ui.selectable_value(val, i as u32, name).clicked;
					}
				});
				
				if !desc.is_empty() {
					ui.helptext(&*desc);
				}
			});
		}
	}
	
	changed
}