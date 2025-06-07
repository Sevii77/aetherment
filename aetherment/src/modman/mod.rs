use std::{collections::{HashSet, HashMap}, io::{Read, Write, Seek}};
use serde::{Deserialize, Serialize};
use crate::ui_ext::EnumTools;

#[cfg(any(feature = "plugin", feature = "client"))] pub mod backend;
pub mod meta;
pub mod settings;
// pub mod priority;
// pub mod enabled;
pub mod composite;
pub mod requirement;

// ----------

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum OptionOrStatic<T: OptionValue> {
	OptionSub(String, HashMap<String, T::Value>),
	Option(String),
	OptionMul(String, T::Value),
	OptionGradiant(String, String, T::Value),
	Static(T::Value),
}

impl<T: OptionValue> OptionOrStatic<T> {
	fn resolve(&self, meta: &crate::modman::meta::Meta, settings: &settings::CollectionSettings) -> Option<T::Value> {
		match self {
			OptionOrStatic::OptionSub(opt, options) => settings.get(opt).and_then(|v| {
				match v {
					settings::Value::SingleFiles(v) => {
						let o = meta.options.options_iter().find_map(|v| {
							if v.name == *opt {
								if let crate::modman::meta::OptionSettings::SingleFiles(o) = &v.settings {
									return Some(o)
								}
							}
							
							None
						})?;
						
						let mut o2 = o.options.get(*v as usize)?;
						let mut val;
						loop {
							val = options.iter().find_map(|(n, v)| if n == &o2.name {Some(v.clone())} else {None});
							if val.is_some() {break}
							let Some(inherit) = &o2.inherit else {break};
							let Some(o3) = o.options.iter().find_map(|v| if v.name == *inherit {Some(v)} else {None}) else {break};
							o2 = o3;
						}
						
						val
					},
					
					settings::Value::MultiFiles(v) => {
						let o = meta.options.options_iter().find_map(|v| {
							if v.name == *opt {
								if let crate::modman::meta::OptionSettings::MultiFiles(o) = &v.settings {
									return Some(o)
								}
							}
							
							None
						})?;
						
						for (i, o) in o.options.iter().enumerate() {
							if *v & (1 << i) != 0 {
								for (n, v) in options.iter() {
									if *n == o.name {
										return Some(v.clone())
									}
								}
							}
						}
						
						None
					},
					
					_ => None
				}
			}),
			OptionOrStatic::Option(opt) => settings.get(opt).and_then(|a| T::get_value(a)),
			OptionOrStatic::OptionMul(opt, v) => settings.get(opt).and_then(|a| T::get_value(a).map(|a| T::multiplied(a, v.clone()))),
			OptionOrStatic::OptionGradiant(opt, opt2, v) => Some(T::gradiant(
				T::get_value(settings.get(opt)?)?,
				T::get_value(settings.get(opt2)?)?,
				v.clone()
			)),
			OptionOrStatic::Static(v) => Some(v.clone()),
		}
	}
}

pub trait OptionValue {
	type Value: Clone;
	
	fn get_value(value: &settings::Value) -> Option<Self::Value>;
	fn multiplied(a: Self::Value, b: Self::Value) -> Self::Value;
	fn gradiant(a: Self::Value, b: Self::Value, scale: Self::Value) -> Self::Value;
}

impl OptionValue for i32 {
	type Value = Self;
	
	fn get_value(_value: &settings::Value) -> Option<Self::Value> {None}
	fn multiplied(a: Self::Value, b: Self::Value) -> Self::Value {a * b}
	fn gradiant(a: Self::Value, b: Self::Value, scale: Self::Value) -> Self::Value {a * (1 - scale) + b * scale}
}

impl OptionValue for f32 {
	type Value = Self;
	
	fn get_value(_value: &settings::Value) -> Option<Self::Value> {None}
	fn multiplied(a: Self::Value, b: Self::Value) -> Self::Value {a * b}
	fn gradiant(a: Self::Value, b: Self::Value, scale: Self::Value) -> Self::Value {a * (1.0 - scale) + b * scale}
}

impl OptionValue for [f32; 2] {
	type Value = Self;
	
	fn get_value(_value: &settings::Value) -> Option<Self::Value> {None}
	fn multiplied(a: Self::Value, b: Self::Value) -> Self::Value {[a[0] * b[0], a[1] * b[1]]}
	fn gradiant(a: Self::Value, b: Self::Value, scale: Self::Value) -> Self::Value {
		[
			a[0] * (1.0 - scale[0]) + b[0] * scale[0],
			a[1] * (1.0 - scale[1]) + b[1] * scale[1],
		]
	}
}

impl OptionValue for [f32; 3] {
	type Value = Self;
	
	fn get_value(value: &settings::Value) -> Option<Self::Value> {
		match value {
			settings::Value::Rgba(v) => Some([v[0], v[1], v[2]]),
			settings::Value::Rgb(v) => Some(*v),
			settings::Value::Grayscale(v) => Some([*v, *v, *v]),
			_ => None,
		}
	}
	
