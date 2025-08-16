#![allow(dead_code)]

use egui::{Response, Ui, WidgetText};
use crate::EnumTools;

// mod asset_loader;
mod loader {
	pub mod asset;
	pub mod http;
}
pub use loader::asset::*;
pub use loader::http::*;

mod splitter;
pub use splitter::*;
mod coloredit;
pub use coloredit::ColorEditValue;
mod interactable_scene;
pub use interactable_scene::InteractableScene;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
	Horizontal,
	Vertical,
	Both,
}

pub trait UiExt {
	fn texture(&mut self, img: &egui::TextureHandle, max_size: impl Into<egui::Vec2>, uv: impl Into<egui::Rect>) -> Response;
	fn num_edit<Num: egui::emath::Numeric>(&mut self, value: &mut Num, label: impl Into<WidgetText>) -> Response;
	fn num_edit_range<Num: egui::emath::Numeric>(&mut self, value: &mut Num, label: impl Into<WidgetText>, range: std::ops::RangeInclusive<Num>) -> egui::Response;
	fn num_multi_edit<Num: egui::emath::Numeric>(&mut self, values: &mut [Num], label: impl Into<WidgetText>) -> Response;
	fn num_multi_edit_range<Num: egui::emath::Numeric>(&mut self, values: &mut [Num], label: impl Into<WidgetText>, range: &[std::ops::RangeInclusive<Num>]) -> egui::Response;
	fn combo<S: Into<WidgetText>, S2: Into<WidgetText>>(&mut self, preview: S2, label: S, contents: impl FnOnce(&mut Ui));
	fn combo_id<S: Into<WidgetText>>(&mut self, preview: S, id: impl std::hash::Hash, contents: impl FnOnce(&mut Ui));
	fn combo_enum<S: Into<WidgetText>, Enum: EnumTools + PartialEq>(&mut self, val: &mut Enum, label: S) -> Response;
	fn combo_enum_id<Enum: EnumTools + PartialEq>(&mut self, val: &mut Enum, id: impl std::hash::Hash) -> Response;
	fn helptext<S: Into<WidgetText>>(&mut self, text: S);
	fn slider<S: Into<WidgetText>, N: egui::emath::Numeric>(&mut self, value: &mut N, range: std::ops::RangeInclusive<N>, label: S) -> Response;
	fn get_clipboard(&mut self) -> String;
	fn set_clipboard<S: Into<String>>(&mut self, text: S);
	fn userspace_loaders(&mut self, contents: impl FnOnce(&mut Ui));
	fn free_textures(&mut self, prefix: &str);
	fn filled_reserved_vertical(&mut self, id: impl std::hash::Hash, contents: impl FnOnce(&mut Ui, &mut Ui));
	fn filled_reserved_horizontal(&mut self, id: impl std::hash::Hash, contents: impl FnOnce(&mut Ui, &mut Ui));
	fn centered(&mut self, id: impl std::hash::Hash, axis: Axis, content: impl FnOnce(&mut Ui));
	fn splitter(&mut self, id: impl std::hash::Hash, axis: SplitterAxis, default_pos: f32, contents: impl FnOnce(&mut Ui, &mut Ui));
	fn color_edit(&mut self, color: &mut impl ColorEditValue) -> Response;
	fn spacer(&mut self);
}

