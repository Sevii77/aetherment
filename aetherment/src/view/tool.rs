pub mod overlay;

pub struct Tools {
	current_tab: String,
	
	overlay_tab: overlay::Overlay,
}

impl Tools {
	pub fn new() -> Self {
		Self {
			current_tab: "Overlay Creator".to_string(),
			
			overlay_tab: overlay::Overlay::new(),
		}
	}
	
	pub fn draw(&mut self, ui: &mut renderer::Ui) {
		ui.tabs(&["Overlay Creator"], &mut self.current_tab);
		
		match self.current_tab.as_str() {
			"Overlay Creator" => {
				self.overlay_tab.draw(ui);
			},
			
			_ => {
				ui.label("Invalid tab");
			}
		}
	}
}