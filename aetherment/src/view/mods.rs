use std::collections::HashMap;
use crate::modman::settings::Settings;

pub struct Mods {
	active_collection: String,
	selected_mod: String,
	
	import_picker: Option<renderer::FilePicker>,
	
	mods: Vec<String>,
	collections: HashMap<String, String>,
	mod_settings: HashMap<String, HashMap<String, Settings>>,
	
	install_progress: crate::modman::backend::InstallProgress,
	apply_progress: crate::modman::backend::ApplyProgress,
}

impl Mods {
	pub fn new() -> Self {
		let mut v = Self {
			active_collection: String::new(),
			selected_mod: String::new(),
			
			import_picker: None,
			
			mods: Vec::new(),
			collections: HashMap::new(),
			mod_settings: HashMap::new(),
			
			install_progress: crate::modman::backend::InstallProgress::new(),
			apply_progress: crate::modman::backend::ApplyProgress::new(),
		};
		
		v.install_progress.apply = v.apply_progress.clone();
		v.refresh();
		
		v
	}
	
	fn refresh(&mut self) {
		let backend = crate::backend();
		backend.load_mods();
		
		self.mods = backend.get_mods();
		self.mods.sort_unstable();
		
		self.collections = backend.get_collections().into_iter().map(|v| (v.id, v.name)).collect();
		self.mod_settings = self.mods.iter().map(|m| (m.to_owned(), self.collections.iter().map(|(c, _)| (c.to_owned(), backend.get_mod_settings(m, c).unwrap())).collect())).collect();
		
		if !self.collections.iter().any(|(c, _)| c == &self.active_collection) {
			// self.active_collection = self.collections.iter().find(|_| true).map_or_else(|| String::new(), |(c, _)| c.clone());
			self.active_collection = backend.get_active_collection();
		}
	}
	
	pub fn draw(&mut self, ui: &mut renderer::Ui) {
		let backend = crate::backend();
		let config = crate::config();
		config.mark_for_changes();
		let is_busy = self.install_progress.is_busy() || self.apply_progress.is_busy();
		
		// TOOD: make fancy
		if self.install_progress.is_busy() {
			ui.label(format!("{:.0}% {}", self.install_progress.mods.get() * 100.0, self.install_progress.mods.get_msg()));
			ui.label(format!("{:.0}% {}", self.install_progress.current_mod.get() * 100.0, self.install_progress.current_mod.get_msg()));
		}
		
		if self.apply_progress.is_busy() {
			ui.label(format!("{:.0}% {}", self.apply_progress.mods.get() * 100.0, self.apply_progress.mods.get_msg()));
			ui.label(format!("{:.0}% {}", self.apply_progress.current_mod.get() * 100.0, self.apply_progress.current_mod.get_msg()));
		}
		
		ui.splitter("splitter", 0.3, |ui_left, ui_right| {
			{
				let ui = ui_left;
				if ui.button("Import Mods").clicked && self.import_picker.is_none() {
					self.import_picker = Some(renderer::FilePicker::new("Import Mods", &config.config.file_dialog_path, &[".aeth"], renderer::FilePickerMode::OpenFileMultiple));
				}
				
				if let Some(picker) = &mut self.import_picker {
					match picker.show(ui) {
						renderer::FilePickerStatus::Success(dir, paths) => {
							backend.install_mods(self.install_progress.clone(), paths);
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
				
				if ui.button("Reload Mods").clicked {
					self.refresh();
				}
				
				// ui.combo("Active Collection", &self.collections[&self.active_collection], |ui| {
				ui.combo("Active Collection", self.collections.get(&self.active_collection).map_or("Invalid Collection", |v| v.as_str()), |ui| {
					for (id, name) in &self.collections {
						if ui.selectable(name, self.active_collection == *id).clicked {
							self.active_collection = id.clone();
						}
					}
				});
				
				for m in &self.mods {
					if ui.selectable(&m, self.selected_mod == *m).clicked {
						self.selected_mod = m.clone();
					}
				}
			}
			
			ui_right.mark_next_splitter();
			
			{
				use crate::modman::{meta::OptionSettings, settings::Value::*, backend::SettingsType};
				
				let ui = ui_right;
				
				ui.label(&self.selected_mod);
				
				let Some(meta) = backend.get_mod_meta(&self.selected_mod) else {return};
				let Some(mod_settings) = self.mod_settings.get_mut(&self.selected_mod) else {return};
				let Some(settings) = mod_settings.get_mut(&self.active_collection) else {return};
				
				let mut changed = false;
				for (setting_id, val) in settings.iter_mut() {
					let Some(option) = meta.options.iter().find(|o| o.name == *setting_id) else {continue};
					
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
							ui.label("TODO: Grayscale");
						}
						
						Opacity(val) => {
							ui.label("TODO: Opacity");
						}
						
						Mask(val) => {
							ui.label("TODO: Mask");
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
				
				ui.enabled(!is_busy, |ui| {
					if ui.button("Apply").clicked {
						backend.apply_mod_settings(&self.selected_mod, &self.active_collection, SettingsType::Some(settings.clone()));
						backend.finalize_apply(self.apply_progress.clone())
					}
				});
			}
		});
		
		_ = config.save();
	}
}