	fn multiplied(a: Self::Value, b: Self::Value) -> Self::Value {
		[
			(a[0] * b[0]).clamp(0.0, 1.0),
			(a[1] * b[1]).clamp(0.0, 1.0),
			(a[2] * b[2]).clamp(0.0, 1.0),
		]
	}
	
	fn gradiant(a: Self::Value, b: Self::Value, scale: Self::Value) -> Self::Value {
		[
			a[0] * (1.0 - scale[0]) + b[0] * scale[0],
			a[1] * (1.0 - scale[1]) + b[1] * scale[1],
			a[2] * (1.0 - scale[2]) + b[2] * scale[2],
		]
	}
}

impl OptionValue for [f32; 4] {
	type Value = Self;
	
	fn get_value(value: &settings::Value) -> Option<Self::Value> {
		match value {
			settings::Value::Rgba(v) => Some(*v),
			settings::Value::Rgb(v) => Some([v[0], v[1], v[2], 1.0]),
			settings::Value::Grayscale(v) => Some([*v, *v, *v, 1.0]),
			settings::Value::Opacity(v) => Some([1.0, 1.0, 1.0, *v]),
			_ => None,
		}
	}
	
	fn multiplied(a: Self::Value, b: Self::Value) -> Self::Value {
		[
			(a[0] * b[0]).clamp(0.0, 1.0),
			(a[1] * b[1]).clamp(0.0, 1.0),
			(a[2] * b[2]).clamp(0.0, 1.0),
			(a[3] * b[3]).clamp(0.0, 1.0),
		]
	}
	
	fn gradiant(a: Self::Value, b: Self::Value, scale: Self::Value) -> Self::Value {
		[
			a[0] * (1.0 - scale[0]) + b[0] * scale[0],
			a[1] * (1.0 - scale[1]) + b[1] * scale[1],
			a[2] * (1.0 - scale[2]) + b[2] * scale[2],
			a[3] * (1.0 - scale[3]) + b[3] * scale[3],
		]
	}
}

// ----------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Path {
	Mod(String),
	Game(String),
	Option(String, String),
}

impl EnumTools for Path {
	type Iterator = std::array::IntoIter<Self, 3>;
	
	fn to_str(&self) -> &'static str {
		match self {
			Self::Mod(_) => "Mod",
			Self::Game(_) => "Game",
			Self::Option(_, _) => "Option",
		}
	}
	
	fn iter() -> Self::Iterator {
		[
			Self::Mod(String::new()),
			Self::Game(String::new()),
			Self::Option(String::new(), String::new()),
		].into_iter()
	}
}

// ----------

pub fn get_mod_files(meta: &meta::Meta, files_path: &std::path::Path) -> HashMap<String, Vec<String>> {
	let mut files = HashMap::new();
	let mut insert = |path: Option<&str>, real_path: &str| {
		let entry = files.entry(real_path.to_owned()).or_insert_with(|| Vec::new());
		if let Some(path) = path {
			entry.push(path.to_owned());
		}
	};
	
	let mut add_file = |path: &str, real_path: &str| {
		insert(Some(path), real_path);
		
		if path.ends_with(".comp") {
			let ext = path.trim_end_matches(".comp").split(".").last().unwrap();
			
			let Ok(data) = crate::resource_loader::read_utf8(&files_path.join(real_path)) else {return};
			let Some(comp) = composite::open_composite(ext, &data) else {return};
			for file in comp.get_files() {
				insert(None, file);
			}
		}
	};
	
	for (path, real_path) in &meta.files {
		add_file(path, real_path);
	}
	
	for option in meta.options.options_iter() {
		if let meta::OptionSettings::SingleFiles(v) | meta::OptionSettings::MultiFiles(v) = &option.settings {
			for sub in &v.options {
				for (path, real_path) in &sub.files {
					add_file(path, real_path);
				}
			}
		}
	}
	
	for option in meta.options.options_iter() {
		if let meta::OptionSettings::Path(v) = &option.settings {
			for (_, paths) in &v.options {
				for (_id, path) in paths {
					let meta::ValuePathPath::Mod(path) = path;
					insert(None, path);
				}
			}
		}
	}
	
	files
}

pub fn game_files_hashes(files: HashSet<&str>) -> HashMap<String, String> {
	let mut hashes = HashMap::new();
	let Some(noum) = crate::noumenon() else {return hashes};
	
	for file in files {
		let file = file.trim_end_matches(".comp");
		if let Ok(f) = noum.file::<Vec<u8>>(file) {
			log!("hashing game file of {file}");
			hashes.insert(file.to_string(), crate::hash_str(blake3::hash(&f)));
		}
	}
	
	hashes
}

// TODO: actually use this IN THE MODPACK CREATION STRUCT, HELLO PAST ME??
#[derive(Debug, Clone, Copy)]
pub struct ModCreationSettings {
	/// Used to be able to check changes the game has made to files this mod overrides, useful for ui
	pub current_game_files_hash: bool,
}

pub struct ModPack<W: Write + Seek> {
	writer: zip::ZipWriter<W>,
	options: zip::write::FileOptions,
	settings: ModCreationSettings,
	
