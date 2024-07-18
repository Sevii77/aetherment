use std::sync::{Arc, RwLock};

pub struct Browser {
	busy: Arc<RwLock<bool>>,
	mods: Arc<RwLock<Vec<(usize, crate::remote::ModEntry)>>>,
}

impl Browser {
	pub fn new() -> Self {
		let s = Self {
			busy: Arc::new(RwLock::new(false)),
			mods: Arc::new(RwLock::new(Vec::new())),
		};
		
		s.load_mods();
		s
	}
	
	pub fn draw(&mut self, ui: &mut renderer::Ui, install_progress: crate::modman::backend::InstallProgress) {
		let is_busy = *self.busy.read().unwrap();
		for (selected_version, m) in self.mods.write().unwrap().iter_mut() {
			// ui.child(&m.id, [0.0, 0.0], |ui| {
			ui.push_id(&m.id, |ui| {
				ui.horizontal(|ui| {
					ui.label(&m.name);
					ui.label(format!("(by {})", m.author))
				});
				ui.label(&m.description);
				ui.horizontal(|ui| {
					if !is_busy && ui.button("Install").clicked {
						self.download_mod(m.id.clone(), m.versions[*selected_version].clone(), install_progress.clone());
					}
					
					ui.combo("Version", &m.versions[*selected_version], |ui| {
						for (i, version) in m.versions.iter().enumerate() {
							ui.selectable_value(version, selected_version, i);
						}
					});
				});
			});
			
			ui.add_space(32.0);
		}
	}
	
	fn load_mods(&self) {
		let busy = self.busy.clone();
		if *busy.read().unwrap() {return};
		
		let mods = self.mods.clone();
		*busy.write().unwrap() = true;
		std::thread::spawn(move || {
			if let Ok(m) = crate::remote::get_mods() {
				let mut new = m.into_iter().map(|v| (0, v)).collect::<Vec<_>>();
				new.sort_by(|(_, ma), (_, mb)| ma.name.cmp(&mb.name));
				
				*mods.write().unwrap() = new;
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