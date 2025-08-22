use std::{collections::{HashMap, HashSet}, io::Write, sync::{Arc, Mutex}};
use noumenon::format::{external::Bytes, game::Tex};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::ui_ext::UiExt;

pub struct Creator {
	collection: String,
	name: String,
	bc7: bool,
	files: (HashMap<String, (String, std::path::PathBuf)>, HashMap<String, (String, String)>, Vec<(String, serde_json::Value)>),
	mods: Vec<(String, bool)>,
	progress: crate::modman::backend::Progress,
}

impl Creator {
	pub fn new() -> Self {
		let mut s = Self {
			collection: crate::config().config.active_collection.clone(),
			name: "Player Collection".to_string(),
			bc7: true,
			files: (HashMap::new(), HashMap::new(), Vec::new()),
			mods: Vec::new(),
			progress: crate::modman::backend::Progress::new(),
		};
		
		s.refresh_mods();
		s
	}
	
	fn refresh_mods(&mut self) {
		self.files = crate::backend().get_collection_merged(&self.collection);
		let mut mods = HashSet::new();
		for (_, (mod_id, _)) in &self.files.0 {
			mods.insert(mod_id.clone());
		}
		for (_, (mod_id, _)) in &self.files.1 {
			mods.insert(mod_id.clone());
		}
		for (mod_id, _) in &self.files.2 {
			mods.insert(mod_id.clone());
		}
		self.mods = mods.into_iter().map(|v| (v, true)).collect();
		self.mods.sort_by(|a, b| a.0.cmp(&b.0));
	}
	
	fn create_modpack(&self) {
		let (files, swaps, manips) = self.files.clone();
		let mods = self.mods.iter().filter_map(|(mod_id, enabled)| if *enabled {Some(mod_id.clone())} else {None}).collect::<HashSet<_>>();
		let files = files.into_iter().filter_map(|(a, (mod_id, b))| if mods.contains(&mod_id) {Some((a, b))} else {None}).collect::<HashMap<_, _>>();
		let swaps = swaps.into_iter().filter_map(|(a, (mod_id, b))| if mods.contains(&mod_id) {Some((a, b))} else {None}).collect::<HashMap<_, _>>();
		let manips = manips.into_iter().filter_map(|(mod_id, a)| if mods.contains(&mod_id) {Some(a)} else {None}).collect::<Vec<_>>();
		
		log!("{} files; {} swaps; {} manips", files.len(), swaps.len(), manips.len());
		
		let name = self.name.clone();
		let progress = self.progress.clone();
		let bc7 = self.bc7;
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
				
				// cannot be shared between threads safely, aight unsafe it is
				let pack_box = Box::new(pack);
				let pack_ptr = Box::into_raw(pack_box);
				let info = Arc::new(Mutex::new((0, pack_ptr as usize, HashMap::new(), HashSet::new())));
				files.into_par_iter()
					.for_each_with(info.clone(), |info, (game_path, real_path)| {
						let mut i = info.lock().unwrap();
						i.0 += 1;
						progress.set(i.0 as f32 / total as f32 - 0.001);
						drop(i);
						
						if !real_path.exists() {return};
						let Ok(mut data) = std::fs::read(real_path) else {return};
						let hash = crate::hash_str(blake3::hash(&data));
						let ext = game_path.split(".").last().unwrap();
						let path = format!("files/{hash}.{ext}");
						
						let mut i = info.lock().unwrap();
						let contained = i.3.contains(&path);
						i.2.insert(game_path.clone(), path.clone());
						i.3.insert(path.clone());
						drop(i);
						
						if !contained {
							if bc7 && ext == "tex" {'o: {
								let Ok(mut tex) = Tex::read(&mut std::io::Cursor::new(&data)) else {break 'o};
								if matches!(tex.format, noumenon::format::game::tex::Format::Bc7) {break 'o};
								tex.format = noumenon::format::game::tex::Format::Bc7;
								let mut new_data = Vec::new();
								if tex.write(&mut std::io::Cursor::new(&mut new_data)).is_err() {break 'o};
								data = new_data;
							}}
							
							let i = info.lock().unwrap();
							let pack = unsafe{&mut *(i.1 as *mut zip::ZipWriter<std::io::BufWriter<std::fs::File>>)};
							pack.start_file(path.clone(), options).unwrap();
							pack.write_all(&data).unwrap();
							drop(i);
						}
					});
				
				let mut pack = unsafe{Box::from_raw(pack_ptr)};
				let (_, _, file_map, _) = Mutex::into_inner(Arc::into_inner(info).unwrap()).unwrap();
				
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
					self.refresh_mods();
				}
			}
		});
		ui.spacer();
		
		ui.label("Mod Name");
		ui.text_edit_singleline(&mut self.name);
		ui.spacer();
		
		ui.checkbox(&mut self.bc7, "Compress textures to bc7 (slower but will result in a way smaller modpack)");
		ui.spacer();
		
		ui.collapsing("Mods filter", |ui| {
			ui.horizontal(|ui| {
				if ui.button("Enable All").clicked() {
					for (_, b) in &mut self.mods {
						*b = true;
					}
				}
				
				if ui.button("Disable All").clicked() {
					for (_, b) in &mut self.mods {
						*b = false;
					}
				}
			});
			
			for (mod_id, b) in &mut self.mods {
				ui.checkbox(b, &*mod_id);
			}
		});
		ui.spacer();
		
		ui.label(format!("Pmp modpack will be created at: {:?}", dirs::document_dir().unwrap().join(format!("{}.pmp", self.name))));
		if ui.button("Create pmp modpack").clicked() {
			self.create_modpack();
		}
	}
}