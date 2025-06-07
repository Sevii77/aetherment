pub mod mods;
pub mod browser;
pub mod settings;
// pub mod tool;
pub mod debug;

pub trait View {
	fn name(&self) -> &'static str;
	fn render(&mut self, ui: &mut egui::Ui);
}

pub struct Viewer;
impl egui_dock::TabViewer for Viewer {
	type Tab = Box<dyn View>;
	
	fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
		tab.render(ui);
	}
	
	fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
		tab.name().into()
	}
}