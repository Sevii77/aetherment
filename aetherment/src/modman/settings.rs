use std::{ops::{Deref, DerefMut}, io::Write, path::Path};
use serde::{Deserialize, Serialize};

// TODO: custom implement deserialize/serialize to not need to do that in those functions
// TODO: store all collections in a single file incase of scuffed collection names
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Settings(Vec<(String, Value)>);

impl Settings {
	pub fn from_meta(meta: &super::meta::Meta) -> Self {
		Self(meta.options.iter().map(|option| (option.name.clone(), Value::from_meta_option(option))).collect())
	}
	
	pub fn open_from(meta: &super::meta::Meta, path: &Path) -> Self {
		let mut settings = Self::from_meta(meta);
		if let Ok(s) = crate::resource_loader::read_json::<std::collections::HashMap<String, Value>>(path) {
			for (k, v) in s {
				// settings.insert(k, v);
				settings.iter_mut().find(|v| v.0 == k).map(|dv| dv.1 = v);
			}
		}
		
		settings
	}
	
	pub fn open(meta: &super::meta::Meta, collection: &str) -> Self {
		let dir = dirs::config_dir().ok_or("No Config Dir (???)").unwrap().join("Aetherment").join("mods").join(collection);
		Self::open_from(meta, &dir.join(&meta.name))
	}
	
	pub fn save_to(&self, path: &Path) {
		let mut f = std::fs::File::create(path).unwrap();
		f.write_all(crate::json_pretty(&self.0.iter().map(|v| (v.0.as_str(), &v.1)).collect::<std::collections::HashMap<&str, &Value>>()).unwrap().as_bytes()).unwrap();
	}
	
	pub fn save(&self, name: &str, collection: &str) {
		let dir = dirs::config_dir().ok_or("No Config Dir (???)").unwrap().join("Aetherment").join("mods").join(collection);
		_ = std::fs::create_dir_all(&dir);
		
		self.save_to(&dir.join(name));
	}
	
	pub fn get(&self, id: &str) -> Option<&Value> {
		self.iter().find(|(k, _)| k == id).map(|(_, v)| v)
	}
}

impl Deref for Settings {
	// type Target = std::collections::HashMap<String, Value>;
	type Target = Vec<(String, Value)>;
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Settings {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
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