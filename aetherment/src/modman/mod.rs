use std::{collections::{HashSet, HashMap}, io::{Read, Write, Seek}};
use serde::{Deserialize, Serialize};
use crate::render_helper::EnumTools;

use self::composite::Composite;

pub mod backend;
pub mod meta;
pub mod settings;
// pub mod priority;
// pub mod enabled;
pub mod composite;

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
		// files.insert(real_path.to_owned());
		insert(Some(path), real_path);
		
		if path.ends_with(".comp") {
			match path.trim_end_matches(".comp").split(".").last().unwrap() {
				"tex" | "atex" => {
					let Ok(mut f) = std::fs::File::open(files_path.join(real_path)) else {return};
					let mut buf = Vec::new();
					f.read_to_end(&mut buf).unwrap();
					let comp: composite::tex::Tex = match serde_json::from_slice(&buf) {
						Ok(v) => v,
						Err(e) => {
							log!(err, "Failed to parse tex comp file: {e}\ndata: {}", String::from_utf8_lossy(&buf));
							return;
						}
					};
					
					for file in comp.get_files() {
						// files.insert(file.to_owned());
						insert(None, file);
					}
				}
				
				_ => {return}
			}
		}
	};
	
	for (path, real_path) in &meta.files {
		add_file(path, real_path);
	}
	
	for option in &meta.options {
		if let meta::OptionSettings::SingleFiles(v) | meta::OptionSettings::MultiFiles(v) = &option.settings {
			for sub in &v.options {
				for (path, real_path) in &sub.files {
					add_file(path, real_path);
				}
			}
		}
	}
	
	for option in &meta.options {
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

// pub fn cleanup(mod_path: &std::path::Path) -> Result<(), crate::resource_loader::BacktraceError> {
// 	let meta: meta::Meta = serde_json::from_reader(std::io::BufReader::new(std::fs::File::open(mod_path.join("meta.json"))?))?;
// 	let files = get_mod_files(&meta, &mod_path.join("files"));
// 	
// 	// TODO: cleanup here
// 	
// 	Ok(())
// }

#[derive(Debug, Clone, Copy)]
pub struct ModCreationSettings {
	/// Used to be able to check changes the game has made to files this mod overrides, useful for ui
	pub current_game_files_hash: bool,
}

pub struct ModPack<W: Write + Seek> {
	writer: zip::ZipWriter<W>,
	options: zip::write::FileOptions,
	#[allow(dead_code)] settings: ModCreationSettings,
	
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