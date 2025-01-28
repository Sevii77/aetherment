use std::{borrow::Cow, collections::{HashMap, HashSet}, fs::File, io::{Read, Write}, sync::{Arc, Mutex}};
use serde::{Deserialize, Serialize};
use crate::{log, modman::{meta, OptionOrStatic}, resource_loader::read_json};

#[repr(packed)]
pub struct GetModSettings {
	pub exists: bool,
	pub enabled: bool,
	pub inherit: bool,
	pub priority: i32,
	pub options: HashMap<String, Vec<String>>,
}

static mut FUNCS: Option<PenumbraFunctions> = None;

// #[allow(unused)] pub(crate) fn config_dir() -> Option<std::path::PathBuf> {unsafe {FUNCS.as_ref().map(|v| Some(v.config_dir.clone())).unwrap_or(None)}}
#[allow(unused)] fn redraw() {unsafe {(FUNCS.as_ref().unwrap().redraw)()}}
#[allow(unused)] fn redraw_self() {unsafe {(FUNCS.as_ref().unwrap().redraw_self)()}}
fn is_enabled() -> bool {unsafe {(FUNCS.as_ref().unwrap().is_enabled)()}}
fn root_path() -> std::path::PathBuf {unsafe {(FUNCS.as_ref().unwrap().root_path)()}}
fn mod_list() -> Vec<String> {unsafe {(FUNCS.as_ref().unwrap().mod_list)()}}
fn add_mod_entry(mod_id: &str) -> u8 {unsafe {(FUNCS.as_ref().unwrap().add_mod_entry)(mod_id)}}
fn reload_mod(mod_id: &str) -> u8 {unsafe {(FUNCS.as_ref().unwrap().reload_mod)(mod_id)}}
fn set_mod_enabled(collection_id: &str, mod_id: &str, enabled: bool) -> u8 {unsafe {(FUNCS.as_ref().unwrap().set_mod_enabled)(collection_id, mod_id, enabled)}}
fn set_mod_priority(collection_id: &str, mod_id: &str, priority: i32) -> u8 {unsafe {(FUNCS.as_ref().unwrap().set_mod_priority)(collection_id, mod_id, priority)}}
fn set_mod_inherit(collection_id: &str, mod_id: &str, inherit: bool) -> u8 {unsafe {(FUNCS.as_ref().unwrap().set_mod_inherit)(collection_id, mod_id, inherit)}}
fn set_mod_settings(collection_id: &str, mod_id: &str, option: &str, sub_options: Vec<&str>) -> u8 {unsafe {(FUNCS.as_ref().unwrap().set_mod_settings)(collection_id, mod_id, option, sub_options)}}
pub(crate) fn get_mod_settings(collection_id: &str, mod_id: &str, inherit: bool) -> GetModSettings {unsafe {(FUNCS.as_ref().unwrap().get_mod_settings)(collection_id, mod_id, inherit)}}
pub(crate) fn get_collection(collection_type: super::CollectionType) -> super::Collection {unsafe {(FUNCS.as_ref().unwrap().get_collection)(collection_type)}}
fn current_collection() -> super::Collection {get_collection(super::CollectionType::Current)}
fn get_collections() -> Vec<super::Collection> {unsafe {(FUNCS.as_ref().unwrap().get_collections)()}}

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern fn backend_penumbraipc_modchanged(typ: u8, collection_id: &str, mod_id: &str) {
	// log!("{typ} {mod_id} {collection_id}");
	// super fucking cheesy, idc
	let is_aeth = root_path().join(mod_id).join("aetherment").exists();
	if typ == 3 { // settings
		if !is_aeth {
			crate::backend().apply_mod_settings(mod_id, collection_id, super::SettingsType::Keep);
		}
	} else if !is_aeth || typ != 7 { // edited
		crate::backend().apply_mod_settings(mod_id, collection_id, super::SettingsType::Keep);
	}
}

pub struct PenumbraFunctions {
	// pub config_dir: std::path::PathBuf,
	pub redraw: Box<dyn Fn()>,
	pub redraw_self: Box<dyn Fn()>,
	pub is_enabled: Box<dyn Fn() -> bool>,
	pub root_path: Box<dyn Fn() -> std::path::PathBuf>,
	pub mod_list: Box<dyn Fn() -> Vec<String>>,
	pub add_mod_entry: Box<dyn Fn(&str) -> u8>,
	pub reload_mod: Box<dyn Fn(&str) -> u8>,
	pub set_mod_enabled: Box<dyn Fn(&str, &str, bool) -> u8>,
	pub set_mod_priority: Box<dyn Fn(&str, &str, i32) -> u8>,
	pub set_mod_inherit: Box<dyn Fn(&str, &str, bool) -> u8>,
	pub set_mod_settings: Box<dyn Fn(&str, &str, &str, Vec<&str>) -> u8>,
	pub get_mod_settings: Box<dyn Fn(&str, &str, bool) -> GetModSettings>,
	pub get_collection: Box<dyn Fn(super::CollectionType) -> super::Collection>,
	pub get_collections: Box<dyn Fn() -> Vec<super::Collection>>,
}

pub(crate) fn initialize_functions(funcs: PenumbraFunctions) {
	unsafe{FUNCS = Some(funcs)}
}

/// <(mod_id, collection_id), (settings, file_whitelist)>
type ApplyQueue = HashMap<(String, String), (super::SettingsType, Option<HashSet<String>>)>;

/// This does not care for options, its simply all composite files a mod has
/// <game_path, [mod_id]>
type CompositeLinks = HashMap<String, HashSet<String>>;

/// <collection_id, <game_path, [(real_path, mod_id, priority)]>>
type ModFileCache = HashMap<String, HashMap<String, Vec<(String, String, i32)>>>;

struct CompositeInfo {
	pub composite_links: CompositeLinks,
	pub composite_caches: HashMap<String, crate::modman::composite::CompositeCache>,
}

struct ModInfo {
	pub is_aeth: bool,
	pub meta: crate::modman::meta::Meta,
	// pub composite_cache: crate::modman::composite::CompositeCache,
}

pub struct Penumbra {
	apply_queue: Arc<Mutex<ApplyQueue>>,
	// mods: Arc<HashMap<String, ModInfo>>,
	mods: Arc<Mutex<Vec<String>>>,
	mod_infos: HashMap<String, ModInfo>,
}

impl Penumbra {
	pub fn new() -> Self {
		Self {
			apply_queue: Arc::new(Mutex::new(HashMap::new())),
			mods: Arc::new(Mutex::new(Vec::new())),
			mod_infos: HashMap::new(),
		}
	}
}

