use std::{collections::{HashMap, HashSet}, sync::Arc};
use crate::{modman::settings::Settings, ui_ext::UiExt};

pub struct Mods {
	progress: crate::modman::backend::TaskProgress,
	
	selected_mod: String,
	selected_category_tab: String,
	gamma: u8,
	
	import_picker: Option<egui_file::FileDialog>,
	new_preset_name: String,
	
	last_was_busy: bool,
	mods: Vec<Arc<String>>,
	collections: HashMap<String, String>,
	mod_settings: HashMap<Arc<String>, Settings>,
	mod_settings_remote: HashMap<Arc<String>, crate::remote::settings::Settings>,
	markdown_cache: egui_commonmark::CommonMarkCache,
	
	last_viewed: std::time::Instant,
	last_interacted: std::time::Instant,
}

impl Mods {
	pub fn new(progress: crate::modman::backend::TaskProgress) -> Self {
		let mut s = Self {
			progress,
			
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
			markdown_cache: Default::default(),
			
			last_viewed: std::time::Instant::now(),
			last_interacted: std::time::Instant::now(),
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
		self.mod_settings = self.mods.iter().map(|m| (m.to_owned(), crate::modman::settings::Settings::open(&backend.get_mod_meta(m).unwrap(), m))).collect();
		self.mod_settings_remote = self.mods.iter().map(|m| (m.to_owned(), crate::remote::settings::Settings::open(m))).collect();
		
		let config = crate::config();
		if !self.collections.contains_key(&config.config.active_collection) && self.collections.len() > 0 {
			config.config.active_collection = self.collections.keys().next().unwrap().to_owned();
			_ = config.save_forced();
		}
	}
	
	fn check_apply(&self) {
		let config = &crate::config().config;
		if !self.progress.is_finished()  {return}
		if crate::backend().apply_queue_size() == 0 {return}
		if self.last_viewed.elapsed() < config.auto_apply_last_viewed && self.last_interacted.elapsed() < config.auto_apply_last_interacted {return}
		
		let progress = self.progress.clone();
		progress.add_task_count(1);
		std::thread::spawn(move || {
			crate::backend().finalize_apply(progress.clone());
			progress.progress_task();
			if progress.get_messages().iter().any(|v| v.1) {
				crate::set_notification(1.0, 2, "There were issues applying mods");
				return;
			}
			progress.reset();
		});
		
		let progress = self.progress.clone();
		std::thread::spawn(move || {
			// we don't know if a changed mod will actually require us to composite files
			// we check that within the apply, so if it is done within 1 seconds dont
			// bother notifying the user, it'd just be spam
			std::thread::sleep(std::time::Duration::from_secs(1));
			if !progress.is_busy() && !progress.get_messages().iter().any(|v| v.1) {return}
			
			while progress.is_busy() {
				crate::set_notification(progress.get_task_progress(), 0, "Applying mods");
				std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
			}
			
			if progress.get_messages().iter().any(|v| v.1) {
				crate::set_notification(1.0, 2, "There were issues applying mods");
			} else {
				crate::set_notification(1.0, 1, &format!("Mods have been successfully applied"));
			}
		});
	}
	
	fn draw_modlist_simple(&mut self, ui: &mut egui::Ui) {
		let backend = crate::backend();
		let config = crate::config();
		config.mark_for_changes();
		let is_busy = !self.progress.is_finished();
		if self.last_was_busy && !is_busy {
			self.refresh();
		}
		self.last_was_busy = is_busy;
		
		ui.combo(self.collections.get(&config.config.active_collection).map_or("Invalid Collection", |v| v.as_str()), "", |ui| {
			for (id, name) in &self.collections {
				if ui.selectable_label(config.config.active_collection == *id, name).clicked() {
					config.config.active_collection = id.clone();
				}
			}
		});
		
		if ui.button("Import Mods").clicked() && self.import_picker.is_none() {
			let mut dialog = egui_file::FileDialog::open_file(Some(config.config.file_dialog_path.clone()))
				.title("Import Mods")
				.multi_select(true)
				.filename_filter(Box::new(|name| name.ends_with(".aeth")));
			dialog.open();
			self.import_picker = Some(dialog);
		}
		
		if let Some(picker) = &mut self.import_picker {
			match picker.show(ui.ctx()).state() {
				egui_file::State::Selected => {
					let progress = self.progress.clone();
					let paths = picker.selection().into_iter().map(|v| v.to_path_buf()).collect();
					std::thread::spawn(move || {
						progress.add_task_count(1);
						crate::backend().install_mods_path(progress.clone(), paths);
						progress.progress_task();
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
		
		if !is_busy && ui.button("Reload Mods").clicked() {
			self.refresh();
		}
		
		ui.add_space(16.0);
		
		let queue_size = backend.apply_queue_size();
		ui.add_enabled_ui(!is_busy && queue_size > 0 , |ui| {
			ui.label(format!("{queue_size} mods have changes that might require an apply"));
			if ui.button("Manually Apply Now").clicked() {
				let progress = self.progress.clone();
				std::thread::spawn(move || {
					progress.add_task_count(1);
					crate::backend().finalize_apply(progress.clone());
					progress.progress_task();
				});
			}
		});
		
		ui.add_space(16.0);
		
		for m in &self.mods {
			let Some(meta) = backend.get_mod_meta(m) else {continue};
			
			ui.push_id(m, |ui| {
				if !backend.get_mod_enabled(m, &config.config.active_collection) {
					let style = ui.style_mut();
					let color = style.visuals.noninteractive().fg_stroke;
					style.visuals.override_text_color = Some(color.color);
				}
				
				if ui.selectable_label(self.selected_mod == **m, &meta.name).clicked() {
					ui.free_textures("aetherment://");
					self.selected_mod = m.to_string();
				}
			});
		}
		
		_ = config.save();
	}
	
	// fn draw_modlist_unified(&mut self, ui: &mut egui::Ui) {
	// 	
	// }
	
	fn draw_modpage(&mut self, ui: &mut egui::Ui) {
		use crate::modman::{meta::OptionSettings, backend::SettingsType};
		
		let backend = crate::backend();
		let Some(meta) = backend.get_mod_meta(&self.selected_mod) else {return};
		let Some(mod_settings) = self.mod_settings.get_mut(&self.selected_mod) else {return};
		let Some(remote_settings) = self.mod_settings_remote.get_mut(&self.selected_mod) else {return};
		
		let mut presets = mod_settings.presets.clone();
		let settings = mod_settings.get_collection(&meta, &crate::config().config.active_collection);
		
		let mut changed = false;
		
		let warnings = meta.requirements.iter().filter_map(|v| match v.get_status() {
			crate::modman::requirement::Status::Ok => None,
			crate::modman::requirement::Status::Warning(msg) => Some(msg),
		}).collect::<Vec<_>>();
		if warnings.len() > 0 {
			for msg in warnings {
				ui.label(egui::RichText::new(msg).background_color(egui::Color32::RED));
			}
			ui.add_space(16.0);
		}
		
		ui.horizontal(|ui| {
			ui.label(&meta.name);
			ui.label(format!("({})", meta.version))
		});
		ui.add_space(16.0);
		
		if !meta.description.is_empty() {
			draw_description(ui, &self.selected_mod, &meta.description, &mut self.markdown_cache);
			ui.add_space(16.0);
		}
		
		let mut enabled = backend.get_mod_enabled(&self.selected_mod, &crate::config().config.active_collection);
		if ui.checkbox(&mut enabled, "Enabled").changed() {
			backend.set_mod_enabled(&self.selected_mod, &crate::config().config.active_collection, enabled);
		}
		
		if !remote_settings.origin.is_empty() && ui.checkbox(&mut remote_settings.auto_update, "Auto Update").changed() {
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
				if ui.selectable_label("Default" == selected_preset, "Default").clicked() {
					set_settings(&HashMap::new());
				}
			}
			
			for p in &meta.presets {
				if ui.selectable_label(p.name == selected_preset, &p.name).clicked() {
					set_settings(&p.settings);
				}
			}
			
			let mut delete = None;
			for (i, p) in presets.iter().enumerate() {
				ui.horizontal(|ui| {
					ui.push_id(i, |ui| {
						let resp = ui.button("🗑");
						if resp.clicked() {
							delete = Some(i);
						}
						resp.on_hover_text("Delete");
						
						let resp = ui.button("📋");
						if resp.clicked() {
							if let Ok(json) = serde_json::to_vec(p) {
								log!("copied {}", base64::Engine::encode(&base64::prelude::BASE64_STANDARD_NO_PAD, &json));
								ui.set_clipboard(base64::Engine::encode(&base64::prelude::BASE64_STANDARD_NO_PAD, json));
							}
						}
						resp.on_hover_text("Copy to clipboard");
						
						if ui.selectable_label(p.name == selected_preset, &p.name).clicked() {
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
				if ui.button("+").clicked() && self.new_preset_name.len() > 0 && self.new_preset_name != "Custom" && self.new_preset_name != "Default" {
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
			
			if ui.button("Import preset from clipboard").clicked() {'import: {
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
			
			changed |= draw_option(ui, &self.selected_mod, &meta, settings, option, &option.name, &option.description, &mut self.markdown_cache);
		}
		
		ui.add_space(32.0);
		if let Some(style) = &meta.plugin_settings.dalamud {
			ui.horizontal(|ui| {
				if ui.button("Set Dalamud Style").clicked() {
					let gamma = 1.2 - (self.gamma as f32 / 250.0);
					style.apply(&meta.name, &meta, settings, gamma);
				}
				
				ui.slider(&mut self.gamma, 0..=100, "Gamma");
				ui.helptext("Set this to your game gamma setting");
			});
		}
		
		if changed {
			backend.apply_mod_settings(&self.selected_mod, &crate::config().config.active_collection, SettingsType::Some(settings.clone()));
			mod_settings.presets = presets;
			mod_settings.save(&self.selected_mod);
			self.last_interacted = std::time::Instant::now();
		}
	}
}

impl super::View for Mods {
	fn title(&self) -> &'static str {
		"Mods"
	}

	fn ui(&mut self, ui: &mut egui::Ui, viewer: &super::Viewer) {
		if let crate::modman::backend::Status::Error(error) = viewer.backend_status {
			ui.label(error);
			return;
		}
		
		ui.splitter("splitter", crate::ui_ext::SplitterAxis::Horizontal, 0.3, |ui_left, ui_right| {
			let h = ui_left.available_height() - 20.0;
			egui::ScrollArea::vertical()
				.id_salt("left")
				.auto_shrink(false)
				.max_height(h)
				.show(ui_left, |ui| self.draw_modlist_simple(ui));
			
			if ui_left.add(egui::Button::new(egui::RichText::new("Support me on Buy Me a Coffee").color(egui::Color32::from_rgb(0, 0, 0))).fill(egui::Color32::from_rgb(254, 210, 0))).clicked() {
				ui_left.ctx().open_url(egui::OpenUrl::new_tab("https://buymeacoffee.com/sevii77"));
			}
			
			egui::ScrollArea::vertical()
				.id_salt("right")
				.auto_shrink(false)
				.show(ui_right, |ui| self.draw_modpage(ui));
		});
		
		self.last_viewed = std::time::Instant::now();
	}
	
	fn tick(&mut self) {
		self.check_apply();
	}
}

fn draw_description(ui: &mut egui::Ui, mod_id: &str, text: &str, md_cache: &mut egui_commonmark::CommonMarkCache) {
	if text.starts_with("[md]") {
		ui.userspace_loaders(|ui| {
			ui.set_width_range(50.0..=400.0);
			egui_commonmark::CommonMarkViewer::new()
				.default_implicit_uri_scheme(format!("aetherment://{}/", mod_id))
				.show(ui, md_cache, &text[4..]);
		});
	} else {
		ui.label(text);
	}
}

fn draw_help(ui: &mut egui::Ui, mod_id: &str, text: &str, md_cache: &mut egui_commonmark::CommonMarkCache) {
	ui.label("(❓)")
		.on_hover_cursor(egui::CursorIcon::Help)
		.on_hover_ui(|ui| draw_description(ui, mod_id, text, md_cache));
}

fn draw_option(ui: &mut egui::Ui, mod_id: &str, meta: &crate::modman::meta::Meta, settings: &mut crate::modman::settings::CollectionSettings, option: &crate::modman::meta::Option, name: &str, desc: &str, md_cache: &mut egui_commonmark::CommonMarkCache) -> bool {
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
							changed |= ui.selectable_value(val, i as u32, &sub.name).clicked();
							if !sub.description.is_empty() {
								draw_help(ui, mod_id, &sub.description, md_cache);
							}
						});
					}
				});
				
				if !desc.is_empty() {
					draw_help(ui, mod_id, desc, md_cache);
				}
			});
			
			if let Some(selected) = selected {
				for sub in selected.options.iter() {
					match sub {
						crate::modman::meta::ValueGroupedOptionEntryType::Category(v) => _ = ui.label(v),
						
						crate::modman::meta::ValueGroupedOptionEntryType::Option(v) => {
							if let Some(first) = v.options.first() {
								if let Some(opt) = meta.options.options_iter().find(|x| x.name == *first) {
									if draw_option(ui, mod_id, meta, settings, opt, &v.name, &v.description, md_cache) {
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
							changed |= ui.selectable_value(val, i as u32, &sub.name).clicked();
							if !sub.description.is_empty() {
								draw_help(ui, mod_id, &sub.description, md_cache);
							}
						});
					}
				});
				
				if !desc.is_empty() {
					draw_help(ui, mod_id, &*desc, md_cache);
				}
			});
		}
		
		MultiFiles(val) => {
			let OptionSettings::MultiFiles(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); return false};
			ui.horizontal(|ui| {
				ui.label(name);
				if !desc.is_empty() {
					draw_help(ui, mod_id, &*desc, md_cache);
				}
			});
			
			ui.indent("", |ui| {
				for (i, sub) in o.options.iter().enumerate() {
					ui.horizontal(|ui| {
						let mut toggled = *val & (1 << i) != 0;
						if ui.checkbox(&mut toggled, &sub.name).changed() {
							*val ^= 1 << i;
							changed = true;
						}
						
						if !sub.description.is_empty() {
							draw_help(ui, mod_id, &sub.description, md_cache);
						}
					});
				}
			});
		}
		
		Rgb(val) => {
			let OptionSettings::Rgb(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); return false};
			ui.horizontal(|ui| {
				changed |= ui.color_edit(val).changed();
				ui.label(name);
				for (i, v) in val.iter_mut().enumerate() {*v = v.clamp(o.min[i], o.max[i])}
				if !desc.is_empty() {
					draw_help(ui, mod_id, &*desc, md_cache);
				}
			});
		}
		
		Rgba(val) => {
			let OptionSettings::Rgba(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); return false};
			ui.horizontal(|ui| {
				changed |= ui.color_edit(val).changed();
				ui.label(name);
				for (i, v) in val.iter_mut().enumerate() {*v = v.clamp(o.min[i], o.max[i])}
				if !desc.is_empty() {
					draw_help(ui, mod_id, &*desc, md_cache);
				}
			});
		}
		
		Grayscale(val) => {
			let OptionSettings::Grayscale(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); return false};
			ui.horizontal(|ui| {
				changed |= ui.slider(val, o.min..=o.max, name).changed();
				*val = val.clamp(o.min, o.max);
				if !desc.is_empty() {
					draw_help(ui, mod_id, &*desc, md_cache);
				}
			});
		}
		