impl UiExt for Ui {
	fn texture(&mut self, img: &egui::TextureHandle, max_size: impl Into<egui::Vec2>, uv: impl Into<egui::Rect>) -> egui::Response {
		let max_size = max_size.into();
		let uv: egui::Rect = uv.into();
		let size = img.size_vec2();
		let width = size.x * (uv.max.x - uv.min.x);
		let height = size.y * (uv.max.y - uv.min.y);
		let scale = (max_size.x / width).min(max_size.y / height);
		self.add(egui::Image::new(img).uv(uv).fit_to_exact_size(egui::vec2(width * scale, height * scale)))
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
	
	fn combo<S: Into<WidgetText>, S2: Into<WidgetText>>(&mut self, preview: S2, label: S, contents: impl FnOnce(&mut Ui)) {
		egui::ComboBox::from_label(label)
			.height(300.0)
			.selected_text(preview)
			.show_ui(self, contents);
	}
	
	fn combo_id<S: Into<WidgetText>>(&mut self, preview: S, id: impl std::hash::Hash, contents: impl FnOnce(&mut Ui)) {
		egui::ComboBox::from_id_salt(id)
			.height(300.0)
			.selected_text(preview)
			.show_ui(self, contents);
	}
	
	fn combo_enum<S: Into<WidgetText>, Enum: EnumTools + PartialEq>(&mut self, val: &mut Enum, label: S) -> Response {
		egui::ComboBox::from_label(label)
			.height(300.0)
			.selected_text(val.to_str())
			.show_ui(self, |ui| {
				for item in Enum::iter() {
					let name = item.to_str();
					ui.selectable_value(val, item, name);
				}
			}).response
	}
	
	fn combo_enum_id<Enum: EnumTools + PartialEq>(&mut self, val: &mut Enum, id: impl std::hash::Hash) -> Response {
		egui::ComboBox::from_id_salt(id)
			.height(300.0)
			.selected_text(val.to_str())
			.show_ui(self, |ui| {
				for item in Enum::iter() {
					let name = item.to_str();
					ui.selectable_value(val, item, name);
				}
			}).response
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
	
	fn userspace_loaders(&mut self, contents: impl FnOnce(&mut Ui)) {
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
	
	fn filled_reserved_vertical(&mut self, id: impl std::hash::Hash, contents: impl FnOnce(&mut Ui, &mut Ui)) {
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
	
	fn filled_reserved_horizontal(&mut self, id: impl std::hash::Hash, contents: impl FnOnce(&mut Ui, &mut Ui)) {
		let id = self.id().with(id);
		let reserved_size = self.data(|v| v.get_temp(id).unwrap_or(0.0));
		
		let layout = egui::Layout::top_down(egui::Align::Min);
		let mut rect = self.available_rect_before_wrap();
		rect.max.x -= reserved_size;
		
		self.horizontal(|ui| {
			ui.allocate_rect(rect, egui::Sense::hover());
			
			let mut ui_filled = ui.new_child(egui::UiBuilder::new().layout(layout).max_rect(rect));
			ui_filled.set_clip_rect(rect);
			
			let mut ui_reserved = ui.new_child(egui::UiBuilder::new().layout(layout));
			
			contents(&mut ui_filled, &mut ui_reserved);
			
			let reserved_size = ui_reserved.min_rect().width();
			ui.data_mut(|v| v.insert_temp(id, reserved_size));
		});
	}
	
	fn centered(&mut self, id: impl std::hash::Hash, axis: Axis, content: impl FnOnce(&mut Ui)) {
		let id = self.id().with(id);
		let available = self.available_size();
		let size = self.data(|v| v.get_temp(id).unwrap_or(available));
		let offset = match axis {
			Axis::Horizontal => egui::vec2((available.x - size.x) / 2.0, 0.0),
			Axis::Vertical => egui::vec2(0.0, (available.y - size.y) / 2.0),
			Axis::Both => (available - size) / 2.0,
		};
		let mut ui = self.new_child(egui::UiBuilder::new()
			.max_rect(egui::Rect::from_min_size(self.next_widget_position() + offset, size)));
		
		content(&mut ui);
		
		let rect = ui.min_rect();
		self.allocate_rect(rect, egui::Sense::empty());
		self.data_mut(|v| v.insert_temp(id, rect.size()));
	}
	
	fn splitter(&mut self, id: impl std::hash::Hash, axis: splitter::SplitterAxis, default_pos: f32, contents: impl FnOnce(&mut Ui, &mut Ui)) {
		Splitter::new(self.id().with(id), axis)
			.default_pos(default_pos)
			.show(self, contents);
	}
	
	fn color_edit(&mut self, color: &mut impl ColorEditValue) -> Response {
		coloredit::color_edit(self, color)
	}
	
	fn spacer(&mut self) {
		self.add_space(8.0);
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