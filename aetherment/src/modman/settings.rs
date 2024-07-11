use std::{collections::HashMap, io::Write, ops::{Deref, DerefMut}, path::Path};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct CollectionSettings(HashMap<String, Value>);
impl Deref for CollectionSettings {
	type Target = HashMap<String, Value>;
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for CollectionSettings {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Settings {
	pub collections: HashMap<String, CollectionSettings>,
	pub presets: Vec<Preset>,
}

impl Settings {
	pub fn get_collection(&mut self, meta: &super::meta::Meta, collection_id: &str) -> &mut CollectionSettings {
		let collection_hash = crate::hash_str(blake3::hash(collection_id.as_bytes()));
		self.collections.entry(collection_hash).or_insert_with(|| CollectionSettings(meta.options.options_iter().map(|option| (option.name.clone(), Value::from_meta_option(option))).collect()))
	}
	
	pub fn open_from(meta: &super::meta::Meta, path: &Path) -> Self {
		let mut s = crate::resource_loader::read_json::<Self>(path).unwrap_or_default();
		for (_, c) in s.collections.iter_mut() {
			for o in meta.options.options_iter() {
				if !c.contains_key(&o.name) {
					c.insert(o.name.to_owned(), Value::from_meta_option(o));
				}
			}
		}
		s
	}
	
	pub fn open(meta: &super::meta::Meta, mod_id: &str) -> Self {
		let id_hash = crate::hash_str(blake3::hash(mod_id.as_bytes()));
		
		let dir = dirs::config_dir().ok_or("No Config Dir (???)").unwrap().join("Aetherment").join("mods");
		Self::open_from(meta, &dir.join(id_hash))
	}
	
	pub fn save_to(&self, path: &Path) {
		let mut f = std::fs::File::create(path).unwrap();
		f.write_all(crate::json_pretty(&self).unwrap().as_bytes()).unwrap()
	}
	
	pub fn save(&self, mod_id: &str) {
		let id_hash = crate::hash_str(blake3::hash(mod_id .as_bytes()));
		
		let dir = dirs::config_dir().ok_or("No Config Dir (???)").unwrap().join("Aetherment").join("mods");
		_ = std::fs::create_dir_all(&dir);
		
		self.save_to(&dir.join(id_hash));
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