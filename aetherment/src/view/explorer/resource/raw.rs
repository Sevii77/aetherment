use std::rc::Rc;

pub struct RawView {
	data: Vec<u8>,
	text: Option<Rc<str>>,
	len: usize,
	scroll: usize,
}

impl RawView {
	pub fn new(path: &super::Path) -> Result<Self, crate::resource_loader::BacktraceError> {
		let data = super::read_file(path)?;
		let mut text = None;
		let mut len = data.len();
		if !data.contains(&0) {
			if let Ok(s) = str::from_utf8(&data) {
				text = Some(s.into());
				len = s.chars().filter(|v| *v == '\n').count();
			}
		}
		
		Ok(Self {
			data,
			text,
			len,
			scroll: 0,
		})
	}
	
	fn ui_text(&mut self, ui: &mut egui::Ui) {
		egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
			let Some(text) = &self.text else {return};
			
			ui.monospace(text.to_string());
		});
	}
	
	fn ui_binary(&mut self, ui: &mut egui::Ui) {
		ui.monospace("          00 01 02 03  04 05 06 07  08 09 0A 0B  0C 0D 0E 0F");
		
		let mut cui = ui.new_child(egui::UiBuilder::new()
			.ui_stack_info(egui::UiStackInfo::new(egui::UiKind::ScrollArea)));
		
		for i in self.scroll..self.len / 16 + 1 {
			let row = &self.data[i * 16..self.len.min(i * 16 + 16)];
			
			let mut line = format!("{i:08x}");
			let mut text = "  ".to_string();
			
			for j in 0..16 {
				if j % 4 == 0 {
					line.push(' ');
				}
				
				if j + i * 16 < self.len {
					let v = row[j];
					line.push_str(&format!(" {:02x}", v));
					text.push(if v >= 32 && v <= 126 {char::from_u32(v as u32).unwrap()} else {'.'})
				} else {
					line.push_str("   ");
				}
			}
			
			line.push_str(&text);
			
			let r = cui.monospace(line);
			if !cui.is_rect_visible(r.rect) {
				break;
			}
		}
		
		let scroll = cui.ctx().input(|v| v.raw_scroll_delta.y);
		self.scroll = (self.scroll as f32 - (scroll / 10.0)).clamp(0.0, (self.len / 16) as f32) as usize;
	}
}

impl super::ResourceView for RawView {
	fn title(&self) -> String {
		if self.text.is_some() {
			"Text".to_string()
		} else {
			"Binary".to_string()
		}
	}
	
	fn has_changes(&self) -> bool {
		false
	}
	
	fn ui(&mut self, ui: &mut egui::Ui) {
		if self.text.is_some() {
			self.ui_text(ui);
		} else {
			self.ui_binary(ui);
		}
	}
}