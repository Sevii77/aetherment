use std::{io::{Read, Seek}, sync::{atomic::AtomicU32, Arc, RwLock, RwLockReadGuard}};

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
pub struct TaskProgress {
	task_count: Arc<AtomicU32>,
	task_progress: Arc<AtomicU32>,
	task_msg: Arc<RwLock<String>>,
	pub sub_task: Progress,
}

impl TaskProgress {
	pub fn new() -> Self {
		Self {
			task_count: Arc::new(AtomicU32::new(0)),
			task_progress: Arc::new(AtomicU32::new(0)),
			task_msg: Arc::new(RwLock::new(String::new())),
			sub_task :Progress::new(),
		}
	}
	
	pub fn is_finished(&self) -> bool {
		self.task_progress.load(std::sync::atomic::Ordering::Relaxed) == self.task_count.load(std::sync::atomic::Ordering::Relaxed)
	}
	
	pub fn set_task_count(&self, value: usize) {
		self.task_count.store(value as u32, std::sync::atomic::Ordering::Relaxed);
	}
	
	pub fn add_task_count(&self, value: usize) {
		self.task_count.fetch_add(value as u32, std::sync::atomic::Ordering::Relaxed);
	}
	
	pub fn progress_task(&self) {
		self.task_progress.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
	}
	
	pub fn get_task_progress(&self) -> f32 {
		self.task_progress.load(std::sync::atomic::Ordering::Relaxed) as f32 / self.task_count.load(std::sync::atomic::Ordering::Relaxed) as f32
	}
	
	pub fn get_task_msg(&self) -> RwLockReadGuard<String> {
		self.task_msg.read().unwrap()
	}
	
	pub fn set_task_msg(&self, value: impl Into<String>) {
		*self.task_msg.write().unwrap() = value.into();
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
			msg: Arc::new(RwLock::new(String::new())),
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
	
	pub fn set_msg(&self, value: impl Into<String>) {
		*self.msg.write().unwrap() = value.into();
	}
}

#[derive(Debug)]
pub struct Collection {
	pub name: String,
	pub id: String,
}

impl Collection {
	pub fn is_valid(&self) -> bool {
		self.id != "00000000-0000-0000-0000-000000000000"
	}
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
	fn install_mods_path(&mut self, progress: TaskProgress, files: Vec<std::path::PathBuf>) {
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
	fn install_mods(&mut self, progress: TaskProgress, files: Vec<(String, std::fs::File)>);
	
	fn apply_mod_settings(&mut self, mod_id: &str, collection_id: &str, settings: SettingsType);
	fn finalize_apply(&mut self, progress: TaskProgress);
	fn apply_queue_size(&self) -> usize;
	
	fn apply_services(&self);
	
	// fn get_aeth_meta(&self, mod_id: &str) -> Option<super::meta::Meta>;
	
	fn load_mods(&mut self);
	fn get_mod_meta(&self, mod_id: &str) -> Option<&crate::modman::meta::Meta>;
	fn get_mod_asset(&self, mod_id: &str, path: &str) -> std::io::Result<Vec<u8>>;
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

// trimmed https://github.com/Ottermandias/Penumbra.Api/blob/552246e595ffab2aaba2c75f578d564f8938fc9a/Enums/ApiCollectionType.cs
#[derive(Debug, Clone)]
#[repr(u8)]
pub enum CollectionType {
	Yourself  = 0,
	Default   = 0xE0,
	Interface = 0xE1,
	Current   = 0xE2,
}

impl crate::EnumTools for CollectionType {
	type Iterator = std::array::IntoIter<Self, 4>;
	
	fn to_str(&self) -> &'static str {
		match self {
			CollectionType::Yourself => "Yourself",
			CollectionType::Default => "Base",
			CollectionType::Interface => "Interface",
			CollectionType::Current => "Current",
		}
	}
	
	fn iter() -> Self::Iterator {
		[
			Self::Yourself,
			Self::Default,
			Self::Interface,
			Self::Current,
		].into_iter()
	}
}