impl super::Backend for Penumbra {
	fn name(&self) -> &'static str {
		"Penumbra (IPC)"
	}
	
	fn description(&self) -> &'static str {
		"Penumbra mod loader using IPC"
	}
	
	fn get_status(&self) -> super::Status {
		if is_enabled() {
			super::Status::Ok
		} else {
			super::Status::Error("Penumbra is not installed or mods are disabled".to_string())
		}
	}
	
	fn get_mods(&self) -> Vec<String> {
		// self.mods.iter().map(|(m, _)| m.to_owned()).collect()
		self.mods.lock().unwrap().clone()
	}
	
	fn get_active_collection(&self) -> String {
		current_collection().id
	}
	
	fn get_collections(&self) -> Vec<super::Collection> {
		get_collections()
	}
	
	// only mods that originally started with ui/ with be considered, game paths are not checked
	// we only rewrite "remap" as this way we dont have to rewrite any other files
	// TODO: possibly change this?
	fn install_mods(&mut self, progress: super::InstallProgress, files: Vec<(String, std::fs::File)>) {
		progress.set_busy(true);
		progress.mods.set(0.0);
		progress.mods.set_msg("Installing Mods");
		progress.current_mod.set_msg("");
		
		let apply_queue = self.apply_queue.clone();
		let mods = self.mods.clone();
		std::thread::spawn(move || {
			if let Err(err) = (|| -> Result<(), crate::resource_loader::BacktraceError> {
				let total_mods = files.len();
				// for (mod_index, file) in files.into_iter().enumerate() {
				for (mod_index, (mod_id, file)) in files.into_iter().enumerate() {
					progress.mods.set(mod_index as f32 / total_mods as f32);
					// progress.mods.set_msg(&file.to_string_lossy());
					// let mut pack = zip::ZipArchive::new(std::io::BufReader::new(std::fs::File::open(file)?))?;
					progress.mods.set_msg(&mod_id);
					let mut pack = zip::ZipArchive::new(file)?;
					
					let mut meta_buf = Vec::new();
					pack.by_name("meta.json")?.read_to_end(&mut meta_buf)?;
					let meta = serde_json::from_slice::<meta::Meta>(&meta_buf)?;
					
					// get all files that are used directly, so we can keep the rest zipped
					let mut direct_files = HashSet::new();
					for (path, _) in meta.files.iter() {
						if !path.ends_with(".comp") {
							direct_files.insert(path.as_str());
						}
					}
					for opt in meta.options.options_iter() {
						if let crate::modman::meta::OptionSettings::SingleFiles(v) |
							crate::modman::meta::OptionSettings::MultiFiles(v) = &opt.settings {
							for sub in v.options.iter() {
								for (path, _) in sub.files.iter() {
									if !path.ends_with(".comp") {
										direct_files.insert(path.as_str());
									}
								}
							}
						}
					}
					
					// let mod_id = meta.name.clone();
					let mod_dir = root_path().join(&mod_id);
					_ = std::fs::create_dir(&mod_dir);
					let aeth_dir = mod_dir.join("aetherment");
					_ = std::fs::create_dir(&aeth_dir);
					let files_dir = mod_dir.join("files");
					_ = std::fs::create_dir(&files_dir);
					File::create(aeth_dir.join("meta.json"))?.write_all(&meta_buf)?;
					// buf.clear();
					
					let mut compdata_used = false;
					let mut compdata = zip::ZipWriter::new(std::io::BufWriter::new(File::create(files_dir.join("_compdata"))?));
					compdata.add_directory("files", zip::write::FileOptions::default()
						.compression_method(zip::CompressionMethod::Deflated)
						.compression_level(Some(9))
						.large_file(true))?;
					
					File::create(mod_dir.join("meta.json"))?.write_all(crate::json_pretty(&PMeta {
						FileVersion: 3,
						Name: meta.name.clone(),
						Author: meta.author.clone(),
						Description: meta.description.clone(),
						Version: meta.version.clone(),
						Website: meta.website.clone(),
						ModTags: meta.tags.iter().map(|v| v.to_owned()).collect(),
					})?.as_bytes())?;
					
					File::create(mod_dir.join("default_mod.json"))?.write_all(crate::json_pretty(&PDefaultMod {
						// Name: String::new(),
						// Description: String::new(),
						Files: HashMap::new(),
						FileSwaps: HashMap::new(),
						Manipulations: Vec::new(),
					})?.as_bytes())?;
					
					// std::io::copy(&mut file.by_name("remap")?, &mut File::create(aeth_dir.join("remap"))?)?;
					let mut remap_buf = Vec::new();
					pack.by_name("remap")?.read_to_end(&mut remap_buf)?;
					let mut remap = serde_json::from_slice::<HashMap<String, String>>(&remap_buf)?;
					// std::fs::write(aeth_dir.join("remap"), remap_buf)?;
					let mut remap_rev = HashMap::new();
					for (org, hash) in &remap {
						remap_rev.entry(hash.to_owned()).or_insert_with(|| Vec::new()).push(org.to_owned());
					}
					
					if let Ok(mut hashes) = pack.by_name("hashes") {
						std::io::copy(&mut hashes, &mut File::create(aeth_dir.join("hashes"))?)?;
					}
					
					let pack_len = pack.len();
					for i in 0..pack_len {
						progress.current_mod.set(i as f32 / pack_len as f32);
						let mut f = pack.by_index(i)?;
						if f.is_dir() {continue};
						let Some(name) = f.enclosed_name() else {continue};
						let Ok(name) = name.strip_prefix("files/") else {continue};
						if name.components().count() > 1 {continue};
						let name = name.to_owned();
						let hash = name.to_string_lossy().to_string();
						let ext = if let Some(p) = hash.find(".") {&hash[p + 1..]} else {""};
						
						let org_paths = remap_rev.get(&hash).ok_or("Remap does not contain hash")?;
						let is_direct = org_paths.iter().any(|v| direct_files.contains(v.as_str()));
						if is_direct {
							let mut buf = Vec::new();
							f.read_to_end(&mut buf)?;
							
							let mut write_org = false;
							for org_path in org_paths {
								if org_path.starts_with("ui/") {
									let hashed_path = crate::hash_str(blake3::hash(org_path.as_bytes()));
									let new_name = format!("{hashed_path}.{ext}");
									// std::io::copy(&mut f, &mut File::create(files_dir.join(&new_name))?)?;
									std::fs::write(files_dir.join(&new_name), &buf)?;
									remap.insert(org_path.to_owned(), new_name);
								} else {
									write_org = true;
								}
							}
							
							if write_org {
								// std::io::copy(&mut f, &mut File::create(files_dir.join(name))?)?;
								std::fs::write(files_dir.join(&name), &buf)?;
							}
						} else {
							compdata_used = true;
							compdata.raw_copy_file(f)?;
						}
					}
					
					std::fs::write(aeth_dir.join("remap"), crate::json_pretty(&remap)?.as_bytes())?;
					
					if !compdata_used {
						_ = std::fs::remove_file(files_dir.join("_compdata"));
					}
					
					add_mod_entry(&mod_id);
					
					for c in get_collections() {
						let s = get_mod_settings(&c.id, &mod_id, false);
						if s.enabled && !s.inherit {
							let settings = crate::modman::settings::Settings::open(&meta, &mod_id).get_collection(&meta, &c.id).to_owned();
							apply_queue.lock().unwrap().insert((mod_id.to_owned(), c.id), (super::SettingsType::Some(settings), None));
						}
					}
					// let settings = crate::modman::settings::Settings::from_meta(&meta);
					// apply_queue.lock().unwrap().insert((mod_id.to_owned(), current_collection().id), (super::SettingsType::Some(settings), None));
					
					mods.lock().unwrap().push(mod_id.to_owned());
				}
				
				progress.mods.set(1.0);
				
				Ok(())
			})() {
				log!(err, "{err}");
				progress.current_mod.set_msg(&err.to_string());
			}
			
			let mut squeue = apply_queue.lock().unwrap();
			let queue = squeue.clone();
			let comp_info = get_composite_info(mods.lock().unwrap().iter().map(|v| v.as_ref()).collect());
			finalize_apply(queue, comp_info, progress.apply.clone());
			squeue.clear();
			
			progress.set_busy(false);
		});
	}
	
	fn apply_mod_settings(&mut self, mod_id: &str, collection_id: &str, settings: super::SettingsType) {
		self.apply_queue.lock().unwrap().insert((mod_id.to_owned(), collection_id.to_owned()), (settings.clone(), None));
	}
	
	fn finalize_apply(&mut self, progress: super::ApplyProgress) {
		let mut squeue = self.apply_queue.lock().unwrap();
		let queue = squeue.clone();
		let comp_info = get_composite_info(self.mods.lock().unwrap().iter().map(|v| v.as_ref()).collect());
		std::thread::spawn(move || {
			finalize_apply(queue, comp_info, progress);
		});
		squeue.clear();
	}
	
	fn apply_queue_size(&self) -> usize {
		self.apply_queue.lock().unwrap().len()
	}
	
	fn apply_services(&self) {
		apply_ui_colors();
	}
	
	fn load_mods(&mut self) {
		let root = root_path();
		
		self.mod_infos = mod_list().into_iter().filter_map(|id| {
			let mod_dir = root.join(&id);
			let is_aeth = mod_dir.join("aetherment").exists();
			
			let meta = if is_aeth {
				read_json::<meta::Meta>(&mod_dir.join("aetherment").join("meta.json")).ok()?
			} else {
				return None;
			};
			
			Some((id, ModInfo{
				is_aeth,
				meta,
			}))
		}).collect();
		
		let mut mods = self.mods.lock().unwrap();
		*mods = self.mod_infos.iter().map(|(id, _)| id.to_owned()).collect();
		
		log!(log, "Loaded mods: {}", mods.len());
	}
	
	fn get_mod_meta(&self, mod_id: &str) -> Option<&meta::Meta> {
		self.mod_infos.get(mod_id).map(|m| &m.meta)
	}
	
	fn get_mod_enabled(&self, mod_id: &str, collection_id: &str) -> bool {
		get_mod_settings(collection_id, mod_id, true).enabled
	}
	
	fn set_mod_enabled(&mut self, mod_id: &str, collection_id: &str, enabled: bool) {
		set_mod_enabled(collection_id, mod_id, enabled);
	}
	
	fn get_mod_priority(&self, mod_id: &str, collection_id: &str) -> i32 {
		get_mod_settings(collection_id, mod_id, true).priority
	}
	
	fn set_mod_priority(&mut self, mod_id: &str, collection_id: &str, priority: i32) {
		set_mod_priority(collection_id, mod_id, priority);
	}
}

