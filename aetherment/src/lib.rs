#![cfg_attr(feature = "plugin", feature(lazy_cell))]
#![allow(static_mut_refs)]

#[macro_use]
mod log;
pub use log::LogType;

mod resource_loader;
mod config;
pub mod modman;
#[cfg(any(feature = "plugin", feature = "client"))] mod ui_ext;
#[cfg(any(feature = "plugin", feature = "client"))] mod view;
#[cfg(any(feature = "plugin", feature = "client"))] mod remote;
#[cfg(any(feature = "plugin", feature = "client"))] pub mod service;
// #[cfg(any(feature = "plugin", feature = "client"))] pub extern crate renderer;
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
#[cfg(any(feature = "plugin", feature = "client"))] 
static mut BACKEND: Option<Box<dyn modman::backend::Backend>> = None;
#[cfg(any(feature = "plugin", feature = "client"))] 
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

pub trait EnumTools {
	type Iterator: core::iter::Iterator<Item = Self>;
	
	fn to_str(&self) -> &'static str;
	fn to_string(&self) -> String {self.to_str().to_string()}
	fn iter() -> Self::Iterator;
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

#[cfg(any(feature = "plugin", feature = "client"))]
pub struct Core {
	views: egui_dock::DockState<Box<dyn view::View>>,
	
	backend_last_error: bool,
	
	install_progress: crate::modman::backend::InstallProgress,
	apply_progress: crate::modman::backend::ApplyProgress,
}

#[cfg(any(feature = "plugin", feature = "client"))]
impl Core {
	pub fn new(log: fn(log::LogType, &str), backend_initializers: modman::backend::BackendInitializers, requirement_initializers: modman::requirement::RequirementInitializers, optional_initializers: modman::meta::OptionalInitializers, services_initializers: service::ServicesInitializers) -> Self {
		unsafe {
			log::LOG = log;
			// BACKEND = Some(std::sync::Mutex::new(modman::backend::new_backend(backend_initializers)));
			BACKEND = Some(modman::backend::new_backend(backend_initializers));
			modman::requirement::initialize(requirement_initializers);
			
			if let Some(dalamud) = optional_initializers.dalamud {
				modman::meta::dalamud::SETSTYLE = dalamud;
			}
			
			service::initialize(services_initializers);
		}
		
		let mut install_progress = crate::modman::backend::InstallProgress::new();
		let apply_progress = crate::modman::backend::ApplyProgress::new();
		install_progress.apply = apply_progress.clone();
		
		let s = Self {
			views: egui_dock::DockState::new(vec![
				Box::new(view::mods::Mods::new(install_progress.clone(), apply_progress.clone())),
				Box::new(view::browser::Browser::new(install_progress.clone())),
				Box::new(view::settings::Settings::new()),
				Box::new(view::tool::Tools::new()),
				Box::new(view::debug::Debug::new()),
			]),
			
			backend_last_error: matches!(backend().get_status(), modman::backend::Status::Error(_)),
			
			install_progress,
			apply_progress,
		};
		
		if !s.backend_last_error {
			let progress = s.install_progress.clone();
			std::thread::spawn(move || {
				backend().apply_services();
				remote::check_updates(progress);
			});
		}
		
		s
	}
	
	pub fn draw(&mut self, ui: &mut egui::Ui) {
		let status = backend().get_status();
		match status {
			modman::backend::Status::Ok => {
				if self.backend_last_error {
					let progress = self.install_progress.clone();
					std::thread::spawn(move || {
						backend().apply_services();
						remote::check_updates(progress);
					});
					
					// self.mods_tab.refresh();
				}
				
				self.backend_last_error = false;
			}
			
			modman::backend::Status::Error(_) => self.backend_last_error = true,
		}
		
		ui.scope(|ui| {
			ui.spacing_mut().item_spacing.y = 0.0;
			let rounding = ui.visuals().widgets.noninteractive.rounding;
			let top = egui::Rounding{ne: rounding.ne, nw: rounding.nw, ..Default::default()};
			
			if self.install_progress.is_busy() {
				ui.add(egui::ProgressBar::new(self.install_progress.mods.get())
					.text(format!("{:.0}% Installing {}", self.install_progress.mods.get() * 100.0, self.install_progress.mods.get_msg()))
					.rounding(top));
				
				ui.add(egui::ProgressBar::new(self.install_progress.current_mod.get())
					.text(format!("{:.0}% Working on {}", self.install_progress.current_mod.get() * 100.0, self.install_progress.current_mod.get_msg()))
					.rounding(egui::Rounding::same(0.0)));
			}
			
			if self.apply_progress.is_busy() {
				ui.add(egui::ProgressBar::new(self.apply_progress.mods.get())
					.text(format!("{:.0}% Applying {}", self.apply_progress.mods.get() * 100.0, self.apply_progress.mods.get_msg()))
					.rounding(if self.install_progress.is_busy() {egui::Rounding::same(0.0)} else {top}));
				
				ui.add(egui::ProgressBar::new(self.apply_progress.current_mod.get())
					.text(format!("{:.0}% Working on {}", self.apply_progress.current_mod.get() * 100.0, self.apply_progress.current_mod.get_msg()))
					.rounding(egui::Rounding::same(0.0)));
			}
		});
		
		let spacing = ui.spacing().item_spacing.y;
		ui.add_space(-spacing);
		
		// TODO: disable mods and browser tab if backend error
		egui_dock::DockArea::new(&mut self.views)
			.id(egui::Id::new("tabs"))
			.style(egui_dock::Style::from_egui(ui.style().as_ref()))
			.draggable_tabs(false)
			.show_close_buttons(false)
			.tab_context_menus(false)
			.show_inside(ui, &mut view::Viewer);
	}
}