pub struct Tree {
	
}

impl Tree {
	pub fn new() -> Self {
		Self {
			
		}
	}
}

impl super::ExplorerView for Tree {
	fn title(&self) -> String {
		"Tree".to_string()
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, _renderer: &crate::Renderer) {
		ui.label("Tree here");
	}
}