fn apply_ui_colors() {
	let collection = get_collection(super::CollectionType::Interface);
	if !collection.is_valid() {return}
	
	let mut final_ui_colors = HashMap::<(bool, u32), (i32, [u8; 3])>::new();
	
	let root = root_path();
	for id in mod_list().into_iter() {
		let mod_dir = root.join(&id);
		let aeth_dir = mod_dir.join("aetherment");
		if !aeth_dir.exists() {continue}
		let settings = get_mod_settings(&collection.id, &id, true);
		if !settings.enabled {continue}
		let priority = settings.priority;
		let Ok(ui_colors) = read_json::<Vec<(bool, u32, OptionOrStatic<[f32; 3]>)>>(&aeth_dir.join("uicolorcache")) else {continue};
		let Ok(meta) = read_json::<meta::Meta>(&aeth_dir.join("meta.json")) else {continue};
		let mut settings = crate::modman::settings::Settings::open(&meta, &id);
		let collection_settings = settings.get_collection(&meta, &collection.id);
		for (use_theme, index, color) in ui_colors {
			let Some(color) = color.resolve(&meta, &collection_settings) else {continue};
			let color = [(color[0] * 255.0).clamp(0.0, 255.0) as u8, (color[1] * 255.0).clamp(0.0, 255.0) as u8, (color[2] * 255.0).clamp(0.0, 255.0) as u8];
			
			if let Some(c) = final_ui_colors.get_mut(&(use_theme, index)) {
				if priority > c.0 {
					c.0 = priority;
					c.1 = color;
				}
			} else {
				final_ui_colors.insert((use_theme, index), (priority, color));
			}
		}
	}
	
	crate::service::uicolor::clear_colors();
	for ((use_theme, index), (_, color)) in final_ui_colors {
		crate::service::uicolor::set_color(use_theme, index, color);
	}
}

