use std::f32::consts::TAU;
use egui::{Response, Ui};

use super::UiExt;

// we use srgb no gamma, even in 0-1 form
pub trait ColorEditValue {
	fn has_alpha(&self) -> bool;
	fn get_srgba(&self) -> [f32; 4];
	fn set_srgba(&mut self, value: [f32; 4]);
	
	// TODO: this should be hsva range, rgba range is stupid, the main purpose
	// is to not allow too dark/light or for transparency purposes
	fn get_range(&self) -> [std::ops::RangeInclusive<f32>; 4] {
		[0.0..=1.0, 0.0..=1.0, 0.0..=1.0, 0.0..=1.0]
	}
	
	fn color32(&self) -> egui::Color32 {
		let clr = self.get_srgba();
		if self.has_alpha() {
			egui::Color32::from_rgba_unmultiplied((clr[0] * 255.0) as u8, (clr[1] * 255.0) as u8, (clr[2] * 255.0) as u8, (clr[3] * 255.0) as u8)
		} else {
			egui::Color32::from_rgb((clr[0] * 255.0) as u8, (clr[1] * 255.0) as u8, (clr[2] * 255.0) as u8)
		}
	}
}

impl ColorEditValue for [f32; 3] {
	fn has_alpha(&self) -> bool {
		false
	}
	
	fn get_srgba(&self) -> [f32; 4] {
		[self[0], self[1], self[2], 1.0]
	}
	
	fn set_srgba(&mut self, value: [f32; 4]) {
		self[0] = value[0];
		self[1] = value[1];
		self[2] = value[2];
	}
}

impl ColorEditValue for [f32; 4] {
	fn has_alpha(&self) -> bool {
		true
	}
	
	fn get_srgba(&self) -> [f32; 4] {
		*self
	}
	
	fn set_srgba(&mut self, value: [f32; 4]) {
		*self = value;
	}
}

impl ColorEditValue for ([f32; 3], [std::ops::RangeInclusive<f32>; 3]) {
	fn has_alpha(&self) -> bool {
		false
	}
	
	fn get_srgba(&self) -> [f32; 4] {
		[self.0[0], self.0[1], self.0[2], 1.0]
	}
	
	fn set_srgba(&mut self, value: [f32; 4]) {
		self.0[0] = value[0];
		self.0[1] = value[1];
		self.0[2] = value[2];
	}
	
	fn get_range(&self) -> [std::ops::RangeInclusive<f32>; 4] {
		[self.1[0].clone(), self.1[1].clone(), self.1[2].clone(), 0.0..=1.0]
	}
}

impl ColorEditValue for ([f32; 4], [std::ops::RangeInclusive<f32>; 4]) {
	fn has_alpha(&self) -> bool {
		true
	}
	
	fn get_srgba(&self) -> [f32; 4] {
		self.0
	}
	
	fn set_srgba(&mut self, value: [f32; 4]) {
		self.0 = value;
	}
	
	fn get_range(&self) -> [std::ops::RangeInclusive<f32>; 4] {
		self.1.clone()
	}
}

pub fn color_edit(ui: &mut Ui, color: &mut impl ColorEditValue) -> Response {
	let resp = color_edit_button(ui, color);
	
	let mut clr = color.get_srgba();
	let range = color.get_range();
	clr[0] = clr[0].clamp(*range[0].start(), *range[0].end());
	clr[1] = clr[1].clamp(*range[1].start(), *range[1].end());
	clr[2] = clr[2].clamp(*range[2].start(), *range[2].end());
	clr[3] = clr[3].clamp(*range[3].start(), *range[3].end());
	color.set_srgba(clr);
	
	resp
}

#[derive(Clone, Copy, PartialEq)]
enum Held {
	None,
	Ring,
	Triangle,
}

