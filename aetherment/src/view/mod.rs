pub mod mods;
pub mod browser;
pub mod settings;
pub mod tool;
pub mod explorer;
pub mod debug;

pub trait View {
	fn title(&self) -> &'static str;
	fn ui(&mut self, ui: &mut egui::Ui, renderer: &crate::Renderer);
}

pub struct Viewer<'r> {
	pub renderer: &'r crate::Renderer,
}

impl<'r> egui_dock::TabViewer for Viewer<'r> {
	type Tab = Box<dyn View>;
	
	fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
		tab.title().into()
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
		tab.ui(ui, self.renderer);
	}
}