// TODO: support only updating files of options that just changed (composite_info has info about that)
fn finalize_apply(apply_queue: ApplyQueue, composite_info: CompositeInfo, progress: super::ApplyProgress) {
	// 1. sort the queue based on priority, lowest > highest
	// 2. apply first mod
	// 3. check if any mods with a higher priority have the same file of the mod we just applied AND are composite files
	// 4. if so: add it to the queue and resort it
	// repeat until queue is empty
	
	progress.set_busy(true);
	
	type Entry = (String, i32, super::SettingsType, Option<HashSet<String>>);
	fn sort_queue(queue: &mut Vec<Entry>) {
		queue.sort_unstable_by(|(an, ap, _, _), (bn, bp, _, _)| bp.cmp(ap).then_with(|| bn.cmp(an)));
	}
	
	fn push_queue(queue: &mut Vec<Entry>, entry: Entry) -> bool {
		if queue.iter().any(|(mod_id, _, _, _)| *mod_id == entry.0) {return false}
		queue.push(entry);
		true
	}
	
	let composite_links = &composite_info.composite_links;
	
	let mut mods_done = 0;
	let mut total_mods = 0;
	let mut queue = HashMap::new();
	for ((mod_id, collection_id), (settings, whitelist)) in apply_queue {
		let priority = get_mod_settings(&collection_id, &mod_id, true).priority;
		// queue.entry(collection_id).or_insert_with(|| Vec::new()).push((mod_id, priority, settings, whitelist));
		let sub = queue.entry(collection_id).or_insert_with(|| Vec::new());
		if push_queue(sub, (mod_id, priority, settings, whitelist)) {
			total_mods += 1;
		}
	}
	
	let mut to_cleanup = HashSet::new();
	for (collection_id, queue) in &mut queue {
		sort_queue(queue);
		
		while queue.len() > 0 {
			let (mod_id, priority, settings, file_whitelist) = queue.pop().unwrap();
			log!("applying mod {mod_id} in collection {collection_id}");
			progress.mods.set(mods_done as f32 / total_mods as f32);
			progress.mods.set_msg(&mod_id);
			
			let changed_files = match apply_mod(&mod_id, collection_id, settings, file_whitelist, progress.current_mod.clone()) {
				Ok(v) => v,
				Err(e) => {
					log!(err, "error applying mod {mod_id} in collection {collection_id} ({e:?})");
					continue;
				}
			};
			
			log!("changed files:");
			for v in &changed_files {
				log!("\t{v}");
			}
			
			let mut add_queue = HashSet::new();
			for f in &changed_files {
				if let Some(linked_mods) = composite_links.get(f) {
					for linked_mod_id in linked_mods {
						let linked_settings = get_mod_settings(&collection_id, linked_mod_id, true);
						if linked_settings.exists && linked_settings.enabled && linked_settings.priority > priority {
							add_queue.insert((linked_mod_id, linked_settings.priority));
						}
					}
				}
			}
			
			let mut do_sort_queue = false;
			for (to_add_id, to_add_priority) in add_queue {
				if push_queue(queue, (to_add_id.to_owned(), to_add_priority, super::SettingsType::Keep, Some(changed_files.clone()))) {
					do_sort_queue = true;
					total_mods += 1;
				}
			}
			
			if do_sort_queue {
				sort_queue(queue);
			}
			
			mods_done += 1;
			
			to_cleanup.insert(mod_id);
		}
	}
	
	for mod_id in to_cleanup {
		cleanup_mod(&mod_id);
	}
	
	apply_ui_colors();
	
	progress.set_busy(false);
}

