use egui::Widget;
use crate::{modman::meta, ui_ext::UiExt};

mod error;
mod raw;
pub(crate) mod tex;
mod mdl;
mod mtrl;
mod sklb;

#[derive(Debug, Clone)]
pub enum Path {
	Game(String),
	Real(std::path::PathBuf),
	Mod {
		mod_meta: std::rc::Rc<std::cell::RefCell<meta::Meta>>,
		mod_root: std::path::PathBuf,
		game_path: String,
		real_path: Option<String>,
		option: Option<(String, String)>,
	}
}

impl Path {
	pub fn from_str(path: impl Into<String>) -> Self {
		let path = path.into();
		
		#[cfg(target_family = "windows")]
		if path.len() >= 3 && &path[1..=2] == ":/" {
			Path::Real(path.into())
		} else {
			Path::Game(path)
		}
		
		#[cfg(not(target_family = "windows"))]
		if path.len() >= 1 && &path[0..=0] == "/" {
			Path::Real(path.into())
		} else {
			Path::Game(path)
		}
	}
	
	pub fn from_mod_new(mod_root: impl Into<std::path::PathBuf>, game_path: impl Into<String>, option: Option<(String, String)>) -> Self {
		let mod_root = mod_root.into();
		let meta = crate::resource_loader::read_json::<meta::Meta>(&mod_root.join("meta.json")).unwrap();
		Self::from_mod(std::rc::Rc::new(std::cell::RefCell::new(meta)), mod_root, game_path, option)
	}
	
	pub fn from_mod(mod_meta: std::rc::Rc<std::cell::RefCell<meta::Meta>>, mod_root: impl Into<std::path::PathBuf>, game_path: impl Into<String>, option: Option<(String, String)>) -> Self {
		let mut s = Self::Mod {
			mod_meta,
			mod_root: mod_root.into(),
			game_path: game_path.into(),
			real_path: None,
			option,
		};
		
		s.calculate_real_path();
		s
	}
	
	pub fn calculate_real_path(&mut self) {
		let Path::Mod{mod_meta, game_path, real_path, option, ..} = self else {return};
		
		*real_path = None;
		match option.as_ref() {
			Some((option_name, suboption_name)) => {
				for opt in mod_meta.borrow().options.iter() {
					let meta::OptionType::Option(opt) = opt else {continue};
					if opt.name != *option_name {continue};
					let (meta::OptionSettings::MultiFiles(sub) | meta::OptionSettings::SingleFiles(sub)) = &opt.settings else {continue};
					for sub in &sub.options {
						if sub.name != *suboption_name {continue};
						let Some(v) = sub.files.get(game_path) else {continue};
						*real_path = Some(v.to_string());
					}
				}
			}
			
			None => {
				if let Some(v) = mod_meta.borrow().files.get(game_path) {
					*real_path = Some(v.to_string());
				}
			}
		}
	}
	
	pub fn ext(&self) -> String {
		match self {
			Path::Game(v) => v.split(".").last().unwrap().to_string(),
			Path::Real(v) => v.to_string_lossy().split(".").last().unwrap().to_string(),
			Path::Mod{game_path, ..} => game_path.trim_end_matches(".comp").split(".").last().unwrap().to_string(),
		}
	}
	
	pub fn as_path(&self) -> &str {
		match self {
			Path::Game(game_path) => game_path,
			Path::Real(game_path) => game_path.to_str().unwrap(),
			Path::Mod{game_path, ..} => game_path,
		}
	}
	
	pub fn is_composite(&self) -> bool {
		match self {
			Path::Mod{game_path, ..} => game_path.ends_with(".comp"),
			_ => false,
		}
	}
}

impl Into<Path> for &str {
	fn into(self) -> Path {
		Path::from_str(self)
	}
}

pub fn read_file(path: &Path) -> Result<Vec<u8>, crate::resource_loader::BacktraceError> {
	match path {
		Path::Game(path) =>
			Ok(crate::backend()
				.get_file(path, &crate::config().config.active_collection, i32::MAX)
				.ok_or("Failed getting file")?),
		
		Path::Real(path) =>
			Ok(std::fs::read(path)?),
		
		Path::Mod{mod_root, real_path, ..} => {
			let real_path = real_path.as_ref().ok_or("No file for current option selection")?;
			// log!("{:?}", mod_root.join("files").join(real_path));
			Ok(std::fs::read(mod_root.join("files").join(real_path))?)
		},
	}
}

// ----------

pub enum Export {
	Converter(noumenon::Convert),
	Bytes(Vec<u8>),
	Invalid,
}

pub trait ResourceView {
	fn title(&self) -> String;
	fn ui(&mut self, ui: &mut egui::Ui, renderer: &renderer::Renderer) -> super::Action;
	#[allow(unused_variables)]
	fn assign_mod(&mut self, mod_info: super::ModInfo) {}
	fn is_composite(&self) -> bool {false}
	fn export(&self) -> Export;
}

