pub struct Error {
	error: crate::resource_loader::BacktraceError
}

impl Error {
	pub fn new(error: crate::resource_loader::BacktraceError) -> Self {
		Self {
			error,
		}
	}
}

impl super::ResourceView for Error {
	fn title(&self) -> String {
		"Error".to_string()
	}
	
	fn ui(&mut self, ui: &mut egui::Ui) {
		ui.label(egui::RichText::new(format!("{:#?}", self.error)).color(egui::Color32::RED));
	}
}