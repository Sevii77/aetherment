pub mod tattoo;

pub struct Tools {
	views: egui_dock::DockState<Box<dyn super::View>>,
}

impl Tools {
	pub fn new() -> Self {
		Self {
			views: egui_dock::DockState::new(vec![
				Box::new(tattoo::Tattoo::new()),
			]),
		}
	}
}

impl super::View for Tools {
	fn title(&self) -> &'static str {
		"Tools"
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, renderer: &renderer::Renderer) {
		egui_dock::DockArea::new(&mut self.views)
			.id(egui::Id::new("tool_tabs"))
			.style(egui_dock::Style::from_egui(ui.style().as_ref()))
			.draggable_tabs(false)
			.show_close_buttons(false)
			.show_leaf_close_all_buttons(false)
			.show_leaf_collapse_buttons(false)
			.tab_context_menus(false)
			.show_inside(ui, &mut super::Viewer{renderer});
	}
}