// https://github.com/emilk/egui/blob/main/crates/egui/src/widgets/color_picker.rs#L491
// modified to use our custom picker
fn color_edit_button(ui: &mut Ui, color: &mut impl ColorEditValue) -> Response {
	let popup_id = ui.auto_id_with("popup");
	let open = ui.memory(|mem| mem.is_popup_open(popup_id));
	let mut button_response = color_button(ui, color.color32(), open);
	if ui.style().explanation_tooltips {
		button_response = button_response.on_hover_text("Click to edit color");
	}
	
	if button_response.clicked() {
		ui.memory_mut(|mem| mem.toggle_popup(popup_id));
	}
	
	const COLOR_SLIDER_WIDTH: f32 = 275.0;
	
	// TODO(emilk): make it easier to show a temporary popup that closes when you click outside it
	if ui.memory(|mem| mem.is_popup_open(popup_id)) {
		let area_response = egui::Area::new(popup_id)
			.kind(egui::UiKind::Picker)
			.order(egui::Order::Foreground)
			.fixed_pos(button_response.rect.max)
			.show(ui.ctx(), |ui| {
				ui.spacing_mut().slider_width = COLOR_SLIDER_WIDTH;
				egui::Frame::popup(ui.style()).show(ui, |ui| {
					if color_picker(ui, color) {
						button_response.mark_changed();
					}
				});
			})
			.response;
		
		if !button_response.clicked()
			&& (ui.input(|i| i.key_pressed(egui::Key::Escape)) || area_response.clicked_elsewhere())
		{
			ui.memory_mut(|mem| mem.close_popup());
		}
	}

	button_response
}

// https://github.com/emilk/egui/blob/main/crates/egui/src/widgets/color_picker.rs#L87
// no changes, need to copy since its private
fn color_button(ui: &mut Ui, color: egui::Color32, open: bool) -> Response {
	let size = ui.spacing().interact_size;
	let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
	response.widget_info(|| egui::WidgetInfo::new(egui::WidgetType::ColorButton));
	
	if ui.is_rect_visible(rect) {
		let visuals = if open {
			&ui.visuals().widgets.open
		} else {
			ui.style().interact(&response)
		};
		let rect = rect.expand(visuals.expansion);
		
		let stroke_width = 1.0;
		egui::widgets::color_picker::show_color_at(ui.painter(), color, rect.shrink(stroke_width));
		
		let corner_radius = visuals.corner_radius.at_most(2); // Can't do more rounding because the background grid doesn't do any rounding
		ui.painter().rect_stroke(
			rect,
			corner_radius,
			(stroke_width, visuals.bg_fill), // Using fill for stroke is intentional, because default style has no border
			egui::StrokeKind::Inside,
		);
	}
	
	response
}

fn contrast_color(color: impl Into<egui::Rgba>) -> egui::Color32 {
	if color.into().intensity() < 0.5 {
		egui::Color32::WHITE
	} else {
		egui::Color32::BLACK
	}
}

