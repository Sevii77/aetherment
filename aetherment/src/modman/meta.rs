use std::{collections::HashMap, fs::File, io::Write, path::Path};
use serde::{de::Visitor, Deserialize, Serialize};
use crate::render_helper::EnumTools;

pub mod dalamud;

// TODO: add option_sync or smth so that a submod can sync its options with a master mod (having rounded corners mod for mui needed to be adjusted seperatly is dumb)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Meta {
	pub name: String,
	pub description: String,
	pub version: String,
	pub author: String,
	pub website: String,
	pub tags: Vec<String>,
	pub dependencies: Vec<String>,
	pub options: Options,
	pub presets: Vec<super::settings::Preset>,
	
	pub files: HashMap<String, String>,
	pub file_swaps: HashMap<String, String>,
	pub manipulations: Vec<Manipulation>,
	pub ui_colors: Vec<UiColor>,
	
	pub plugin_settings: PluginSettings,
	
	pub requirements: Vec<super::requirement::Requirement>,
}

impl Default for Meta {
	fn default() -> Self {
		Self {
			name: "New Mod".to_string(),
			description: String::new(),
			version: "1.0.0".to_string(),
			author: String::new(),
			website: String::new(),
			tags: Vec::new(),
			dependencies: Vec::new(),
			options: Options(Vec::new()),
			presets: Vec::new(),
			
			files: HashMap::new(),
			file_swaps: HashMap::new(),
			manipulations: Vec::new(),
			ui_colors: Vec::new(),
			
			plugin_settings: PluginSettings::default(),
			
			requirements: Vec::new(),
		}
	}
}

impl Meta {
	pub fn save(&self, path: &Path) -> std::io::Result<()> {
		// serde_json::to_writer_pretty(&mut File::create(path)?, self)?;
		File::create(path)?.write_all(crate::json_pretty(self)?.as_bytes())?;
		Ok(())
	}
	
	/// This only returns basic registered files (SingleFiles option, MultiFiles option, no option)
	pub fn get_registered_files(&self) -> HashMap<&str, Vec<&str>> {
		let mut paths = HashMap::new();
		for (game, real) in &self.files {
			paths.entry(real.as_str()).or_insert_with(|| Vec::new()).push(game.as_str())
		}
		
		for option in self.options.options_iter() {
			if let OptionSettings::SingleFiles(v) | OptionSettings::MultiFiles(v) = &option.settings {
				for sub in &v.options {
					for (game, real) in &sub.files {
						paths.entry(real.as_str()).or_insert_with(|| Vec::new()).push(game.as_str())
					}
				}
			}
		}
		
		paths
	}
}

// ----------

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct PluginSettings {
	pub dalamud: std::option::Option<dalamud::Style>,
}

#[derive(Default)]
pub struct OptionalInitializers {
	pub dalamud: std::option::Option<fn(&str)>,
}

// ----------

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Options(pub Vec<OptionType>);
impl Options {
	pub fn options_iter(&self) -> impl Iterator<Item = &Option> + DoubleEndedIterator {
		self.0.iter().filter_map(|v| if let OptionType::Option(v) = v {Some(v)} else {None})
	}
	
	pub fn categories_iter(&self) -> impl Iterator<Item = &str> + DoubleEndedIterator {
		self.0.iter().filter_map(|v| if let OptionType::Category(v) = v {Some(v.as_str())} else {None})
	}
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum OptionType {
	Category(String),
	Option(Option),
}

impl<'de> Deserialize<'de> for OptionType {
	fn deserialize<D>(d: D) -> Result<Self, D::Error> where
	D: serde::Deserializer<'de> {
		struct DeVisitor;
		impl<'de> Visitor<'de> for DeVisitor {
			type Value = OptionType;
			
			fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				f.write_str("String or Option")
			}
			
			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where
			E: serde::de::Error, {
				Ok(OptionType::Category(v.to_owned()))
			}
			
			fn visit_string<E>(self, v: String) -> Result<Self::Value, E> where
			E: serde::de::Error, {
				Ok(OptionType::Category(v))
			}
			
			fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where
			A: serde::de::MapAccess<'de>, {
				let mut name = None;
				let mut description = None;
				let mut settings = None;
				
				while let Some(key) = map.next_key::<String>()? {
					match key.as_str() {
						"name" => name = Some(map.next_value()?),
						"description" => description = Some(map.next_value()?),
						"settings" => settings = Some(map.next_value()?),
						_ => return Err(serde::de::Error::unknown_field(&key, &["name", "description", "settings"]))
					}
				}
				
				Ok(OptionType::Option(Option {
					name: name.ok_or_else(|| serde::de::Error::missing_field("name"))?,
					description: description.ok_or_else(|| serde::de::Error::missing_field("description"))?,
					settings: settings.ok_or_else(|| serde::de::Error::missing_field("settings"))?,
				}))
			}
		}
		
		d.deserialize_any(DeVisitor)
	}
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Option {
	pub name: String,
	pub description: String,
	pub settings: OptionSettings,
}

impl std::hash::Hash for Option {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.name.hash(state);
		self.description.hash(state);
		self.settings.to_str().hash(state);
	}
}

