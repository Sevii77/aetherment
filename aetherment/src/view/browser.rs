use std::{ops::DerefMut, sync::{Arc, Mutex, RwLock}};

use crate::ui_ext::UiExt;

enum BrowseResult {
	Mods(Vec<(usize, crate::remote::ModEntry)>),
	Error(String),
}

pub struct Browser {
	busy: Arc<RwLock<bool>>,
	mods: Arc<Mutex<BrowseResult>>,
	install_progress: crate::modman::backend::InstallProgress
}

impl Browser {
	pub fn new(install_progress: crate::modman::backend::InstallProgress) -> Self {
		let s = Self {
			busy: Arc::new(RwLock::new(false)),
			mods: Arc::new(Mutex::new(BrowseResult::Mods(Vec::new()))),
			install_progress,
		};
		
		s.load_mods();
		s
	}
	
	fn load_mods(&self) {
		let busy = self.busy.clone();
		if *busy.read().unwrap() {return};
		
		let mods = self.mods.clone();
		*busy.write().unwrap() = true;
		*mods.lock().unwrap() = BrowseResult::Mods(Vec::new());
		
		std::thread::spawn(move || {
			match crate::remote::get_mods() {
				Ok(m) => {
					let mut new = m.into_iter().map(|v| (0, v)).collect::<Vec<_>>();
					new.sort_by(|(_, ma), (_, mb)| ma.name.cmp(&mb.name));
					
					*mods.lock().unwrap() = BrowseResult::Mods(new);
				}
				
				Err(err) => {
					*mods.lock().unwrap() = BrowseResult::Error(err.to_string());
				}
			}
			
			*busy.write().unwrap() = false;
		});
	}
	
	fn download_mod(&self, mod_id: String, version: String, progress: crate::modman::backend::InstallProgress) {
		let busy = self.busy.clone();
		if *busy.read().unwrap() {return};
		
		*busy.write().unwrap() = true;
		std::thread::spawn(move || {
			match crate::remote::download(&mod_id, &version) {
				Ok(f) => crate::backend().install_mods(progress, vec![(mod_id, f)]),
				Err(err) => log!("Failed downloading and installing mod ({err})"),
			}
			
			*busy.write().unwrap() = false;
		});
	}
}

impl super::View for Browser {
	fn title(&self) -> &'static str {
		"Browser"
	}
	
	fn ui(&mut self, ui: &mut egui::Ui) {
		let is_busy = *self.busy.read().unwrap();
		match self.mods.lock().unwrap().deref_mut() {
			BrowseResult::Mods(mods) => {
				for (selected_version, m) in mods.iter_mut() {
					// ui.child(&m.id, [0.0, 0.0], |ui| {
					ui.push_id(&m.id, |ui| {
						ui.horizontal(|ui| {
							ui.label(&m.name);
							ui.label(format!("(by {})", m.author))
						});
						ui.label(&m.description);
						ui.horizontal(|ui| {
							if !is_busy && ui.button("Install").clicked {
								self.download_mod(m.id.clone(), m.versions[*selected_version].clone(), self.install_progress.clone());
							}
							
							ui.combo(&m.versions[*selected_version], "Version", |ui| {
								for (i, version) in m.versions.iter().enumerate() {
									ui.selectable_value(selected_version, i, version);
								}
							});
						});
					});
					
					ui.add_space(32.0);
				}
			}
			
			BrowseResult::Error(err) => {
				ui.label(&*err);
			}
		}
		
		if !is_busy && ui.button("Refresh").clicked {
			self.load_mods();
		}
	}
}