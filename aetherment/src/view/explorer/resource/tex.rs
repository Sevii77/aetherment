use noumenon::format::{external::Bytes, game::Tex};
use crate::ui_ext::UiExt;

pub struct TexView {
	tex: Tex,
	img: Option<egui::TextureHandle>,
	depth: u32,
	mip: u32,
}

impl TexView {
	pub fn new(path: &super::Path) -> Result<Self, crate::resource_loader::BacktraceError> {
		let data = super::read_file(path)?;
		
		Ok(Self {
			tex: Tex::read(&mut std::io::Cursor::new(&data))?,
			img: None,
			depth: 0,
			mip: 0,
		})
	}
	
	fn load_image(&mut self, ctx: &egui::Context) {
		let slice = self.tex.slice(self.depth, self.mip);
		let data = egui::ColorImage {
			size: [slice.width as usize, slice.height as usize],
			pixels: slice.pixels.chunks_exact(4).map(|v| egui::Color32::from_rgba_unmultiplied(v[0], v[1], v[2], v[3])).collect(),
		};
		
		if let Some(img) = self.img.as_mut() {
			img.set(data, egui::TextureOptions::NEAREST);
		} else {
			self.img = Some(ctx.load_texture("explorer::resource::tex", data, egui::TextureOptions::NEAREST));
		}
	}
}

impl super::ResourceView for TexView {
	fn title(&self) -> String {
		"Texture".to_string()
	}
	
	fn has_changes(&self) -> bool {
		false
	}
	
	fn ui(&mut self, ui: &mut egui::Ui) {
		if self.img.is_none() {
			self.load_image(ui.ctx());
		}
		
		ui.splitter("splitter", crate::ui_ext::SplitterAxis::Horizontal, 0.8, |ui_left, ui_right| {
			let ui = ui_left;
			if let Some(img) = &self.img {
				let max_size = ui.available_size();
				let size = img.size_vec2();
				let scale = (max_size.x / size.x).min(max_size.y / size.y);
				let size = egui::vec2(size.x * scale, size.y * scale);
				let next = ui.next_widget_position();
				let offset = egui::pos2(next.x + (max_size.x - size.x) / 2.0, next.y + (max_size.y - size.y) / 2.0); 
				let rect = egui::Rect{min: offset, max: offset + size};
				
				ui.set_clip_rect(rect);
				let draw = ui.painter();
				draw.rect_filled(rect, egui::CornerRadius::ZERO, egui::Color32::GRAY);
				for y in 0..size.y as usize / 32 + 1 {
					for x in (0..size.x as usize / 32 + 1).step_by(2) {
						let offset = offset + egui::vec2(if y % 2 == 0 {x * 32} else {x * 32 + 32} as f32, y as f32 * 32.0);
						draw.rect_filled(egui::Rect{min: offset, max: offset + egui::vec2(32.0, 32.0)}, egui::CornerRadius::ZERO, egui::Color32::DARK_GRAY);
					}
				}
				
				ui.put(rect, egui::Image::new(img).fit_to_exact_size(size));
			}
			
			let ui = ui_right;
			ui.label(format!("format: {:?}", self.tex.format));
			ui.label(format!("dimensions: {}x{}x{}", self.tex.width, self.tex.height, self.tex.depth));
			
			let max_depth = self.tex.depth / 2u32.pow(self.mip as u32);
			let mut changed = ui.slider(&mut self.depth, 0..=max_depth.max(1) - 1, "Depth").changed();
			changed |= ui.slider(&mut self.mip, 0..=self.tex.mip_levels - 1, "Mip").changed();
			if changed {
				self.depth = self.depth.min(max_depth);
				self.load_image(ui.ctx());
			}
		});
	}
}