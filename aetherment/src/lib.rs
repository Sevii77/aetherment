#[macro_use]
mod log;
mod resource_loader;
mod render_helper;
mod config;
pub mod modman;
mod view;

pub use log::LogType;
// pub use renderer;

pub extern crate renderer;
pub use noumenon as noumenon_; // idk what to call it

static mut CONFIG: Option<config::ConfigManager> = None;
pub fn config() -> &'static mut config::ConfigManager {
	unsafe{CONFIG.get_or_insert_with(|| config::ConfigManager::load(&dirs::config_dir().unwrap().join("Aetherment").join("config.json")))}
}

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
	tools_tab: view::tool::Tools,
	
	current_tab: &'static str,
}

impl Core {
	pub fn new(log: fn(log::LogType, String), backend_initializers: modman::backend::BackendInitializers) -> Self {
		unsafe {
			log::LOG = log;
			BACKEND = Some(modman::backend::new_backend(backend_initializers));
		}
		
		backend().load_mods();
		
		Self {
			mods_tab: view::mods::Mods::new(),
			tools_tab: view::tool::Tools::new(),
			
			current_tab: "Mods",
		}
	}
	
	pub fn draw(&mut self, ui: &mut renderer::Ui) {
		ui.tabs(&["Mods", "Settings", "Tools", "Debug"], &mut self.current_tab);
		
		match self.current_tab {
			"Mods" => {
				self.mods_tab.draw(ui);
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