		Opacity(val) => {
			let OptionSettings::Grayscale(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); return false};
			ui.horizontal(|ui| {
				changed |= ui.slider(val, o.min..=o.max, name).changed();
				*val = val.clamp(o.min, o.max);
				if !desc.is_empty() {
					draw_help(ui, mod_id, &*desc, md_cache);
				}
			});
		}
		
		Mask(val) => {
			let OptionSettings::Grayscale(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); return false};
			ui.horizontal(|ui| {
				changed |= ui.slider(val, o.min..=o.max, name).changed();
				*val = val.clamp(o.min, o.max);
				if !desc.is_empty() {
					draw_help(ui, mod_id, &*desc, md_cache);
				}
			});
		}
		
		Path(val) => {
			let OptionSettings::Path(o) = &option.settings else {ui.label(format!("Invalid setting type for {setting_id}")); return false};
			ui.horizontal(|ui| {
				ui.combo(o.options.get(*val as usize).map_or("Invalid", |v| &v.0), name, |ui| {
					for (i, (name, _)) in o.options.iter().enumerate() {
						changed |= ui.selectable_value(val, i as u32, name).clicked();
					}
				});
				
				if !desc.is_empty() {
					draw_help(ui, mod_id, &*desc, md_cache);
				}
			});
		}
	}
	
	changed
}