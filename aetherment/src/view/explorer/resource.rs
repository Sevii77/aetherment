use egui::Widget;
use crate::ui_ext::UiExt;

mod error;
mod raw;
mod tex;
mod mdl;
mod mtrl;
mod sklb;

pub enum Path {
	Game(String),
	Real(std::path::PathBuf),
}

impl Path {
	pub fn ext(&self) -> String {
		match self {
			Path::Game(v) => v.split(".").last().unwrap().to_string(),
			Path::Real(v) => v.to_string_lossy().split(".").last().unwrap().to_string(),
		}
	}
}

pub fn read_file(path: &Path) -> Result<Vec<u8>, crate::resource_loader::BacktraceError> {
	match path {
		Path::Game(path) =>
			Ok(crate::noumenon()
				.ok_or("No Noumenon instance")?
				.file::<Vec<u8>>(&path)?),
		
		Path::Real(path) =>
			Ok(std::fs::read(path)?),
	}
}

// ----------

pub trait ResourceView {
	fn title(&self) -> String;
	fn has_changes(&self) -> bool;
	fn ui(&mut self, ui: &mut egui::Ui, renderer: &renderer::Renderer);
}

// ----------

pub struct Resource {
	path: String,
	resource: Box<dyn ResourceView>,
	changed: bool,
}

impl Resource {
	pub fn new(path: &str) -> Self {
		Self {
			path: path.to_string(),
			resource: load_resource_view(path),
			changed: false,
		}
	}
}

impl super::ExplorerView for Resource {
	fn title(&self) -> String {
		format!("{} - {}{}",
			self.resource.title(),
			self.path.split("/").last().unwrap(),
			if self.resource.has_changes() {" *"} else {""}
		)
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, renderer: &renderer::Renderer) {
		ui.filled_reserved_vertical("path_footer", |ui_filled, ui_reserved| {
			self.resource.ui(ui_filled, renderer);
			
			let resp = egui::TextEdit::singleline(&mut self.path)
				.desired_width(f32::INFINITY)
				.ui(ui_reserved);
			
			if resp.changed() {
				self.changed = true;
			}
			
			if resp.lost_focus() && self.changed {
				self.resource = load_resource_view(&self.path);
				self.changed = false;
			}
		});
	}
}

fn load_resource_view(path: &str) -> Box<dyn ResourceView> {
	#[cfg(target_family = "windows")]
	let path = if path.len() >= 3 && &path[1..=2] == ":/" {
		Path::Real(path.into())
	} else {
		Path::Game(path.to_string())
	};
	
	#[cfg(not(target_family = "windows"))]
	let path = if path.len() >= 1 && &path[0..=0] == "/" {
		Path::Real(path.into())
	} else {
		Path::Game(path.to_string())
	};
	
	match path.ext().as_str() {
		"tex" | "atex" => tex::TexView::new(&path).map_or_else::<Box<dyn ResourceView> , _, _>(|err| Box::new(error::ErrorView::new(err)), |v| Box::new(v)),
		"mdl" => mdl::MdlView::new(&path).map_or_else::<Box<dyn ResourceView> , _, _>(|err| Box::new(error::ErrorView::new(err)), |v| Box::new(v)),
		"mtrl" => mtrl::MtrlView::new(&path).map_or_else::<Box<dyn ResourceView> , _, _>(|err| Box::new(error::ErrorView::new(err)), |v| Box::new(v)),
		"sklb" => sklb::SklbView::new(&path).map_or_else::<Box<dyn ResourceView> , _, _>(|err| Box::new(error::ErrorView::new(err)), |v| Box::new(v)),
		_ => raw::RawView::new(&path).map_or_else::<Box<dyn ResourceView> , _, _>(|err| Box::new(error::ErrorView::new(err)), |v| Box::new(v)),
	}
}