use std::hash::Hash;
use crate::Response as Resp;

mod splitter;

impl Into<Resp> for egui::Response {
	fn into(self) -> Resp {
		Resp {
			clicked: self.clicked(),
			double_clicked: self.double_clicked(),
			changed: self.changed(),
			held: self.is_pointer_button_down_on(),
			hovered: self.hovered(),
		}
	}
}

impl crate::Icon {
	pub fn str(&self) -> &'static str {
		match self {
			crate::Icon::Dir => "🗀",
			crate::Icon::DirOpen => "🗁",
			crate::Icon::File => "🗋",
		}
	}
}

pub struct Ui<'a> {
	ui: &'a mut egui::Ui,
	next_width: Option<f32>,
}

impl<'a> Ui<'a> {
	pub fn new(ui: &'a mut egui::Ui) -> Self {
		Self {
			ui,
			next_width: None,
		}
	}
	
	fn handle_add(&mut self, widget: impl egui::Widget) -> egui::Response {
		if let Some(w) = self.next_width.take() {
			// self.ui.add_sized([w, 0.0], widget)
			let layout = egui::Layout::top_down_justified(egui::Align::LEFT);
			self.ui.allocate_ui_with_layout([w, 0.0].into(), layout, |ui| ui.add(widget)).inner
		} else {
			self.ui.add(widget)
		}
	}
	
	pub fn debug(&mut self) {
		self.ui.ctx().clone().style_ui(self.ui);
	}
	
	pub fn get_f32(&mut self, key: impl Hash, default: f32) -> f32 {
		self.ui.memory_mut(|m| *m.data.get_persisted_mut_or(egui::Id::new(key), default))
	}
	
	pub fn set_f32(&mut self, key: impl Hash, value: f32) {
		self.ui.memory_mut(|m| *m.data.get_persisted_mut_or(egui::Id::new(key), value) = value);
	}
	
	pub fn get_i32(&mut self, key: impl Hash, default: i32) -> i32 {
		self.ui.memory_mut(|m| *m.data.get_persisted_mut_or(egui::Id::new(key), default))
	}
	
	pub fn set_i32(&mut self, key: impl Hash, value: i32) {
		self.ui.memory_mut(|m| *m.data.get_persisted_mut_or(egui::Id::new(key), value) = value);
	}
	
	pub fn mouse_pos(&mut self) -> [f32; 2] {
		self.ui.ctx().pointer_interact_pos().map_or_else(|| self.ui.ctx().pointer_latest_pos().map_or_else(|| [0.0f32; 2], |v| [v.x, v.y]), |v| [v.x, v.y])
	}
	
	pub fn set_clipboard<S: AsRef<str>>(&mut self, text: S) {
		todo!()
	}
	
	pub fn get_clipboard(&mut self) -> String {
		todo!()
	}
	
	pub fn modifiers(&mut self) -> crate::Modifiers {
		let v = self.ui.ctx().input(|v| v.modifiers);
		crate::Modifiers {
			alt: v.alt,
			ctrl: v.ctrl,
			shift: v.shift,
		}
	}
	
	pub fn available_size(&mut self) -> [f32; 2] {
		self.ui.available_size().into()
	}
	
	pub fn push_id(&mut self, id: impl Hash, contents: impl FnOnce(&mut Ui)) {
		self.ui.push_id(id, |ui| contents(&mut Ui::new(ui)));
	}
	
	pub fn enabled(&mut self, enabled: bool, contents: impl FnOnce(&mut Ui)) {
		self.ui.add_enabled_ui(enabled, |ui| contents(&mut Ui::new(ui)));
	}
	
	pub fn indent(&mut self, contents: impl FnOnce(&mut Ui)) {
		self.ui.indent("indent", |ui| contents(&mut Ui::new(ui)));
	}
	
	pub fn set_width(&mut self, width: f32) {
		self.next_width = Some(width);
	}
	
	pub fn add_space(&mut self, spacing: f32) {
		todo!()
	}
	
	pub fn tooltip(&mut self, contents: impl FnOnce(&mut Ui)) {
		egui::show_tooltip_at_pointer(self.ui.ctx(), egui::Id::new("tooltip"), |ui| contents(&mut Ui::new(ui)));
	}
	
	// Elements
	pub fn window(&mut self, args: crate::WindowArgs, contents: impl FnOnce(&mut Ui)) {
		let window = egui::Window::new(args.title)
			.min_size(args.min_size)
			.max_size(args.max_size);
		
		let window = match args.pos {
			crate::Arg::Once(v) => window.default_pos(v).movable(true),
			crate::Arg::Always(v) => window.fixed_pos(v).movable(false),
		};
		
		let window = match args.size {
			crate::Arg::Once(v) => window.default_size(v).resizable(true),
			crate::Arg::Always(v) => window.fixed_size(v).resizable(false),
		};
		
		
		match args.open {
			Some(v) => window.open(v),
			None => window,
		}.show(self.ui.ctx(), |ui| contents(&mut Ui::new(ui)));
	}
	