	done: HashSet<blake3::Hash>,
	remap: HashMap<String, String>,
}

impl<W: Write + Seek> ModPack<W> {
	pub fn new(writer: W, settings: ModCreationSettings) -> Self {
		let options = zip::write::FileOptions::default()
			.compression_method(zip::CompressionMethod::Deflated)
			.compression_level(Some(9))
			.large_file(true);
		
		let mut writer = zip::ZipWriter::new(writer);
		_ = writer.add_directory("files", options);
		
		Self {
			writer,
			options,
			settings,
			
			done: HashSet::new(),
			remap: HashMap::new(),
		}
	}
	
	pub fn add_file(&mut self, path: &str, data: &[u8]) -> Result<(), crate::resource_loader::BacktraceError> {
		let hash = blake3::hash(data);
		let hash_str = crate::hash_str(hash);
		let filename = path.split("/").last().unwrap();
		let ext = if let Some(p) = filename.find(".") {&filename[p + 1..filename.len()]} else {""};
		let hash_ext = format!("{hash_str}.{ext}");
		self.remap.insert(path.to_string(), hash_ext.clone());
		
		if self.done.contains(&hash) {return Ok(())}
		self.done.insert(hash);
		
		let name = format!("files/{hash_ext}");
		self.writer.start_file(name, self.options)?;
		self.writer.write_all(data)?;
		
		Ok(())
	}
	
	pub fn add_meta_file(&mut self, data: &[u8]) -> Result<(), crate::resource_loader::BacktraceError> {
		self.writer.start_file("meta.json", self.options)?;
		self.writer.write_all(data)?;
		
		Ok(())
	}
	
	pub fn add_meta(&mut self, meta: &meta::Meta) -> Result<(), crate::resource_loader::BacktraceError> {
		self.add_meta_file(&crate::json_pretty(&meta)?.as_bytes())?;
		
		Ok(())
	}
	
	// TODO: this really should be done inside, too lazy atm
	pub fn add_hashes(&mut self, hashes: &HashMap<String, String>) -> Result<(), crate::resource_loader::BacktraceError> {
		self.writer.start_file("hashes", self.options)?;
		self.writer.write_all(&serde_json::to_vec(&hashes)?)?;
		
		Ok(())
	}
	
	pub fn finalize(mut self) -> Result<W, crate::resource_loader::BacktraceError> {
		self.writer.start_file("remap", self.options)?;
		self.writer.write_all(&serde_json::to_vec(&self.remap)?)?;
		
		Ok(self.writer.finish()?)
	}
}

// TODO: use proper error
pub fn create_mod(mod_path: &std::path::Path, settings: ModCreationSettings) -> Result<std::path::PathBuf, crate::resource_loader::BacktraceError> {
	let meta_buf = {
		let mut buf = Vec::new();
		std::fs::File::open(mod_path.join("meta.json"))?.read_to_end(&mut buf)?;
		buf
	};
	let meta: meta::Meta = serde_json::from_slice(&meta_buf)?;
	let packs_path = mod_path.join("packs");
	_ = std::fs::create_dir(&packs_path);
	
	let files_path = mod_path.join("files");
	let files = get_mod_files(&meta, &files_path);
	
	log!("all files: {files:?}");
	
	let pack_path = packs_path.join(format!("{}.aeth", meta.version));
	if pack_path.exists() {return Err("Path with this version already exists".into())}
	let mut writer = ModPack::new(std::io::BufWriter::new(std::fs::File::create(&pack_path)?), settings);
	
	if settings.current_game_files_hash {
		let hashes = game_files_hashes(files.values().flat_map(|v| v.iter().map(|v| v.as_str())).collect());
		writer.add_hashes(&hashes)?;
	}
	
	writer.add_meta(&meta)?;
	
	let mut buf = Vec::new();
	for (real_path, _paths) in &files {
		log!("packing file {real_path}");
		let mut f = std::fs::File::open(files_path.join(&real_path))?;
		f.read_to_end(&mut buf)?;
		writer.add_file(real_path, &buf)?;
		buf.clear();
	}
	
	writer.finalize()?;
	
	Ok(pack_path)
}

pub fn check_diff<R: Read + Seek>(mod_data: R) -> Result<Vec<String>, crate::resource_loader::BacktraceError> {
	let noum = crate::noumenon().ok_or("Noumenon not loaded")?;
	let mut pack = zip::ZipArchive::new(mod_data)?;
	
	let mut hashes_buf = Vec::new();
	pack.by_name("hashes")?.read_to_end(&mut hashes_buf)?;
	let hashes = serde_json::from_slice::<HashMap<String, String>>(&hashes_buf)?;
	
	let mut changes = Vec::new();
	for (path, hash_org) in hashes {
		if let Ok(f) = noum.file::<Vec<u8>>(&path) {
			let hash = crate::hash_str(blake3::hash(&f));
			if hash != hash_org {
				changes.push(path);
			}
		} else {
			log!("{path} does not exist in the game files");
		}
	}
	
	Ok(changes)
}