// TODO: check last settings so we can avoid re-compositing files which settings didnt change
fn apply_mod(mod_id: &str, collection_id: &str, settings: super::SettingsType, file_whitelist: Option<HashSet<String>>, progress: super::Progress) -> Result<HashSet<String>, crate::resource_loader::BacktraceError> {
	let mut changed_files = HashSet::new();
	
	let root = root_path();
	let mod_dir = root.join(mod_id);
	let aeth_dir = mod_dir.join("aetherment");
	if !aeth_dir.exists() {
		return Ok(get_mod_files(collection_id, mod_id).map_or(changed_files, |v| v.into_iter().map(|(v, _)| v).collect()));
	}
	let files_dir = mod_dir.join("files");
	let files_comp_dir = mod_dir.join("files_comp");
	_ = std::fs::create_dir(&files_comp_dir);
	
	let meta = serde_json::from_reader::<_, meta::Meta>(std::io::BufReader::new(File::open(aeth_dir.join("meta.json"))?))?;
	let remap = serde_json::from_reader::<_, HashMap<String, String>>(std::io::BufReader::new(File::open(aeth_dir.join("remap"))?))?;
	
	let settings = match settings {
		super::SettingsType::Clear => {
			set_mod_inherit(collection_id, mod_id.as_ref(), false);
			return Ok(changed_files);
		}
		
		super::SettingsType::Keep => crate::modman::settings::Settings::open(&meta, mod_id).get_collection(&meta, collection_id).to_owned(),
		super::SettingsType::Some(v) => v,
	};
	
	// ----------
	
	let penum_settings = get_mod_settings(&collection_id, mod_id, true);
	if !penum_settings.enabled {
		let Ok(group) = read_json::<PGroup>(&mod_dir.join("group_001__collection.json")) else {return Ok(changed_files)};
		let Some(option) = group.Options.into_iter().find(|v| v.Name == collection_id) else {return Ok(changed_files)};
		return Ok(option.Files.into_iter().map(|(v, _)| v).collect());
	}
	
	let priority = penum_settings.priority;
	
	// TODO: store this and update it properly after every mod apply, this is increddibly inefficient
	let mod_file_cache = get_mod_cache();
	let get_file = |path: &str, collection: &str, priority: i32| -> Option<Vec<u8>> {
		let mut real_path = None;
		let mut real_path_prio = i32::MIN;
		if let Some(collection) = mod_file_cache.get(collection) {
			if let Some(files) = collection.get(path) {
				for (rpath, mod_id, prio) in files {
					if *prio >= priority {continue}
					if *prio < real_path_prio {continue}
					real_path = Some((mod_id, rpath));
					real_path_prio = *prio;
				}
			}
		}
		
		if let Some((mod_id, path)) = real_path {
			// log!("Loading file {path} from mod {mod_id} to overlay onto");
			crate::resource_loader::load_file_disk(&root.join(&mod_id).join(path)).ok()
		} else {
			// log!("Loading file {path} from game to overlay onto");
			crate::noumenon()?.file(path).ok()
		}
	};
	
	let files_compdata = std::rc::Rc::new(std::cell::RefCell::new(File::open(files_dir.join("_compdata")).ok().and_then(|v| zip::ZipArchive::new(std::io::BufReader::new(v)).ok())));
	let read_compdata = |path: &str| {
		if let Some(compdata) = &mut *files_compdata.borrow_mut() {
			if let Ok(mut f) = compdata.by_name(&format!("files/{path}")) {
				if f.is_file() {
					let mut buf = Vec::new();
					f.read_to_end(&mut buf).ok()?;
					return Some(buf);
				}
			}
		}
		
		None
	};
	
	let file_resolver = |path: &crate::modman::Path| {
		match path {
			crate::modman::Path::Mod(path) => {
				let Some(true_path) = remap.get(path) else {return None};
				
				if let Some(data) = read_compdata(&true_path) {
					return Some(Cow::Owned(data));
				}
				
				crate::resource_loader::load_file_disk::<Vec<u8>>(&files_dir.join(true_path)).ok().map(|v| Cow::Owned::<Vec<u8>>(v))
			}
			
			crate::modman::Path::Game(path) => {
				get_file(&path, &collection_id, priority).map(|v| Cow::Owned(v))
			}
			
			crate::modman::Path::Option(id, sub_id) => {
				let Some(setting) = settings.get(id) else {return None};
				let crate::modman::settings::Value::Path(i) = setting else {return None};
				let Some(option) = meta.options.options_iter().find(|v| v.name == *id) else {return None};
				let meta::OptionSettings::Path(v) = &option.settings else {return None};
				let Some((_, paths)) = v.options.get(*i as usize) else {return None};
				let Some(path) = paths.iter().find(|v| v.0 == *sub_id) else {return None};
				match &path.1 {
					crate::modman::meta::ValuePathPath::Mod(path) => {
						let Some(true_path) = remap.get(path) else {return None};
						crate::resource_loader::load_file_disk::<Vec<u8>>(&files_dir.join(true_path)).ok().map(|v| Cow::Owned(v))
					}
				}
			}
		}
	};
	
	// ----------
	
	let mut files_done = 0;
	let mut total_files = 0;
	
	// count how much we got to do
	total_files += meta.files.len();
	for option in meta.options.options_iter() {
		let Some(value) = settings.get(&option.name) else {continue};
		
		match &option.settings {
			meta::OptionSettings::SingleFiles(v) => {
				let crate::modman::settings::Value::SingleFiles(value) = value else {continue};
				if let Some(o) = v.options.get(*value as usize) {
					total_files += o.files.len();
				}
			}
			
			meta::OptionSettings::MultiFiles(v) => {
				let crate::modman::settings::Value::MultiFiles(value) = value else {continue};
				for (i, o) in v.options.iter().enumerate() {
					if value & (1 << i) != 0 {
						total_files += o.files.len();
					}
				}
			}
			
			_ => {}
		}
	}
	
	// actually do the thing
	let mut p_option = POption {
		Name: collection_id.to_owned(),
		Description: Some(String::new()),
		Priority: Some(1),
		Files: HashMap::new(),
		FileSwaps: HashMap::new(),
		Manipulations: Vec::new(),
	};
	
	let mut final_ui_colors = HashMap::<(bool, u32), OptionOrStatic<[f32; 3]>>::new();
	
	// let mut add_datas = |files: &HashMap<String, String>, swaps: &HashMap<String, String>, manips: &Vec<meta::Manipulation>| -> Result<(), crate::resource_loader::BacktraceError> {
	let mut add_datas = |files: &HashMap<&str, &str>, swaps: &HashMap<&str, &str>, _manips: &Vec<&meta::Manipulation>| -> Result<(), crate::resource_loader::BacktraceError> {
		// use crate::modman::composite::*;
		
		for (game_path, real_path) in files {
			progress.set(files_done as f32 / total_files as f32);
			progress.set_msg(game_path);
			
			let game_path_stripped = game_path.strip_suffix(".comp").unwrap_or(game_path);
			if let Some(whitelist) = &file_whitelist {
				if !whitelist.contains(game_path_stripped) {
					files_done += 1;
					continue;
				}
			}
			
			let Some(real_path_remapped) = remap.get(*real_path) else {
				log!("remap did not contain {real_path}");
				continue;
			};
			let mut true_real_path = format!("files/{real_path_remapped}");
			
			if game_path.ends_with(".comp") {
				// log!(log, "compositing file {game_path}");
				let ext = game_path.trim_end_matches(".comp").split(".").last().unwrap();
				let comp = read_compdata(real_path_remapped)
					.or_else(|| crate::resource_loader::read_utf8(&files_dir.join(real_path_remapped)).ok())
					.and_then(|v| crate::modman::composite::open_composite(ext, &v));
				
				// let comp: Option<Box<dyn Composite>> = match ext {
				// 	"tex" | "atex" => Some(Box::new(read_json::<tex::Tex>(&files_dir.join(real_path_remapped))?)),
				// 	_ => None
				// };
				
				if let Some(comp) = comp {
					match comp.composite(&meta, &settings, &file_resolver) {
						Ok(data) => {
							// log!(log, "Succeeded to composite file {game_path}");
							
							let hash = if game_path.starts_with("ui/") {
								crate::hash_str(blake3::hash(format!("{game_path}{collection_id}").as_bytes()))
							} else {
								crate::hash_str(blake3::hash(&data))
							};
							
							std::fs::write(files_comp_dir.join(format!("{hash}.{ext}")), data)?;
							true_real_path = format!("files_comp/{hash}.{ext}")
						}
						
						Err(err) => {
							log!(log, "Failed to composite file {game_path} ({err:?})");
							continue;
						}
					}
				}
			}
			
			p_option.Files.insert(game_path_stripped.to_owned(), true_real_path);
			changed_files.insert(game_path_stripped.to_owned());
			
			files_done += 1;
		}
		
		for (a, b) in swaps {
			if p_option.FileSwaps.contains_key(*a) {continue}
			p_option.FileSwaps.insert(a.to_string(), b.to_string());
		}
		
		// for manip in manips {
		// 	match manip {
		// 		meta::Manipulation::Imc {
		// 			attribute_and_sound,
		// 			material_id,
		// 			decal_id,
		// 			vfx_id,
		// 			material_animation_id,
		// 			attribute_mask,
		// 			sound_id,
		// 			
		// 			primary_id,
		// 			secondary_id,
		// 			variant,
		// 			object_type,
		// 			equip_slot,
		// 			body_slot,
		// 		} => p_option.Manipulations.push(PManipulation::Imc {
		// 			Entry: ImcEntry {
		// 				AttributeAndSound: Some(*attribute_and_sound),
		// 				MaterialId: Some(*material_id),
		// 				DecalId: Some(*decal_id),
		// 				VfxId: Some(*vfx_id),
		// 				MaterialAnimationId: Some(*material_animation_id),
		// 				AttributeMask: Some(*attribute_mask),
		// 				SoundId: Some(*sound_id),
		// 			},
		// 			PrimaryId: *primary_id,
		// 			SecondaryId: *secondary_id,
		// 			Variant: *variant,
		// 			ObjectType: object_type.to_owned(),
		// 			EquipSlot: equip_slot.to_owned(),
		// 			BodySlot: body_slot.to_owned(),
		// 		}),
		// 		
		// 		meta::Manipulation::Eqdp {
		// 			entry,
		// 			set_id,
		// 			slot,
		// 			race,
		// 			gender,
		// 		} => p_option.Manipulations.push(PManipulation::Eqdp {
		// 			Entry: *entry,
		// 			SetId: *set_id,
		// 			Slot: slot.to_owned(),
		// 			Race: race.to_owned(),
		// 			Gender: gender.to_owned(),
		// 		}),
		// 		
		// 		meta::Manipulation::Eqp {
		// 			entry,
		// 			set_id,
		// 			slot,
		// 		} => p_option.Manipulations.push(PManipulation::Eqp {
		// 			Entry: *entry,
		// 			SetId: *set_id,
		// 			Slot: slot.to_owned(),
		// 		}),
		// 		
		// 		meta::Manipulation::Est {
		// 			entry,
		// 			set_id,
		// 			slot,
		// 			race,
		// 			gender,
		// 		} => p_option.Manipulations.push(PManipulation::Est {
		// 			Entry: *entry,
		// 			SetId: *set_id,
		// 			Slot: slot.to_owned(),
		// 			Race: race.to_owned(),
		// 			Gender: gender.to_owned(),
		// 		}),
		// 		
		// 		meta::Manipulation::Gmp {
		// 			enabled,
		// 			animated,
		// 			rotation_a,
		// 			rotation_b,
		// 			rotation_c,
		// 			unknown_a,
		// 			unknown_b,
		// 			unknown_total,
		// 			value,
		// 			
		// 			set_id,
		// 		} => p_option.Manipulations.push(PManipulation::Gmp {
		// 			Entry: GmpEntry {
		// 				Enabled: Some(*enabled),
		// 				Animated: Some(*animated),
		// 				RotationA: Some(*rotation_a),
		// 				RotationB: Some(*rotation_b),
		// 				RotationC: Some(*rotation_c),
		// 				UnknownA: Some(*unknown_a),
		// 				UnknownB: Some(*unknown_b),
		// 				UnknownTotal: Some(*unknown_total),
		// 				Value: Some(*value),
		// 			},
		// 			SetId: *set_id,
		// 		}),
		// 		
		// 		meta::Manipulation::Rsp {
		// 			entry,
		// 			sub_race,
		// 			attribute,
		// 		} => p_option.Manipulations.push(PManipulation::Rsp {
		// 			Entry: *entry,
		// 			SubRace: sub_race.to_owned(),
		// 			Attribute: attribute.to_owned(),
		// 		})
		// 	}
		// }
		
		Ok(())
	};
	
	fn contains<V>(map: &HashMap<&str, V>, key: &str) -> bool {
		return if let Some(k) = key.strip_suffix(".comp") {
			map.contains_key(k)
		} else {
			map.contains_key(format!("{key}.comp").as_str())
		} | map.contains_key(key);
	}
	
	for option in meta.options.options_iter().rev() {
		let Some(value) = settings.get(&option.name) else {continue};
		
		match &option.settings {
			meta::OptionSettings::SingleFiles(v) => {
				log!("SingleFiles: {value:?}");
				let crate::modman::settings::Value::SingleFiles(value) = value else {continue};
				log!("setting is correct");
				if let Some(o) = v.options.get(*value as usize) {
					let mut files = o.files.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<HashMap<_, _>>();
					let mut file_swaps = o.file_swaps.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<HashMap<_, _>>();
					let mut manipulations = o.manipulations.iter().map(|v| v).collect::<Vec<_>>();
					let mut ui_colors = o.ui_colors.iter().map(|v| v).collect::<Vec<_>>();
					
					let mut inherit_o = o;
					loop {
						let Some(inherit_name) = &inherit_o.inherit else {break};
						let Some(inherit) = v.options.iter().find(|v| v.name == *inherit_name) else {break};
						log!("Inherit: {inherit_name}");
						inherit_o = inherit;
						
						inherit_o.files.iter().for_each(|(k, v)| if !contains(&files, k.as_str()) {log!("{k} does not exist in option, adding from inherit"); files.insert(k, v);});
						inherit_o.file_swaps.iter().for_each(|(k, v)| if !file_swaps.contains_key(k.as_str()) {file_swaps.insert(k, v);});
						inherit_o.manipulations.iter().for_each(|v| if !manipulations.contains(&v) {manipulations.push(v);});
						inherit_o.ui_colors.iter().for_each(|v| if !ui_colors.contains(&v) {ui_colors.push(v);});
					}
					
					add_datas(&files, &file_swaps, &manipulations)?;
					
					for color in ui_colors {
						final_ui_colors.insert((color.use_theme, color.index), color.color.clone());
					}
				}
			}
			
			meta::OptionSettings::MultiFiles(v) => {
				log!("MultiFiles: {value:?}");
				let crate::modman::settings::Value::MultiFiles(value) = value else {continue};
				log!("setting is correct");
				for (i, o) in v.options.iter().enumerate() {
					if value & (1 << i) != 0 {
						let files = o.files.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<HashMap<_, _>>();
						let file_swaps = o.file_swaps.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<HashMap<_, _>>();
						let manipulations = o.manipulations.iter().map(|v| v).collect::<Vec<_>>();
						add_datas(&files, &file_swaps, &manipulations)?;
						
						for color in &o.ui_colors {
							final_ui_colors.insert((color.use_theme, color.index), color.color.clone());
						}
					}
				}
			}
			
			_ => {}
		}
	}
	
	// add_datas(&meta.files, &meta.file_swaps, &meta.manipulations)?;
	{
		let files = meta.files.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<HashMap<_, _>>();
		let file_swaps = meta.file_swaps.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<HashMap<_, _>>();
		let manipulations = meta.manipulations.iter().map(|v| v).collect::<Vec<_>>();
		add_datas(&files, &file_swaps, &manipulations)?;
		
		for color in &meta.ui_colors {
			final_ui_colors.insert((color.use_theme, color.index), color.color.clone());
		}
	}
	
	// update penumbra mod
	let mut group = match read_json::<PGroup>(&mod_dir.join("group_001__collection.json")) {
		Ok(v) => v,
		Err(_) => PGroup {
			Name: "_collection".to_string(),
			Description: Some("Aetherment managed\nDON'T TOUCH THIS".to_string()),
			Priority: 1,
			Type: "Single".to_string(),
			DefaultSettings: Some(0),
			Options: Vec::new(),
		}
	};
	
	if let Some(option) = group.Options.iter_mut().find(|v| v.Name == collection_id) {
		*option = p_option;
	} else {
		group.Options.push(p_option);
	}
	
	File::create(mod_dir.join("group_001__collection.json"))?.write_all(crate::json_pretty(&group)?.as_bytes())?;
	
	reload_mod(&mod_id);
	set_mod_settings(&collection_id, &mod_id, "_collection", vec![&collection_id]);
	
	// save ui colors
	if get_collection(crate::modman::backend::CollectionType::Interface).id == collection_id {
		std::fs::write(aeth_dir.join("uicolorcache"), serde_json::to_vec(&final_ui_colors.iter().map(|((a, b), c)| (a, b, c)).collect::<Vec<_>>())?)?;
	}
	
	Ok(changed_files)
}

