use std::{collections::HashMap, io::Write};
use crate::ui_ext::UiExt;

pub struct Creator {
	collection: String,
	name: String,
	progress: crate::modman::backend::Progress,
}

impl Creator {
	pub fn new() -> Self {
		Self {
			collection: crate::config().config.active_collection.clone(),
			name: "Player Collection".to_string(),
			progress: crate::modman::backend::Progress::new(),
		}
	}
	
	fn create_modpack(&self) {
		let (files, swaps, manips) = crate::backend().get_collection_merged(&self.collection);
		log!("{} files; {} swaps; {} manips", files.len(), swaps.len(), manips.len());
		
		let name = self.name.clone();
		let progress = self.progress.clone();
		progress.set_msg("Creating mod");
		
		std::thread::spawn(move || {
			let total = files.len();
			let r = (|| -> Result<(), crate::resource_loader::BacktraceError> {
				let path = dirs::document_dir().ok_or("No documents directory")?.join(format!("{}.pmp", name));
				let file = std::fs::File::create(&path)?;
				let mut pack = zip::ZipWriter::new(std::io::BufWriter::new(file));
				
				let options = zip::write::FileOptions::default()
					.compression_method(zip::CompressionMethod::Deflated)
					.compression_level(Some(5))
					.large_file(true);
				
				pack.add_directory("files", options)?;
				
				pack.start_file("meta.json", options)?;
				pack.write_all(crate::json_pretty(&serde_json::json!({
					"FileVersion": 3,
					"Name": name,
					"Author": "",
					"Description": "A collection packed into a single mod",
					"Version": "1.0",
					"Website": "",
					"ModTags": ["Collection"],
				}))?.as_bytes())?;
				
				let mut file_map = HashMap::new();
				for (i, (game_path, real_path)) in files.into_iter().enumerate() {
					// log!(inf, "{game_path}: {real_path:?}");
					
					progress.set(i as f32 / total as f32);
					if !real_path.exists() {continue};
					
					let Ok(data) = std::fs::read(real_path) else {continue};
					let hash = crate::hash_str(blake3::hash(&data));
					let ext = game_path.split(".").last().unwrap();
					let path = format!("files/{hash}.{ext}");
					
					file_map.insert(game_path, path.clone());
					
					pack.start_file(path, options)?;
					pack.write_all(&data)?;
				}
				
				pack.start_file("default_mod.json", options)?;
				pack.write_all(crate::json_pretty(&serde_json::json!({
					"Files": file_map,
					"FileSwap": swaps,
					"Manipulations": manips,
				}))?.as_bytes())?;
				
				pack.finish()?;
				
				progress.set_msg(format!("Finished, mod at {path:?}"));
				
				Ok(())
			})();
			
			if let Err(err) = r {
				progress.set_msg(format!("Failed creating mod {err:?}"));
			}
			
			progress.set(1.0);
		});
	}
}

impl super::super::View for Creator {
	fn title(&self) -> &'static str {
		"Collection Modpack Creator"
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, _renderer: &renderer::Renderer) {
		let progress = self.progress.get();
		if progress > 0.0 {
			ui.label(format!("{} {:.0}%", self.progress.get_msg(), progress * 100.0));
			if progress >= 1.0 && ui.button("Ok").clicked() {
				self.progress.set(0.0);
			}
			
			return;
		}
		
		let collections = crate::backend().get_collections();
		ui.combo_id(collections.iter().find(|v| v.id == self.collection).map_or("Invalid Collection", |v| v.name.as_str()), "collection", |ui| {
			for collection in &collections {
				if ui.selectable_label(self.collection == collection.id, &collection.name).clicked() {
					self.collection = collection.id.clone();
				}
			}
		});
		ui.spacer();
		
		ui.label("Mod Name");
		ui.text_edit_singleline(&mut self.name);
		ui.spacer();
		
		ui.label(format!("Pmp modpack will be created at: {:?}", dirs::document_dir().unwrap().join(format!("{}.pmp", self.name))));
		if ui.button("Create pmp modpack").clicked() {
			self.create_modpack();
		}
	}
}