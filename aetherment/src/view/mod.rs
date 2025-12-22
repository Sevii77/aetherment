pub mod mods;
pub mod browser;
pub mod settings;
pub mod tool;
pub mod explorer;
pub mod debug;

pub trait View {
	fn title(&self) -> &'static str;
	fn ui(&mut self, ui: &mut egui::Ui, viewer: &Viewer);
	fn tick(&mut self) {}
}

pub struct Viewer<'r> {
	pub renderer: &'r renderer::Renderer,
	pub backend_status: &'r crate::modman::backend::Status
}

impl<'r> egui_dock::TabViewer for Viewer<'r> {
	type Tab = Box<dyn View>;
	
	fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
		tab.title().into()
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
		tab.ui(ui, self);
	}
}