fn cleanup_mod(mod_id: &str) {
	let root = root_path();
	let mod_dir = root.join(mod_id);
	let Ok(group) = read_json::<PGroup>(&mod_dir.join("group_001__collection.json")) else {return};
	let Ok(read_dir) = std::fs::read_dir(mod_dir.join("files_comp")) else {return};
	
	let mut files = HashSet::new();
	for o in &group.Options {
		let s = get_mod_settings(&o.Name, mod_id, false);
		if s.enabled {
			for (_, path) in &o.Files {
				files.insert(path.split("/").last().unwrap());
			}
		}
	}
	
	for entry in read_dir {
		if let Ok(entry) = entry {
			if !files.contains(entry.file_name().to_string_lossy().as_ref()) {
				_ = std::fs::remove_file(entry.path())
			}
		}
	}
}

fn get_mod_cache() -> ModFileCache {
	let mut mod_file_cache = ModFileCache::new();
	let collections = get_collections();
	for col in &collections {
		mod_file_cache.insert(col.id.clone(), HashMap::new());
	}
	
	let mut insert = |collection_id: &str, mod_id: &str, priority: i32, game_path: &str, real_path: &str| {
		mod_file_cache.get_mut(collection_id).unwrap().entry(game_path.to_owned()).or_insert_with(|| Vec::new()).push((real_path.to_owned(), mod_id.to_owned(), priority))
	};
	
	let root = root_path();
	for mod_id in mod_list() {
		let Some(groups) = get_mod_groups(&root.join(&mod_id)) else {continue};
		
		let default = match read_json::<PDefaultMod>(&root.join(&mod_id).join("default_mod.json")) {
			Ok(v) => v,
			Err(e) => {log!(err, "Failed to load or parse default_mod.json for mod {mod_id}\n{e:?}"); continue},
		};
		
		for col in &collections {
			let settings = get_mod_settings(&col.id, &mod_id, false);
			if !settings.exists || !settings.enabled {continue}
			let priority = settings.priority;
			
			for (game_path, real_path) in &default.Files {
				insert(&col.id, &mod_id, priority, game_path, real_path);
			}
			
			let options = settings.options;
			for (option, enabled_sub_options) in &options {
				if enabled_sub_options.len() == 0 {continue}
				let Some(group) = groups.get(option.as_str()) else {log!(err, "Failed to find group file ({option}) for mod ({mod_id})"); continue};
				
				for o in &group.Options {
					if enabled_sub_options.contains(&o.Name) {
						for (game_path, real_path) in &o.Files {
							insert(&col.id, &mod_id, priority, game_path, real_path);
						}
					}
				}
			}
		}
	}
	
	mod_file_cache
}