// ----------

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum OptionSettings {
	SingleFiles(ValueFiles),
	MultiFiles(ValueFiles),
	Rgb(ValueRgb),
	Rgba(ValueRgba),
	Grayscale(ValueSingle),
	Opacity(ValueSingle),
	Mask(ValueSingle),
	Path(ValuePath),
	// Composite(Composite), // possibly for the future for merging multiple meshes into 1
}

impl EnumTools for OptionSettings {
	type Iterator = std::array::IntoIter<Self, 8>;
	
	fn to_str(&self) -> &'static str {
		match self {
			Self::SingleFiles(_) => "Single Files",
			Self::MultiFiles(_) => "Multi Files",
			Self::Rgb(_) => "RGB",
			Self::Rgba(_) => "RGBA",
			Self::Grayscale(_) => "Grayscale",
			Self::Opacity(_) => "Opacity",
			Self::Mask(_) => "Mask",
			Self::Path(_) => "Path",
		}
	}
	
	fn iter() -> Self::Iterator {
		[
			Self::SingleFiles(ValueFiles::default()),
			Self::MultiFiles(ValueFiles::default()),
			Self::Rgb(ValueRgb::default()),
			Self::Rgba(ValueRgba::default()),
			Self::Grayscale(ValueSingle::default()),
			Self::Opacity(ValueSingle::default()),
			Self::Mask(ValueSingle::default()),
			Self::Path(ValuePath::default()),
		].into_iter()
	}
}

// ----------

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ValueFiles {
	pub default: u32, // TODO: perhabs dupe this struct and have default value be a vec of bools for multi
	pub options: Vec<ValueFilesOption>,
}

impl Default for ValueFiles {
	fn default() -> Self {
		Self {
			default: 0,
			options: vec![],
		}
	}
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct ValueFilesOption {
	pub name: String,
	pub description: String,
	pub inherit: std::option::Option<String>, // unused in multi
	pub files: HashMap<String, String>,
	pub file_swaps: HashMap<String, String>,
	pub manipulations: Vec<Manipulation>,
	pub ui_colors: Vec<UiColor>,
}

impl Default for ValueFilesOption {
	fn default() -> Self {
		Self {
			name: "New sub option".to_owned(),
			description: String::new(),
			inherit: None,
			files: HashMap::new(),
			file_swaps: HashMap::new(),
			manipulations: Vec::new(),
			ui_colors: Vec::new(),
		}
	}
}

impl std::hash::Hash for ValueFilesOption {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.name.hash(state);
		self.description.hash(state);
	}
}

// ----------

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ValueRgb {
	pub default: [f32; 3],
	pub min: [f32; 3],
	pub max: [f32; 3],
}

impl Default for ValueRgb {
	fn default() -> Self {
		Self {
			default: [1.0, 1.0, 1.0],
			min: [0.0, 0.0, 0.0],
			max: [1.0, 1.0, 1.0],
		}
	}
}

// ----------

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ValueRgba {
	pub default: [f32; 4],
	pub min: [f32; 4],
	pub max: [f32; 4],
}

impl Default for ValueRgba {
	fn default() -> Self {
		Self {
			default: [1.0, 1.0, 1.0, 1.0],
			min: [0.0, 0.0, 0.0, 0.0],
			max: [1.0, 1.0, 1.0, 1.0],
		}
	}
}

// ----------

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ValueSingle {
	pub default: f32,
	pub min: f32,
	pub max: f32,
}

impl Default for ValueSingle {
	fn default() -> Self {
		Self {
			default: 0.0,
			min: 0.0,
			max: 1.0,
		}
	}
}

// ----------

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ValuePath {
	pub default: u32,
	/// [option_name, [id, path]]
	pub options: Vec<(String, Vec<(String, ValuePathPath)>)>,
}

impl Default for ValuePath {
	fn default() -> Self {
		Self {
			default: 0,
			options: Vec::new(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ValuePathPath {
	Mod(String),
}

impl EnumTools for ValuePathPath {
	type Iterator = std::array::IntoIter<Self, 1>;
	
	fn to_str(&self) -> &'static str {
		match self {
			Self::Mod(_) => "Mod",
		}
	}
	
	fn iter() -> Self::Iterator {
		[
			Self::Mod(String::new()),
		].into_iter()
	}
}

// ----------

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Manipulation {
	Imc {
		attribute_and_sound: i32,
		material_id: i32,
		decal_id: i32,
		vfx_id: i32,
		material_animation_id: i32,
		attribute_mask: i32,
		sound_id: i32,
		
		primary_id: i32,
		secondary_id: i32,
		variant: i32,
		object_type: String,
		equip_slot: String,
		body_slot: String,
	},
	
	Eqdp {
		entry: u64,
		set_id: i32,
		slot: String,
		race: String,
		gender: String,
	},
	
	Eqp {
		entry: u64,
		set_id: i32,
		slot: String,
	},
	
	Est {
		entry: u64,
		set_id: i32,
		slot: String,
		race: String,
		gender: String,
	},
	
	Gmp {
		enabled: bool,
		animated: bool,
		rotation_a: i32,
		rotation_b: i32,
		rotation_c: i32,
		unknown_a: i32,
		unknown_b: i32,
		unknown_total: i32,
		value: u64,
		
		set_id: i32,
	},
	
	Rsp {
		entry: f32,
		sub_race: String,
		attribute: String,
	},
}

// ----------

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UiColor {
	pub use_theme: bool,
	pub index: u32,
	pub color: super::OptionOrStatic<[f32; 3]>,
}