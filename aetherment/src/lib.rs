#![allow(static_mut_refs)]

#[macro_use]
mod log;

mod resource_loader;
mod config;
pub mod modman;
#[cfg(any(feature = "plugin", feature = "client"))] mod ui_ext;
#[cfg(any(feature = "plugin", feature = "client"))] mod view;
#[cfg(any(feature = "plugin", feature = "client"))] mod remote;
#[cfg(any(feature = "plugin", feature = "client"))] pub mod service;
pub use noumenon;

static mut CONFIG: Option<config::ConfigManager> = None;
pub fn config() -> &'static mut config::ConfigManager {
	unsafe{CONFIG.get_or_insert_with(|| config::ConfigManager::load(&dirs::config_dir().unwrap().join("Aetherment").join("config.json")))}
}

static mut NOTIFICATION: fn(f32, u8, &str) = |_, _, _| {};
pub fn set_notification(progress: f32, typ: u8, msg: &str) {
	unsafe{NOTIFICATION(progress, typ, msg)}
}

// not thread safe (probably), being used across threads, it will bite me in the ass
// TODO: fix
#[cfg(any(feature = "plugin", feature = "client"))] 
static mut BACKEND: Option<Box<dyn modman::backend::Backend>> = None;
#[cfg(any(feature = "plugin", feature = "client"))] 
pub fn backend() -> &'static Box<dyn modman::backend::Backend> {
	unsafe{BACKEND.as_ref().unwrap()}
}

static mut NOUMENON: Option<Option<noumenon::Noumenon>> = None;
#[cfg(feature = "plugin")]
pub fn noumenon_instance() -> Option<&'static noumenon::Noumenon> {
	unsafe{NOUMENON.get_or_insert_with(|| noumenon::get_noumenon(Some(std::env::current_exe().unwrap().parent().unwrap().parent().unwrap()))).as_ref()}
}
#[cfg(not(feature = "plugin"))]
pub fn noumenon_instance() -> Option<&'static noumenon::Noumenon> {
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
	
	progress: crate::modman::backend::TaskProgress,
}

#[cfg(any(feature = "plugin", feature = "client"))]
impl Core {
	pub fn new(
	ui_ctx: egui::Context,
	notification: fn(f32, u8, &str),
	backend_initializers: modman::backend::BackendInitializers,
	requirement_initializers: modman::requirement::RequirementInitializers,
	optional_initializers: modman::meta::OptionalInitializers,
	services_initializers: service::ServicesInitializers) -> Self {
		ui_ctx.add_bytes_loader(std::sync::Arc::new(ui_ext::AssetLoader::default()));
		egui_extras::install_image_loaders(&ui_ctx);
		
		unsafe {
			// BACKEND = Some(std::sync::Mutex::new(modman::backend::new_backend(backend_initializers)));
			NOTIFICATION = notification;
			BACKEND = Some(modman::backend::new_backend(backend_initializers));
			modman::requirement::initialize(requirement_initializers);
			
			if let Some(dalamud) = optional_initializers.dalamud {
				modman::meta::dalamud::SETSTYLE = dalamud;
			}
			
			service::initialize(services_initializers);
		}
		
		let progress = crate::modman::backend::TaskProgress::new();
		
		let s = Self {
			views: egui_dock::DockState::new(vec![
				Box::new(view::mods::Mods::new(progress.clone())),
				Box::new(view::browser::Browser::new(progress.clone())),
				Box::new(view::settings::Settings::new()),
				Box::new(view::tool::Tools::new()),
				Box::new(view::explorer::Explorer::new()),
				Box::new(view::debug::Debug::new()),
			]),
			
			backend_last_error: matches!(backend().get_status(), modman::backend::Status::Error(_)),
			
			progress,
		};
		
		if !s.backend_last_error {
			let progress = s.progress.clone();
			std::thread::spawn(move || {
				progress.add_task_count(1);
				backend().apply_services();
				remote::check_updates(progress.clone());
				progress.progress_task();
			});
		}
		
		s
	}
	
	pub fn draw(&mut self, ui: &mut egui::Ui, renderer: &renderer::Renderer) {
		let status = backend().get_status();
		match status {
			modman::backend::Status::Ok => {
				if self.backend_last_error {
					let progress = self.progress.clone();
					std::thread::spawn(move || {
						progress.add_task_count(1);
						backend().apply_services();
						remote::check_updates(progress.clone());
						progress.progress_task();
					});
					
					// self.mods_tab.refresh();
				}
				
				self.backend_last_error = false;
			}
			
			modman::backend::Status::Error(_) => self.backend_last_error = true,
		}
		
		ui.scope(|ui| {
			ui.spacing_mut().item_spacing.y = 0.0;
			let rounding = ui.visuals().widgets.noninteractive.corner_radius;
			let top = egui::CornerRadius{ne: rounding.ne, nw: rounding.nw, ..Default::default()};
			
			if self.progress.is_busy() {
				let progress = self.progress.get_task_progress();
				ui.add(egui::ProgressBar::new(progress)
					.text(format!("{:.0}% {}", progress * 100.0, self.progress.get_task_msg()))
					.corner_radius(top));
				
				let progress = self.progress.sub_task.get();
				ui.add(egui::ProgressBar::new(progress)
					.text(format!("{:.0}% {}", progress * 100.0, self.progress.sub_task.get_msg()))
					.corner_radius(egui::CornerRadius::same(0)));
			} else {
				let messages = self.progress.get_messages();
				if messages.len() > 0 {
					for (msg, is_error) in messages {
						if is_error {
							ui.label(egui::RichText::new(msg.as_str()).color(egui::Color32::RED));
						} else {
							ui.label(msg.as_str());
						}
					}
					
					if ui.button("Ok").clicked() {
						self.progress.reset();
					}
				}
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
			.show_leaf_close_all_buttons(false)
			.show_leaf_collapse_buttons(false)
			.tab_context_menus(false)
			.show_inside(ui, &mut view::Viewer{renderer: renderer});
	}
}