fn get_mod_files(collection_id: &str, mod_id: &str) -> Option<HashMap<String, String>> {
	let mut files = HashMap::new();
	let root = root_path();
	let mut groups = get_mod_groups(&root.join(&mod_id))?;
	
	let default = match read_json::<PDefaultMod>(&root.join(&mod_id).join("default_mod.json")) {
		Ok(v) => v,
		Err(e) => {log!(err, "Failed to load or parse default_mod.json for mod {mod_id}\n{e:?}"); return None},
	};
	
	let settings = get_mod_settings(collection_id, mod_id, false);
	if !settings.exists || !settings.enabled {return None};
	
	for (game_path, real_path) in default.Files {
		files.insert(game_path, real_path);
	}
	
	let options = settings.options;
	for (option, enabled_sub_options) in &options {
		if enabled_sub_options.len() == 0 {continue}
		let Some(group) = groups.remove(option.as_str()) else {log!(err, "Failed to find group file ({option}) for mod ({mod_id})"); continue};
		
		for o in group.Options {
			if enabled_sub_options.contains(&o.Name) {
				for (game_path, real_path) in o.Files {
					files.insert(game_path, real_path);
				}
			}
		}
	}
	
	Some(files)
}

fn get_composite_info(mods: Vec<&str>) -> CompositeInfo {
	let mut comp_info = CompositeInfo {
		composite_links: HashMap::new(),
		composite_caches: HashMap::new(),
	};
	
	let root = root_path();
	for mod_id in mods {
		let mod_dir = root.join(mod_id);
		let composite_cache = if let Ok(c) = read_json::<crate::modman::composite::CompositeCache>(&mod_dir.join("aetherment").join("compcache")) {c} else {
			match crate::modman::composite::build_cache(&mod_dir) {
				Ok(v) => v,
				Err(_) => continue,
			}
		};
		
		for (_comp, game_paths) in &composite_cache.composite_external_files {
			for path in game_paths {
				comp_info.composite_links.entry(path.to_owned()).or_insert_with(|| HashSet::new()).insert(mod_id.to_owned());
			}
		}
		
		comp_info.composite_caches.insert(mod_id.to_owned(), composite_cache);
	}
	
	comp_info
}

fn get_mod_groups(path: &std::path::Path) -> Option<HashMap<String, PGroup>> {
	let mod_id = path.file_name()?.to_string_lossy().to_owned();
	Some(std::fs::read_dir(path).ok()?
		.into_iter()
		.filter_map(|v| v.ok())
		.filter_map(|v| {
			let file_name = v.file_name().to_string_lossy().into_owned();
			if file_name.starts_with("group_") {
				match read_json::<PGroup>(&v.path()) {
					Ok(v) => Some((v.Name.clone(), v)),
					Err(e) => {log!(err, "Failed to load or parse group file {file_name} for mod {mod_id}\n{e:?}"); None},
				}
			} else {None}
		}).collect::<HashMap<_, _>>())
}

