pub struct ErrorView {
	error: crate::resource_loader::BacktraceError
}

impl ErrorView {
	pub fn new(error: crate::resource_loader::BacktraceError) -> Self {
		Self {
			error,
		}
	}
}

impl super::ResourceView for ErrorView {
	fn title(&self) -> String {
		"Error".to_string()
	}
	
	fn has_changes(&self) -> bool {
		false
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, _renderer: &renderer::Renderer) {
		ui.label(egui::RichText::new(format!("{:#?}", self.error)).color(egui::Color32::RED));
	}
}