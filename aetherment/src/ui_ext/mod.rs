use egui::{Response, WidgetText};

#[cfg(any(feature = "plugin", feature = "client"))]
mod splitter;

pub use splitter::*;

pub trait EnumTools {
	type Iterator: core::iter::Iterator<Item = Self>;
	
	fn to_str(&self) -> &'static str;
	fn to_string(&self) -> String {self.to_str().to_string()}
	fn iter() -> Self::Iterator;
}

pub trait UiExt {
	fn combo<S: Into<WidgetText>, S2: Into<WidgetText>>(&mut self, preview: S2, label: S, contents: impl FnOnce(&mut egui::Ui));
	fn combo_enum<S: Into<WidgetText>, Enum: EnumTools + PartialEq>(&mut self, val: &mut Enum, label: S) -> Response;
	fn helptext<S: Into<WidgetText>>(&mut self, text: S);
	fn slider<S: Into<WidgetText>, N: egui::emath::Numeric>(&mut self, value: &mut N, range: std::ops::RangeInclusive<N>, label: S) -> Response;
	fn get_clipboard(&mut self) -> String;
	fn set_clipboard<S: Into<String>>(&mut self, text: S);
}

#[cfg(any(feature = "plugin", feature = "client"))]
impl UiExt for egui::Ui {
	fn combo<S: Into<WidgetText>, S2: Into<WidgetText>>(&mut self, preview: S2, label: S, contents: impl FnOnce(&mut egui::Ui)) {
		egui::ComboBox::from_label(label)
			.height(300.0)
			.selected_text(preview)
			.show_ui(self, contents);
	}
	
	fn combo_enum<S: Into<WidgetText>, Enum: EnumTools + PartialEq>(&mut self, val: &mut Enum, label: S) -> Response {
		let resp = egui::ComboBox::from_label(label)
			.height(300.0)
			.selected_text(val.to_str())
			.show_ui(self, |ui| {
				let mut resp = None::<Response>;
				for item in Enum::iter() {
					let name = item.to_str();
					let r = ui.selectable_value(val, item, name);
					if let Some(r2) = resp.as_mut()  {
						resp = Some(r2.union(r));
					} else {
						resp = Some(r);
					}
				}
				
				resp
			});
		
		resp.inner.unwrap().unwrap_or_else(|| resp.response)
	}
	
	fn helptext<S: Into<WidgetText>>(&mut self, text: S) {
		self.label("❓").on_hover_text(text);
	}
	
	fn slider<S: Into<WidgetText>, N: egui::emath::Numeric>(&mut self, value: &mut N, range: std::ops::RangeInclusive<N>, label: S) -> Response {
		self.add(egui::Slider::new(value, range).text(label))
	}
	
	fn get_clipboard(&mut self) -> String {
		use clipboard::ClipboardProvider;
		let Ok(mut clip) = clipboard::ClipboardContext::new() else {return String::new()};
		clip.get_contents().unwrap_or_default()
	}
	
	fn set_clipboard<S: Into<String>>(&mut self, text: S) {
		self.ctx().copy_text(text.into());
	}
}