// output of c# Path.GetInvalidFileNameChars()
// static INVALID_CHARS: [u8; 41] = [34, 60, 62, 124, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 58, 42, 63, 92, 47];
// fn clean_path(path: &str) -> String {
// 	// path.chars().filter_map(|v| if !INVALID_CHARS.contains(&(v as u8)) {Some(v.to_ascii_lowercase())} else {None}).collect::<String>()
// 	path.trim().chars().map(|v| if !INVALID_CHARS.contains(&(v as u8)) {v.to_ascii_lowercase()} else {'_'}).collect::<String>()
// }

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct PMeta {
	FileVersion: i32,
	Name: String,
	Author: String,
	Description: String,
	Version: String,
	Website: String,
	ModTags: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct PDefaultMod {
	// Name: String,
	// Description: String,
	Files: HashMap<String, String>,
	FileSwaps: HashMap<String, String>,
	Manipulations: Vec<PManipulation>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct PGroup {
	Name: String,
	Description: Option<String>,
	Priority: i32,
	Type: String,
	DefaultSettings: Option<u32>,
	Options: Vec<POption>,
}

#[derive(Debug, Deserialize, Serialize)]
struct POption {
	Name: String,
	Description: Option<String>,
	Priority: Option<u32>, // used for multi
	Files: HashMap<String, String>,
	FileSwaps: HashMap<String, String>,
	Manipulations: Vec<PManipulation>,
}

type PManipulation = serde_json::Value;
// #[allow(non_snake_case)]
// #[derive(Debug, Deserialize, Serialize)]
// #[serde(tag = "Type", content = "Manipulation")]
// pub(crate) enum PManipulation {
// 	Imc {
// 		Entry: ImcEntry,
// 		PrimaryId: i32,
// 		SecondaryId: i32,
// 		Variant: i32,
// 		ObjectType: String,
// 		EquipSlot: String,
// 		BodySlot: String,
// 	},
// 	
// 	Eqdp {
// 		Entry: u64,
// 		SetId: i32,
// 		Slot: String,
// 		Race: String,
// 		Gender: String,
// 	},
// 	
// 	Eqp {
// 		Entry: u64,
// 		SetId: i32,
// 		Slot: String,
// 	},
// 	
// 	Est {
// 		Entry: u64,
// 		SetId: i32,
// 		Slot: String,
// 		Race: String,
// 		Gender: String,
// 	},
// 	
// 	Gmp {
// 		Entry: GmpEntry,
// 		SetId: i32,
// 	},
// 	
// 	Rsp {
// 		Entry: f32,
// 		SubRace: String,
// 		Attribute: String,
// 	},
// }
// 
// impl PManipulation {
// 	pub fn convert_to_aeth(self) -> meta::Manipulation {
// 		match self {
// 			Self::Imc {
// 				Entry,
// 				PrimaryId,
// 				SecondaryId,
// 				Variant,
// 				ObjectType,
// 				EquipSlot,
// 				BodySlot,
// 			} => meta::Manipulation::Imc {
// 				attribute_and_sound: Entry.AttributeAndSound.unwrap_or(0),
// 				material_id: Entry.MaterialId.unwrap_or(0),
// 				decal_id: Entry.DecalId.unwrap_or(0),
// 				vfx_id: Entry.VfxId.unwrap_or(0),
// 				material_animation_id: Entry.MaterialAnimationId.unwrap_or(0),
// 				attribute_mask: Entry.AttributeMask.unwrap_or(0),
// 				sound_id: Entry.SoundId.unwrap_or(0),
// 				
// 				primary_id: PrimaryId,
// 				secondary_id: SecondaryId,
// 				variant: Variant,
// 				object_type: ObjectType,
// 				equip_slot: EquipSlot,
// 				body_slot: BodySlot,
// 			},
// 			
// 			Self::Eqdp {
// 				Entry,
// 				SetId,
// 				Slot,
// 				Race,
// 				Gender,
// 			} => meta::Manipulation::Eqdp {
// 				entry: Entry,
// 				set_id: SetId,
// 				slot: Slot,
// 				race: Race,
// 				gender: Gender,
// 			},
// 			
// 			Self::Eqp {
// 				Entry,
// 				SetId,
// 				Slot,
// 			} => meta::Manipulation::Eqp {
// 				entry: Entry,
// 				set_id: SetId,
// 				slot: Slot,
// 			},
// 			
// 			Self::Est {
// 				Entry,
// 				SetId,
// 				Slot,
// 				Race,
// 				Gender,
// 			} => meta::Manipulation::Est {
// 				entry: Entry,
// 				set_id: SetId,
// 				slot: Slot,
// 				race: Race,
// 				gender: Gender,
// 			},
// 			
// 			Self::Gmp {
// 				Entry,
// 				SetId,
// 			} => meta::Manipulation::Gmp {
// 				enabled: Entry.Enabled.unwrap_or(true),
// 				animated: Entry.Animated.unwrap_or(true),
// 				rotation_a: Entry.RotationA.unwrap_or(0),
// 				rotation_b: Entry.RotationB.unwrap_or(0),
// 				rotation_c: Entry.RotationC.unwrap_or(0),
// 				unknown_a: Entry.UnknownA.unwrap_or(0),
// 				unknown_b: Entry.UnknownB.unwrap_or(0),
// 				unknown_total: Entry.UnknownTotal.unwrap_or(0),
// 				value: Entry.Value.unwrap_or(0),
// 				
// 				set_id: SetId,
// 			},
// 			
// 			Self::Rsp {
// 				Entry,
// 				SubRace,
// 				Attribute,
// 			} => meta::Manipulation::Rsp {
// 				entry: Entry,
// 				sub_race: SubRace,
// 				attribute: Attribute,
// 			},
// 		}
// 	}
// }
// 
// // No clue if the options are needed, but during testing i had 1 group of 1 mod that didnt contain AttributeAndSound in the ImcEntry
// // and i just dont want to figure out what is optional and what isnt since the penumbra meta structs(if you can even call it that) arent cleanly laid out
// #[derive(Debug, Deserialize, Serialize)]
// pub(crate) struct ImcEntry {
// 	AttributeAndSound: Option<i32>,
// 	MaterialId: Option<i32>,
// 	DecalId: Option<i32>,
// 	VfxId: Option<i32>,
// 	MaterialAnimationId: Option<i32>,
// 	AttributeMask: Option<i32>,
// 	SoundId: Option<i32>,
// }
// 
// #[derive(Debug, Deserialize, Serialize)]
// pub(crate) struct GmpEntry {
// 	Enabled: Option<bool>,
// 	Animated: Option<bool>,
// 	RotationA: Option<i32>,
// 	RotationB: Option<i32>,
// 	RotationC: Option<i32>,
// 	UnknownA: Option<i32>,
// 	UnknownB: Option<i32>,
// 	UnknownTotal: Option<i32>,
// 	Value: Option<u64>,
// }

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct SortOrder {
	pub Data: HashMap<String, String>,
	pub EmptyFolders: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Collection {
	pub Version: i32,
	pub Name: String,
	pub Settings: HashMap<String, CollectionModSettings>,
	pub Inheritance: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CollectionModSettings {
	pub Settings: HashMap<String, i32>,
	pub Priority: i32,
	pub Enabled: bool,
}