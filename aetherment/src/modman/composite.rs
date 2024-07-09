use std::{borrow::Cow, collections::HashMap, fs::File, path::Path};
use serde::{Deserialize, Serialize};

pub mod tex;

#[derive(Debug)]
pub enum CompositeError {
	Tex(tex::CompositeError),
	
	BinaryWriter(noumenon::Error),
}

impl From<noumenon::Error> for CompositeError {
	fn from(value: noumenon::Error) -> Self {
		CompositeError::BinaryWriter(value)
	}
}

pub trait Composite {
	fn get_files(&self) -> Vec<&str>;
	fn get_files_game(&self) -> Vec<&str>;
	fn get_options(&self) -> Vec<&str>;
	fn composite<'a>(&self, settings: &crate::modman::settings::CollectionSettings, file_resolver: &dyn Fn(&crate::modman::Path) -> Option<Cow<'a, Vec<u8>>>) -> Result<Vec<u8>, CompositeError>;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompositeCache {
	/// composite_file, [game_files]
	pub composite_external_files: HashMap<String, Vec<String>>,
	
	/// option, [composite_files]
	pub option_composite_files: HashMap<String, Vec<String>>,
}

pub fn build_cache(mod_dir: &Path) -> Result<CompositeCache, crate::resource_loader::BacktraceError> {
	let mut cache = CompositeCache {
		composite_external_files: HashMap::new(),
		option_composite_files: HashMap::new(),
	};
	
	let aeth_dir = mod_dir.join("aetherment");
	let files_dir = mod_dir.join("files");
	let remap = serde_json::from_reader::<_, HashMap<String, String>>(std::io::BufReader::new(File::open(aeth_dir.join("remap"))?))?;
	let meta = serde_json::from_reader::<_, super::meta::Meta>(std::io::BufReader::new(File::open(aeth_dir.join("meta.json"))?))?;
	for (real_path, game_paths) in meta.get_registered_files() {
		if let (Some(real_path_remapped), Some(game_path)) = (remap.get(real_path), game_paths.iter().find(|v| v.ends_with(".comp"))) {
			let ext = game_path.trim_end_matches(".comp").split(".").last().unwrap();
			let comp: Option<Box<dyn Composite>> = match ext {
				"tex" | "atex" => Some(Box::new(serde_json::from_reader::<_, tex::Tex>(std::io::BufReader::new(File::open(files_dir.join(real_path_remapped))?))?)),
				_ => None
			};
			
			if let Some(comp) = comp {
				let files = comp.get_files_game();
				if files.len() > 0 {
					cache.composite_external_files.insert(game_path.to_string(), files.into_iter().map(|v| v.to_owned()).collect());
				}
				
				let options = comp.get_options();
				for option in options {
					cache.option_composite_files.entry(option.to_owned()).or_insert_with(|| Vec::new()).push(game_path.to_string());
				}
			}
		}
	}
	
	std::fs::write(aeth_dir.join("compcache"), serde_json::to_vec(&cache)?)?;
	
	Ok(cache)
}