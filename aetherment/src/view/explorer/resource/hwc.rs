use noumenon::format::{external::Bytes, game::Hwc};

pub struct HwcView {
	hwc: Hwc,
	img: Option<egui::TextureHandle>,
}

impl HwcView {
	pub fn new(path: &super::Path) -> Result<Self, crate::resource_loader::BacktraceError> {
		let data = super::read_file(path)?;
		
		Ok(Self {
			hwc: Hwc::read(&mut std::io::Cursor::new(&data))?,
			img: None,
		})
	}
}

impl super::ResourceView for HwcView {
	fn title(&self) -> String {
		"Hardware Cursor".to_string()
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, _renderer: &renderer::Renderer) -> crate::view::explorer::Action {
		let img = self.img.get_or_insert_with(|| {
			let data = egui::ColorImage {
				size: [64; 2],
				pixels: self.hwc.pixels.chunks_exact(4).map(|v| egui::Color32::from_rgba_unmultiplied(v[0], v[1], v[2], v[3])).collect(),
			};
			
			ui.ctx().load_texture("explorer::resource::hwc", data, egui::TextureOptions::NEAREST)
		});
		
		super::tex::preview(ui, img, ui.available_size(), false, 32);
		
		crate::view::explorer::Action::None
	}
	
	fn export(&self) -> super::Export {
		super::Export::Converter(noumenon::Convert::Hwc(self.hwc.clone()))
	}
}