	pub fn child(&mut self, id: impl Hash, size: [f32; 2], contents: impl FnOnce(&mut Ui)) {
		egui::ScrollArea::both()
			.max_width(size[0])
			.max_height(size[1])
			.show(self.ui, |ui| ui.push_id(id, |ui| contents(&mut Ui::new(ui))));
		
		// let rect = egui::Rect::from_min_size(self.ui.next_widget_position(), size.into());
		// let mut child = self.ui.child_ui_with_id_source(rect, egui::Layout::default(), id);
		// child.set_clip_rect(rect);
		// contents(&mut Ui::new(&mut child));
	}
	
	pub fn splitter(&mut self, id: impl Hash, default: f32, contents: impl FnOnce(&mut Ui, &mut Ui)) {
		splitter::Splitter::new(id, splitter::SplitterAxis::Horizontal).default_pos(default).show(self.ui, |left, right|
			contents(&mut Ui::new(left), &mut Ui::new(right)));
	}
	
	// needed for imgui
	pub fn mark_next_splitter(&mut self) {}
	
	pub fn horizontal<T>(&mut self, contents: impl FnOnce(&mut Ui) -> T) -> T {
		self.ui.horizontal(|ui| contents(&mut Ui::new(ui))).inner
	}
	
	pub fn label<S: AsRef<str>>(&mut self, label: S) {
		self.handle_add(egui::Label::new(label.as_ref()));
	}
	
	pub fn button<S: AsRef<str>>(&mut self, label: S) -> Resp {
		self.handle_add(egui::Button::new(label.as_ref())).into()
	}
	
	pub fn selectable<S: AsRef<str>>(&mut self, label: S, selected: bool) -> Resp {
		todo!()
	}
	
	pub fn selectable_min<S: AsRef<str>>(&mut self, label: S, selected: bool) -> Resp {
		self.handle_add(egui::SelectableLabel::new(selected, label.as_ref())).into()
	}
	
	pub fn checkbox<S: AsRef<str>>(&mut self, label: S, checked: &mut bool) -> Resp {
		self.handle_add(egui::Checkbox::new(checked, label.as_ref())).into()
	}
	
	pub fn input_text<S: AsRef<str>>(&mut self, label: S, string: &mut String) -> Resp {
		self.ui.horizontal(|ui| {
			let mut widget = egui::TextEdit::singleline(string);
			widget = if let Some(w) = self.next_width.take() {widget.desired_width(w)} else {widget};
			let r = widget.show(ui).response;
			// let r = ui.text_edit_singleline(string);
			ui.label(label.as_ref());
			r
		}).inner.into()
	}
	
	pub fn combo<S: AsRef<str>, S2: AsRef<str>>(&mut self, label: S, preview: S2, contents: impl FnOnce(&mut Ui)) {
		let mut widget = egui::ComboBox::from_label(label.as_ref())
			.selected_text(preview.as_ref());
		widget = if let Some(w) = self.next_width.take() {widget.width(w)} else {widget};
		widget.show_ui(self.ui, |ui| contents(&mut Ui::new(ui)));
	}
	
	pub fn helptext<S: AsRef<str>>(&mut self, text: S) {
		self.ui.label("❓").on_hover_text(text.as_ref());
	}
	
	pub fn color_edit_rgb<S: AsRef<str>>(&mut self, label: S, color: &mut [f32; 3]) -> Resp {
		// TODO: proper color edit
		num_multi_edit_range(self.ui, color, label.as_ref(), &[0.0..=1.0, 0.0..=1.0, 0.0..=1.0]).into()
	}
	
	pub fn color_edit_rgba<S: AsRef<str>>(&mut self, label: S, color: &mut [f32; 4]) -> Resp {
		// TODO: proper color edit
		num_multi_edit_range(self.ui, color, label.as_ref(), &[0.0..=1.0, 0.0..=1.0, 0.0..=1.0, 0.0..=1.0]).into()
	}
}

fn num_multi_edit_range<Num: egui::emath::Numeric>(ui: &mut egui::Ui, values: &mut [Num], label: impl Into<egui::WidgetText>, range: &[std::ops::RangeInclusive<Num>]) -> egui::Response {
	ui.horizontal(|ui| {
		let mut resp = ui.add(create_drag(&mut values[0]).clamp_range(range[0].clone()));
		for (i, value) in values.iter_mut().skip(1).enumerate() {
			resp |= ui.add(create_drag(value).clamp_range(range[i].clone()));
		}
		ui.label(label.into());
		resp
	}).inner
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