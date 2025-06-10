#![allow(dead_code)]

use egui::{Response, WidgetText};
use crate::EnumTools;

mod asset_loader;
pub use asset_loader::*;
mod splitter;
pub use splitter::*;

pub trait UiExt {
	fn texture(&mut self, img: egui::TextureHandle, max_size: impl Into<egui::Vec2>, uv: impl Into<egui::Rect>) -> egui::Response;
	fn num_edit<Num: egui::emath::Numeric>(&mut self, value: &mut Num, label: impl Into<egui::WidgetText>) -> egui::Response;
	fn num_edit_range<Num: egui::emath::Numeric>(&mut self, value: &mut Num, label: impl Into<egui::WidgetText>, range: std::ops::RangeInclusive<Num>) -> egui::Response;
	fn num_multi_edit<Num: egui::emath::Numeric>(&mut self, values: &mut [Num], label: impl Into<egui::WidgetText>) -> egui::Response;
	fn num_multi_edit_range<Num: egui::emath::Numeric>(&mut self, values: &mut [Num], label: impl Into<egui::WidgetText>, range: &[std::ops::RangeInclusive<Num>]) -> egui::Response;
	fn combo<S: Into<WidgetText>, S2: Into<WidgetText>>(&mut self, preview: S2, label: S, contents: impl FnOnce(&mut egui::Ui));
	fn combo_enum<S: Into<WidgetText>, Enum: EnumTools + PartialEq>(&mut self, val: &mut Enum, label: S);
	fn combo_enum_id<Enum: EnumTools + PartialEq>(&mut self, val: &mut Enum, id: impl std::hash::Hash);
	fn helptext<S: Into<WidgetText>>(&mut self, text: S);
	fn slider<S: Into<WidgetText>, N: egui::emath::Numeric>(&mut self, value: &mut N, range: std::ops::RangeInclusive<N>, label: S) -> Response;
	fn get_clipboard(&mut self) -> String;
	fn set_clipboard<S: Into<String>>(&mut self, text: S);
	fn userspace_loaders(&mut self, contents: impl FnOnce(&mut egui::Ui));
	fn free_textures(&mut self, prefix: &str);
	fn filled_reserved_vertical(&mut self, id: impl std::hash::Hash, contents: impl FnOnce(&mut egui::Ui, &mut egui::Ui));
	fn filled_reserved_horizontal(&mut self, id: impl std::hash::Hash, contents: impl FnOnce(&mut egui::Ui, &mut egui::Ui));
}

impl UiExt for egui::Ui {
	fn texture(&mut self, img: egui::TextureHandle, max_size: impl Into<egui::Vec2>, uv: impl Into<egui::Rect>) -> egui::Response {
		let max_size = max_size.into();
		let uv: egui::Rect = uv.into();
		let size = img.size_vec2();
		let width = size.x * (uv.max.x - uv.min.x);
		let height = size.y * (uv.max.y - uv.min.y);
		let scale = (max_size.x / width).min(max_size.y / height);
		// self.add(egui::Image::new(img.id(), egui::vec2(width * scale, height * scale)).uv(uv))
		self.add(egui::Image::new(&img).uv(uv).max_size(egui::vec2(width * scale, height * scale)))
	}
	
	fn num_edit<Num: egui::emath::Numeric>(&mut self, value: &mut Num, label: impl Into<egui::WidgetText>) -> egui::Response {
		self.horizontal(|ui| {
			let resp = ui.add(create_drag(value));
			ui.label(label.into());
			resp
		}).inner
	}
	
	fn num_edit_range<Num: egui::emath::Numeric>(&mut self, value: &mut Num, label: impl Into<egui::WidgetText>, range: std::ops::RangeInclusive<Num>) -> egui::Response {
		self.horizontal(|ui| {
			let resp = ui.add(create_drag(value).range(range));
			ui.label(label.into());
			resp
		}).inner
	}
	
	fn num_multi_edit<Num: egui::emath::Numeric>(&mut self, values: &mut [Num], label: impl Into<egui::WidgetText>) -> egui::Response {
		self.horizontal(|ui| {
			let mut resp = ui.add(create_drag(&mut values[0]));
			for value in values.iter_mut().skip(1) {
				resp |= ui.add(create_drag(value));
			}
			ui.label(label.into());
			resp
		}).inner
	}
	
