#[macro_use]
mod log;
mod resource_loader;
mod render_helper;
mod config;
pub mod modman;
mod view;
pub mod remote;

pub use log::LogType;
// pub use renderer;

pub extern crate renderer;
pub use noumenon as noumenon_; // idk what to call it

static mut CONFIG: Option<config::ConfigManager> = None;
pub fn config() -> &'static mut config::ConfigManager {
	unsafe{CONFIG.get_or_insert_with(|| config::ConfigManager::load(&dirs::config_dir().unwrap().join("Aetherment").join("config.json")))}
}

// static mut BACKEND: Option<std::sync::Mutex<Box<dyn modman::backend::Backend>>> = None;
// pub fn backend() -> std::sync::MutexGuard<'static, Box<dyn modman::backend::Backend>> {
// 	unsafe{BACKEND.as_mut().unwrap().lock().unwrap()}
// }

// not thread safe (probably), being used across threads, it will bite me in the ass
// TODO: fix
static mut BACKEND: Option<Box<dyn modman::backend::Backend>> = None;
pub fn backend() -> &'static mut Box<dyn modman::backend::Backend> {
	unsafe{BACKEND.as_mut().unwrap()}
}

static mut NOUMENON: Option<Option<noumenon::Noumenon>> = None;
#[cfg(feature = "plugin")]
pub fn noumenon() -> Option<&'static noumenon::Noumenon> {
	unsafe{NOUMENON.get_or_insert_with(|| noumenon::get_noumenon(Some(std::env::current_exe().unwrap().parent().unwrap().parent().unwrap()))).as_ref()}
}
#[cfg(not(feature = "plugin"))]
pub fn noumenon() -> Option<&'static noumenon::Noumenon> {
	unsafe{NOUMENON.get_or_insert_with(|| noumenon::get_noumenon(config().config.game_install.as_ref())).as_ref()}
}

pub fn hash_str(hash: blake3::Hash) -> String {
	// base64::encode_config(hash.as_bytes(), base64::URL_SAFE_NO_PAD)
	base32::encode(base32::Alphabet::Rfc4648HexLower{padding:false}, &hash.as_bytes()[..16])
}

pub fn json_pretty<T: serde::Serialize>(data: &T) -> Result<String, serde_json::Error> {
	// serde_json::to_writer_pretty(&mut File::create(path)?, self)?;
	let mut serializer = serde_json::Serializer::with_formatter(Vec::new(), serde_json::ser::PrettyFormatter::with_indent(b"\t"));
	data.serialize(&mut serializer)?;
	Ok(String::from_utf8(serializer.into_inner()).unwrap())
}

pub struct Core {
	mods_tab: view::mods::Mods,
	browser_tab: view::browser::Browser,
	tools_tab: view::tool::Tools,
	
	current_tab: &'static str,
	
	install_progress: crate::modman::backend::InstallProgress,
	apply_progress: crate::modman::backend::ApplyProgress,
}

impl Core {
	pub fn new(log: fn(log::LogType, String), backend_initializers: modman::backend::BackendInitializers) -> Self {
		unsafe {
			log::LOG = log;
			// BACKEND = Some(std::sync::Mutex::new(modman::backend::new_backend(backend_initializers)));
			BACKEND = Some(modman::backend::new_backend(backend_initializers));
		}
		
		// backend().load_mods();
		
		let mut s = Self {
			mods_tab: view::mods::Mods::new(),
			browser_tab: view::browser::Browser::new(),
			tools_tab: view::tool::Tools::new(),
			
			current_tab: "Mods",
			
			install_progress: crate::modman::backend::InstallProgress::new(),
			apply_progress: crate::modman::backend::ApplyProgress::new(),
		};
		
		s.install_progress.apply = s.apply_progress.clone();
		
		let progress = s.install_progress.clone();
		std::thread::spawn(move || {
			remote::check_updates(progress);
		});
		
		s
	}
	
	pub fn draw(&mut self, ui: &mut renderer::Ui) {
		ui.tabs(&["Mods", "Browser", "Settings", "Tools", "Debug"], &mut self.current_tab);
		
		// TOOD: make fancy
		if self.install_progress.is_busy() {
			ui.label(format!("{:.0}% {}", self.install_progress.mods.get() * 100.0, self.install_progress.mods.get_msg()));
			ui.label(format!("{:.0}% {}", self.install_progress.current_mod.get() * 100.0, self.install_progress.current_mod.get_msg()));
		}
		
		if self.apply_progress.is_busy() {
			ui.label(format!("{:.0}% {}", self.apply_progress.mods.get() * 100.0, self.apply_progress.mods.get_msg()));
			ui.label(format!("{:.0}% {}", self.apply_progress.current_mod.get() * 100.0, self.apply_progress.current_mod.get_msg()));
		}
		
		match self.current_tab {
			"Mods" => {
				self.mods_tab.draw(ui, self.install_progress.clone(), self.apply_progress.clone());
			}
			
			"Browser" => {
				self.browser_tab.draw(ui, self.install_progress.clone());
			}
			
			"Settings" => {
				
			}
			
			"Tools" => {
				self.tools_tab.draw(ui);
			}
			
			"Debug" => {
				ui.debug();
			}
			
			_ => {
				ui.label("How did you get here, this tab is not supposed to be a thing")
			}
		}
	}
}