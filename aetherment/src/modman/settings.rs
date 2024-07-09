use std::{collections::HashMap, io::Write, ops::{Deref, DerefMut}, path::Path};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Settings {
	pub settings: HashMap<String, Value>,
	pub presets: Vec<Preset>,
}

impl Settings {
	pub fn exists(mod_id: &str, collection: &str) -> bool {
		let collection_hash = crate::hash_str(blake3::hash(collection.as_bytes()));
		let id_hash = crate::hash_str(blake3::hash(mod_id.as_bytes()));
		dirs::config_dir().ok_or("No Config Dir (???)").unwrap().join("Aetherment").join("mods").join(collection_hash).join(id_hash).exists()
	}
	
	pub fn from_meta(meta: &super::meta::Meta) -> Self {
		Self {
			settings: meta.options.iter().map(|option| (option.name.clone(), Value::from_meta_option(option))).collect(),
			presets: Vec::new(),
		}
	}
	
	pub fn open_from(meta: &super::meta::Meta, path: &Path) -> Self {
		let mut settings = Self::from_meta(meta);
		if let Ok(s) = crate::resource_loader::read_json::<Self>(path) {
			settings.presets = s.presets;
			for (k, v) in s.settings {
				if let Some(k) = settings.get_mut(&k) {
					*k = v;
				}
			}
		}
		
		settings
	}
	
	pub fn open(meta: &super::meta::Meta, mod_id: &str, collection: &str) -> Self {
		let collection_hash = crate::hash_str(blake3::hash(collection.as_bytes()));
		let id_hash = crate::hash_str(blake3::hash(mod_id.as_bytes()));
		
		let dir = dirs::config_dir().ok_or("No Config Dir (???)").unwrap().join("Aetherment").join("mods").join(collection_hash);
		Self::open_from(meta, &dir.join(id_hash))
	}
	
	pub fn save_to(&self, path: &Path) {
		let mut f = std::fs::File::create(path).unwrap();
		f.write_all(crate::json_pretty(&self).unwrap().as_bytes()).unwrap()
	}
	
	pub fn save(&self, mod_id: &str, collection: &str) {
		let collection_hash = crate::hash_str(blake3::hash(collection.as_bytes()));
		let id_hash = crate::hash_str(blake3::hash(mod_id .as_bytes()));
		
		let dir = dirs::config_dir().ok_or("No Config Dir (???)").unwrap().join("Aetherment").join("mods").join(collection_hash);
		_ = std::fs::create_dir_all(&dir);
		
		self.save_to(&dir.join(id_hash));
	}
}

impl Deref for Settings {
	type Target = HashMap<String, Value>;
	
	fn deref(&self) -> &Self::Target {
		&self.settings
	}
}

impl DerefMut for Settings {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.settings
	}
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Preset {
	pub name: String,
	pub settings: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Value {
	SingleFiles(u32),
	MultiFiles(u32),
	Rgb([f32; 3]),
	Rgba([f32; 4]),
	Grayscale(f32),
	Opacity(f32),
	Mask(f32),
	Path(u32),
}

impl Value {
	pub fn from_meta_option(option: &super::meta::Option) -> Self {
		match &option.settings {
			super::meta::OptionSettings::SingleFiles(v) => Self::SingleFiles(v.default),
			super::meta::OptionSettings::MultiFiles(v) => Self::MultiFiles(v.default),
			super::meta::OptionSettings::Rgb(v) => Self::Rgb(v.default),
			super::meta::OptionSettings::Rgba(v) => Self::Rgba(v.default),
			super::meta::OptionSettings::Grayscale(v) => Self::Grayscale(v.default),
			super::meta::OptionSettings::Opacity(v) => Self::Opacity(v.default),
			super::meta::OptionSettings::Mask(v) => Self::Mask(v.default),
			super::meta::OptionSettings::Path(v) => Self::Path(v.default),
		}
	}
}