	fn num_multi_edit_range<Num: egui::emath::Numeric>(&mut self, values: &mut [Num], label: impl Into<egui::WidgetText>, range: &[std::ops::RangeInclusive<Num>]) -> egui::Response {
		self.horizontal(|ui| {
			let mut resp = ui.add(create_drag(&mut values[0]).range(range[0].clone()));
			for (i, value) in values.iter_mut().skip(1).enumerate() {
				resp |= ui.add(create_drag(value).range(range[i].clone()));
			}
			ui.label(label.into());
			resp
		}).inner
	}
	
	fn combo<S: Into<WidgetText>, S2: Into<WidgetText>>(&mut self, preview: S2, label: S, contents: impl FnOnce(&mut egui::Ui)) {
		egui::ComboBox::from_label(label)
			.height(300.0)
			.selected_text(preview)
			.show_ui(self, contents);
	}
	
	fn combo_enum<S: Into<WidgetText>, Enum: EnumTools + PartialEq>(&mut self, val: &mut Enum, label: S) {
		egui::ComboBox::from_label(label)
			.height(300.0)
			.selected_text(val.to_str())
			.show_ui(self, |ui| {
				for item in Enum::iter() {
					let name = item.to_str();
					ui.selectable_value(val, item, name);
				}
			});
	}
	
	fn combo_enum_id<Enum: EnumTools + PartialEq>(&mut self, val: &mut Enum, id: impl std::hash::Hash) {
		egui::ComboBox::from_id_salt(id)
			.height(300.0)
			.selected_text(val.to_str())
			.show_ui(self, |ui| {
				for item in Enum::iter() {
					let name = item.to_str();
					ui.selectable_value(val, item, name);
				}
			});
	}
	
	fn helptext<S: Into<WidgetText>>(&mut self, text: S) {
		self.label("(❓)")
			.on_hover_cursor(egui::CursorIcon::Help)
			.on_hover_text(text);
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
	
	fn userspace_loaders(&mut self, contents: impl FnOnce(&mut egui::Ui)) {
		let loaders = self.ctx().loaders();
		
		let byte = loaders.bytes.lock().clone();
		loaders.bytes.lock().retain(|v| v.id() == "aetherment::AssetLoader");
		
		contents(self);
		
		*loaders.bytes.lock() = byte;
	}
	
	fn free_textures(&mut self, prefix: &str) {
		let ctx = self.ctx();
		let man = ctx.tex_manager();
		let man = man.read();
		
		let mut free = Vec::new();
		for (_, meta) in man.allocated() {
			if meta.name.starts_with(prefix) {
				free.push(meta.name.clone());
			}
		}
		
		drop(man);
		
		for uri in free {
			ctx.forget_image(&uri);
		}
	}
	
	fn filled_reserved_vertical(&mut self, id: impl std::hash::Hash, contents: impl FnOnce(&mut egui::Ui, &mut egui::Ui)) {
		let id = self.id().with(id);
		let reserved_size = self.data(|v| v.get_temp(id).unwrap_or(0.0));
		
		let mut rect = self.available_rect_before_wrap();
		rect.max.y -= reserved_size;
		self.allocate_rect(rect, egui::Sense::hover());
		
		let mut ui_filled = self.new_child(egui::UiBuilder::new().max_rect(rect));
		ui_filled.set_clip_rect(rect);
		
		let mut ui_reserved = self.new_child(egui::UiBuilder::new());
		
		contents(&mut ui_filled, &mut ui_reserved);
		
		let reserved_size = ui_reserved.min_rect().height();
		self.data_mut(|v| v.insert_temp(id, reserved_size));
	}
	
	fn filled_reserved_horizontal(&mut self, id: impl std::hash::Hash, contents: impl FnOnce(&mut egui::Ui, &mut egui::Ui)) {
		let id = self.id().with(id);
		let reserved_size = self.data(|v| v.get_temp(id).unwrap_or(0.0));
		
		let mut rect = self.available_rect_before_wrap();
		rect.max.x -= reserved_size;
		self.allocate_rect(rect, egui::Sense::hover());
		
		let mut ui_filled = self.new_child(egui::UiBuilder::new().max_rect(rect));
		ui_filled.set_clip_rect(rect);
		
		let mut ui_reserved = self.new_child(egui::UiBuilder::new());
		
		contents(&mut ui_filled, &mut ui_reserved);
		
		let reserved_size = ui_reserved.min_rect().width();
		self.data_mut(|v| v.insert_temp(id, reserved_size));
	}
}

fn create_drag<Num: egui::emath::Numeric>(value: &mut Num) -> egui::DragValue {
	if Num::INTEGRAL {
		egui::DragValue::new(value)
	} else {
		egui::DragValue::new(value)
			.max_decimals(3)
			.speed(0.01)
	}
}