// ----------

pub struct Resource {
	// path: String,
	path: Path,
	pub(crate) resource: Box<dyn ResourceView>,
	changed_path: bool,
	pub(crate) changed_content: bool,
}

impl Resource {
	// pub fn new(path: &str) -> Self {
	pub fn new(path: impl Into<Path>) -> Self {
		let path = path.into();
		
		Self {
			resource: load_resource_view(&path),
			path,
			changed_path: false,
			changed_content: false,
		}
	}
}

impl super::ExplorerView for Resource {
	fn as_any(&self) -> &dyn std::any::Any {
		self
	}
	
	fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
		self
	}
	
	fn title(&self) -> String {
		format!("{} - {}{}",
			self.resource.title(),
			self.path.as_path().split("/").last().unwrap(),
			if self.changed_content {" *"} else {""}
		)
	}
	
	fn path(&self) -> Option<&super::resource::Path> {
		Some(&self.path)
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, renderer: &renderer::Renderer) -> super::Action {
		let mut act = super::Action::None;
		ui.filled_reserved_vertical("path_footer", |ui_filled, ui_reserved| {
			act = self.resource.ui(ui_filled, renderer);
			
			if matches!(act, super::Action::Changed) {
				self.changed_content = true;
				act = super::Action::None;
			}
			
			// TODO: this triggers on repeats (held down), fix that
			if ui_filled.interact(ui_filled.max_rect(), egui::Id::new("keybinds"), egui::Sense::hover()).hovered() {
				let ctx = ui_filled.ctx();
				if self.changed_content && ctx.input_mut(|v| v.consume_key(egui::Modifiers::CTRL, egui::Key::S)) {
					act.or(super::Action::Save);
				}
			}
			
			ui_reserved.horizontal(|ui| {
				let mut reload = false;
				if let Path::Mod{mod_meta, game_path, option, ..} = &mut self.path {
					let selected_label = option.as_ref().map_or("None".to_string(), |(a, b)| format!("{a}/{b}"));
					ui.combo_id(&selected_label, "option", |ui| {
						let exists = if mod_meta.borrow().files.contains_key(game_path) {"✔ "} else {""};
						if ui.selectable_label(option.is_none(), format!("{exists}None")).clicked() {
							*option = None;
							reload = true;
						}
						
						for opt in mod_meta.borrow().options.iter() {
							let meta::OptionType::Option(opt) = opt else {continue};
							let (meta::OptionSettings::MultiFiles(sub) | meta::OptionSettings::SingleFiles(sub)) = &opt.settings else {continue};
							for sub in &sub.options {
								let exists = if sub.files.contains_key(game_path) {"✔ "} else {""};
								let label = format!("{exists}{}/{}", opt.name, sub.name);
								if ui.selectable_label(selected_label == label, label).clicked() {
									*option = Some((opt.name.clone(), sub.name.clone()));
									reload = true;
								}
							}
						}
					});
				}
				
				if reload {
					self.path.calculate_real_path();
					self.resource = load_resource_view(&self.path);
				}
				
				let mut path_str = self.path.as_path().to_string();
				let resp = egui::TextEdit::singleline(&mut path_str)
					.desired_width(f32::INFINITY)
					.ui(ui);
				
				if resp.changed() {
					self.changed_path = true;
					
					match &mut self.path {
						Path::Mod{mod_meta, mod_root, ..} => self.path = Path::from_mod(mod_meta.clone(), &*mod_root, path_str, None),
						_ => self.path = Path::from_str(path_str),
					}
				}
				
				if resp.lost_focus() && self.changed_path {
					self.resource = load_resource_view(&self.path);
					self.changed_path = false;
				}
			});
		});
		
		act
	}
}

fn load_resource_view(path: &Path) -> Box<dyn ResourceView> {
	match path.ext().as_str() {
		"tex" | "atex" => tex::TexView::new(path).map_or_else::<Box<dyn ResourceView> , _, _>(|err| Box::new(error::ErrorView::new(err)), |v| Box::new(v)),
		"mdl" => mdl::MdlView::new(path).map_or_else::<Box<dyn ResourceView> , _, _>(|err| Box::new(error::ErrorView::new(err)), |v| Box::new(v)),
		"mtrl" => mtrl::MtrlView::new(path).map_or_else::<Box<dyn ResourceView> , _, _>(|err| Box::new(error::ErrorView::new(err)), |v| Box::new(v)),
		"sklb" => sklb::SklbView::new(path).map_or_else::<Box<dyn ResourceView> , _, _>(|err| Box::new(error::ErrorView::new(err)), |v| Box::new(v)),
		_ => raw::RawView::new(path).map_or_else::<Box<dyn ResourceView> , _, _>(|err| Box::new(error::ErrorView::new(err)), |v| Box::new(v)),
	}
}