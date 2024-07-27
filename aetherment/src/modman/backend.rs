use std::{io::{Read, Seek}, sync::{atomic::{AtomicBool, AtomicU32}, Arc, RwLock, RwLockReadGuard}};

#[allow(non_snake_case)]
pub mod penumbra_ipc;
pub mod dummy;

pub enum Status {
	Ok,
	Error(String),
}

pub enum Filter {
	None,
	Options(Vec<String>),
	Paths(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum SettingsType {
	Some(crate::modman::settings::CollectionSettings),
	Clear,
	Keep,
}

#[derive(Clone)]
pub struct InstallProgress {
	busy: Arc<AtomicBool>,
	pub mods: Progress,
	pub current_mod: Progress,
	pub apply: ApplyProgress
}

impl InstallProgress {
	pub fn new() -> Self {
		Self {
			busy: Arc::new(AtomicBool::new(false)),
			mods: Progress::new(),
			current_mod: Progress::new(),
			apply: ApplyProgress::new(),
		}
	}
	
	pub fn is_busy(&self) -> bool {
		self.busy.load(std::sync::atomic::Ordering::Relaxed)
	}
	
	pub fn set_busy(&self, value: bool) {
		self.busy.store(value, std::sync::atomic::Ordering::Relaxed)
	}
}

#[derive(Clone)]
pub struct ApplyProgress {
	busy: Arc<AtomicBool>,
	pub mods: Progress,
	pub current_mod: Progress,
}

impl ApplyProgress {
	pub fn new() -> Self {
		Self {
			busy: Arc::new(AtomicBool::new(false)),
			mods: Progress::new(),
			current_mod: Progress::new(),
		}
	}
	
	pub fn is_busy(&self) -> bool {
		self.busy.load(std::sync::atomic::Ordering::Relaxed)
	}
	
	pub fn set_busy(&self, value: bool) {
		self.busy.store(value, std::sync::atomic::Ordering::Relaxed)
	}
}

#[derive(Clone)]
pub struct Progress {
	inner: Arc<AtomicU32>,
	msg: Arc<RwLock<String>>,
}

impl Progress {
	pub fn new() -> Self {
		Self {
			inner: Arc::new(AtomicU32::new(0)),
			msg: Arc::new(RwLock::new(String::new()))
		}
	}
	
	pub fn get(&self) -> f32 {
		unsafe{std::mem::transmute::<u32, f32>(self.inner.load(std::sync::atomic::Ordering::Relaxed))}
	}
	
	pub fn set(&self, value: f32) {
		self.inner.store(unsafe{std::mem::transmute::<f32, u32>(value)}, std::sync::atomic::Ordering::Relaxed);
	}
	
	pub fn get_msg(&self) -> RwLockReadGuard<String> {
		self.msg.read().unwrap()
	}
	
	pub fn set_msg(&self, value: &str) {
		*self.msg.write().unwrap() = value.to_string();
	}
}

pub struct Collection {
	pub name: String,
	pub id: String,
}

pub trait Backend {
	fn name(&self) -> &'static str;
	fn description(&self) -> &'static str;
	// fn is_functional(&self) -> bool {true}
	fn get_status(&self) -> Status;
	fn get_mods(&self) -> Vec<String>;
	// fn get_mods(&mut self) -> HashMap<String, Mod>;
	fn get_active_collection(&self) -> String;
	fn get_collections(&self) -> Vec<Collection>;
	// fn install_mod(&mut self, file: &std::path::Path) -> Result<String, crate::resource_loader::BacktraceError>;
	fn install_mods_path(&mut self, progress: InstallProgress, files: Vec<std::path::PathBuf>) {
		self.install_mods(progress,files.into_iter()
			// .filter_map(|v| std::fs::File::open(&v).ok().map(|f| (v.file_name().map_or_else(|| String::new(), |v| v.to_string_lossy().to_string()), f)))
			.filter_map(|v| {
				let f = std::fs::File::open(&v).ok()?;
				let mut pack = zip::ZipArchive::new(f).ok()?;
				
				let mut meta_buf = Vec::new();
				pack.by_name("meta.json").ok()?.read_to_end(&mut meta_buf).ok()?;
				let meta = serde_json::from_slice::<super::meta::Meta>(&meta_buf).ok()?;
				
				let mut pack = pack.into_inner();
				_ = pack.seek(std::io::SeekFrom::Start(0));
				Some((meta.name, pack))
			}).collect())
		
	}
	fn install_mods(&mut self, progress: InstallProgress, files: Vec<(String, std::fs::File)>);
	
	fn apply_mod_settings(&mut self, mod_id: &str, collection_id: &str, settings: SettingsType);
	fn finalize_apply(&mut self, progress: ApplyProgress);
	fn apply_queue_size(&self) -> usize;
	
	// fn get_aeth_meta(&self, mod_id: &str) -> Option<super::meta::Meta>;
	
	fn load_mods(&mut self);
	fn get_mod_meta(&self, mod_id: &str) -> Option<&crate::modman::meta::Meta>;
	// fn get_mod_settings(&self, mod_id: &str, collection_id: &str) -> Option<crate::modman::settings::Settings>;
	
	fn get_mod_enabled(&self, mod_id: &str, collection_id: &str) -> bool;
	fn set_mod_enabled(&mut self, mod_id: &str, collection_id: &str, enabled: bool);
	
	fn get_mod_priority(&self, mod_id: &str, collection_id: &str) -> i32;
	fn set_mod_priority(&mut self, mod_id: &str, collection_id: &str, priority: i32);
}

pub enum BackendInitializers {
	PenumbraIpc(penumbra_ipc::PenumbraFunctions),
	None,
}

pub fn new_backend(backend: BackendInitializers) -> Box<dyn Backend> {
	match backend {
		// #[cfg(feature = "plugin")]
		// BackendInitializers::PenumbraIpc(funcs) => Box::new(penumbra_ipc::Penumbra::new(funcs)),
		BackendInitializers::PenumbraIpc(funcs) => {
			penumbra_ipc::initialize_functions(funcs);
			Box::new(penumbra_ipc::Penumbra::new())
		}
		
		_ => Box::new(dummy::Dummy),
	}
}