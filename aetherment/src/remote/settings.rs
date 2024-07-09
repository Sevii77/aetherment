use std::{io::Write, path::Path};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Settings {
	pub auto_update: bool,
	pub origin: String,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			auto_update: true,
			origin: String::new(),
		}
	}
}

impl Settings {
	pub fn exists(mod_id: &str) -> bool {
		let id_hash = crate::hash_str(blake3::hash(mod_id.as_bytes()));
		dirs::config_dir().ok_or("No Config Dir (???)").unwrap().join("Aetherment").join("remote").join(id_hash).exists()
	}
	
	pub fn open_from(path: &Path) -> Self {
		crate::resource_loader::read_json::<Self>(path).unwrap_or_default()
	}
	
	pub fn open(mod_id: &str) -> Self {
		let id_hash = crate::hash_str(blake3::hash(mod_id.as_bytes()));
		
		let dir = dirs::config_dir().ok_or("No Config Dir (???)").unwrap().join("Aetherment").join("remote");
		Self::open_from(&dir.join(id_hash))
	}
	
	pub fn save_to(&self, path: &Path) {
		let mut f = std::fs::File::create(path).unwrap();
		f.write_all(crate::json_pretty(&self).unwrap().as_bytes()).unwrap()
	}
	
	pub fn save(&self, mod_id: &str) {
		let id_hash = crate::hash_str(blake3::hash(mod_id .as_bytes()));
		
		let dir = dirs::config_dir().ok_or("No Config Dir (???)").unwrap().join("Aetherment").join("remote");
		_ = std::fs::create_dir_all(&dir);
		
		self.save_to(&dir.join(id_hash));
	}
}