fn convert(hsva: egui::epaint::Hsva) -> egui::Color32 {
	let c = hsva.to_rgba_unmultiplied();
	egui::Color32::from_rgba_unmultiplied((c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8, 255)
}

fn color_picker(ui: &mut Ui, color: &mut impl ColorEditValue) -> bool {
	// let mut hsva = egui::epaint::Hsva::from_srgba_premultiplied(color.get_srgba_u8());
	// gamma, linear, standard, w/e, i fucking hate all this shit.
	// we work in broken washed out hsva for interaction for linear shit
	// then we convert it to gamma for when to display
	let c = color.get_srgba();
	let mut hsva = egui::epaint::Hsva::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]);
	let (mut held, hue) = ui.ctx().data(|v| v.get_temp(egui::Id::NULL)).unwrap_or((Held::None, 0.0));
	if hsva.s == 0.0 || hsva.v == 0.0 {
		hsva.h = hue;
	}
	
	let size = ui.spacing().slider_width;
	let thickness = ui.spacing().interact_size.y;
	
	let mut changed = false;
	
	{ // hue ring + triangle
		let (rect, response) = ui.allocate_at_least(egui::vec2(size, size), egui::Sense::click_and_drag());
		
		let dist = size / 2.0 - thickness * 2.0;
		let tri_black = rect.center() - egui::vec2(0.0, dist);
		let tri_clr = rect.center() - egui::vec2(-f32::sin(1.0 / 3.0 * TAU) * dist, f32::cos(1.0 / 3.0 * TAU) * dist);
		let tri_white = rect.center() - egui::vec2(-f32::sin(2.0 / 3.0 * TAU) * dist, f32::cos(2.0 / 3.0 * TAU) * dist);
		
		// interaction
		let hover_ring = response.hover_pos().map(|pos| {
			let pos = pos - rect.center();
			let dist = (pos.x * pos.x + pos.y * pos.y).sqrt();
			dist > size / 2.0 - thickness && dist < size / 2.0
		}).unwrap_or(false);
		
		let hover_tri = response.hover_pos().map(|pos| {
			let yrat = (pos.y - tri_black.y) / (tri_white.y - tri_black.y);
			pos.x > tri_black.x * (1.0 - yrat) + tri_white.x * yrat && pos.x < tri_black.x * (1.0 - yrat) + tri_clr.x * yrat &&
			pos.y > tri_black.y && pos.y < tri_white.y
		}).unwrap_or(false);
		
		if response.drag_started() {
			if hover_ring {
				held = Held::Ring;
			} else if hover_tri {
				held = Held::Triangle;
			}
		} else if response.drag_stopped() {
			held = Held::None;
		}
		
		if let Some(pos) = response.interact_pointer_pos() {
			if held == Held::Ring || (hover_ring && response.clicked()) {
				let pos = pos - rect.center();
				hsva.h = f32::atan2(pos.y, pos.x) / TAU + 0.5;
				
				color.set_srgba(hsva.to_rgba_unmultiplied());
				changed = true;
			} else if held == Held::Triangle || (hover_tri && response.clicked()) {
				hsva.v = ((pos.y - tri_black.y) / (tri_white.y - tri_black.y)).clamp(0.0, 1.0);
				hsva.s = ((pos.x - tri_black.x) / ((tri_black.x - tri_white.x) * hsva.v) / 2.0 + 0.5).clamp(0.0, 1.0);
				
				color.set_srgba(hsva.to_rgba_unmultiplied());
				changed = true;
			}
		}
		
		// ring
		let draw = ui.painter();
		let mut mesh = egui::Mesh::default();
		for i in 0..36 {
			let ratio = i as f32 / 36.0;
			let clr = convert(egui::epaint::Hsva{h: ratio, s: hsva.s, v: hsva.v, a: 1.0});
			let sin = f32::sin(ratio * TAU);
			let cos = f32::cos(ratio * TAU);
			
			mesh.colored_vertex(rect.center() - egui::vec2(cos * (size / 2.0),             sin * (size / 2.0)),             clr);
			mesh.colored_vertex(rect.center() - egui::vec2(cos * (size / 2.0 - thickness), sin * (size / 2.0 - thickness)), clr);
			
			mesh.add_triangle(i * 2,      i * 2 + 1,       (i * 2 + 2) % 72);
			mesh.add_triangle(i * 2 + 1, (i * 2 + 2) % 72, (i * 2 + 3) % 72);
		}
		
		draw.add(egui::Shape::mesh(mesh));
		draw.circle(
			rect.center() - egui::vec2(f32::cos(hsva.h * TAU) * ((size - thickness) / 2.0), f32::sin(hsva.h * TAU) * ((size - thickness) / 2.0)),
			thickness / 2.0,
			convert(hsva),
			egui::Stroke::new(ui.style().interact_selectable(&response, !(hover_ring || held == Held::Ring)).fg_stroke.width, contrast_color(hsva))
		);
		
		// triangle
		let mut mesh = egui::Mesh::default();
		mesh.colored_vertex(tri_black, egui::Color32::BLACK);
		mesh.colored_vertex(tri_clr, convert(egui::epaint::Hsva{h: hsva.h, s: 1.0, v: 1.0, a: 1.0}));
		mesh.colored_vertex(tri_white, egui::Color32::WHITE);
		mesh.add_triangle(0, 1, 2);
		draw.add(egui::Shape::mesh(mesh));
		
		draw.circle(
			egui::pos2(
				tri_black.x * (1.0 - hsva.v) + (tri_white.x * (1.0 - hsva.s) + tri_clr.x * hsva.s) * hsva.v,
				tri_black.y * (1.0 - hsva.v) + tri_white.y * hsva.v,
			),
			thickness / 2.0,
			convert(hsva),
			egui::Stroke::new(ui.style().interact_selectable(&response, !(hover_tri || held == Held::Triangle)).fg_stroke.width, contrast_color(hsva))
		);
	}
	
	if color.has_alpha() { // alpha
		let (rect, response) = ui.allocate_at_least(egui::vec2(size, thickness), egui::Sense::click_and_drag());
		
		if let Some(pos) = response.interact_pointer_pos() {
			hsva.a = ((pos.x - rect.min.x) / (rect.max.x - rect.min.x)).clamp(0.0, 1.0);
			
			color.set_srgba(hsva.to_rgba_unmultiplied());
			changed = true;
		}
		
		let draw = ui.painter();
		let cell = egui::vec2(thickness / 2.0, thickness / 2.0);
		draw.rect_filled(rect, egui::CornerRadius::ZERO, egui::Color32::GRAY);
		for i in 0..(size / thickness * 2.0) as usize {
			let offset = rect.min + egui::vec2(cell.x * i as f32, if i % 2 == 0 {0.0} else {cell.y});
			draw.rect_filled(egui::Rect{min: offset, max: offset + cell}, egui::CornerRadius::ZERO, egui::Color32::DARK_GRAY);
		}
		
		let mut mesh = egui::Mesh::default();
		mesh.colored_vertex(rect.min, egui::Color32::TRANSPARENT);
		mesh.colored_vertex(egui::pos2(rect.max.x, rect.min.y), convert(hsva));
		mesh.colored_vertex(egui::pos2(rect.min.x, rect.max.y), egui::Color32::TRANSPARENT);
		mesh.colored_vertex(rect.max, convert(hsva));
		mesh.add_triangle(0, 1, 2);
		mesh.add_triangle(2, 1, 3);
		draw.add(egui::Shape::mesh(mesh));
		
		let x = (rect.min.x + thickness * 0.2) * (1.0 - hsva.a) + (rect.max.x - thickness * 0.2) * hsva.a;
		draw.rect_stroke(
			egui::Rect{min: egui::pos2(x - thickness * 0.2, rect.min.y), max: egui::pos2(x + thickness * 0.2, rect.max.y)},
			egui::CornerRadius::ZERO,
			egui::Stroke::new(ui.style().interact(&response).fg_stroke.width, contrast_color(hsva)),
			egui::StrokeKind::Inside,
		);
	}
	
	{ // edit fields
		let mut clr = color.get_srgba();
		
		changed |= if color.has_alpha() {
			ui.num_multi_edit_range(&mut clr, "RGBA", &color.get_range())
		} else {
			ui.num_multi_edit_range(&mut clr[0..3], "RGB", &color.get_range()[0..3])
		}.changed();
		
		ui.horizontal(|ui| {
			ui.label("#");
			
			let mut text = format!("{:02x}{:02x}{:02x}{:02x}", (clr[0] * 255.0) as u8, (clr[1] * 255.0) as u8, (clr[2] * 255.0) as u8, (clr[3] * 255.0) as u8);
			ui.set_width(60.0);
			if ui.text_edit_singleline(&mut text).changed() {
				let hex = text.trim_start_matches("#");
				if hex.len() >= 6 {
					if let (Ok(r), Ok(g), Ok(b)) = (u8::from_str_radix(&hex[0..2], 16), u8::from_str_radix(&hex[2..4], 16), u8::from_str_radix(&hex[4..6], 16)) {
						clr[0] = r as f32 / 255.0;
						clr[1] = g as f32 / 255.0;
						clr[2] = b as f32 / 255.0;
					}
				}
				
				if color.has_alpha() && hex.len() >= 8 {
					if let Ok(a) = u8::from_str_radix(&hex[6..8], 16) {
						clr[3] = a as f32 / 255.0;
					}
				}
				
				changed = true;
			}
			
			ui.label("Hex")
		});
		
		color.set_srgba(clr);
	}
	
	ui.ctx().data_mut(|v| *v.get_temp_mut_or(egui::Id::NULL, (Held::None, 0.0)) = (held, hsva.h));
	
	changed
}