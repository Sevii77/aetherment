use std::{ffi::CString, hash::{Hash, Hasher}};
use crate::Response as Resp;

#[path = "imgui/bindings.rs"]
#[allow(warnings)]
mod sys;

impl Into<Resp> for bool {
	fn into(self) -> Resp {
		let hovered = unsafe{sys::igIsItemHovered(0)};
		
		Resp {
			clicked: self,
			double_clicked: hovered && unsafe{sys::igIsMouseDoubleClicked(0)},
			changed: self,
			held: unsafe{sys::igIsItemActive()},
			hovered,
		}
	}
}

// TODO: font awesome icons
impl crate::Icon {
	pub fn str(&self) -> &'static str {
		match self {
			crate::Icon::Dir => "",
			crate::Icon::DirOpen => "",
			crate::Icon::File => "",
		}
	}
}

fn id_u32(id: impl Hash) -> u32 {
	let mut hasher = std::hash::DefaultHasher::new();
	id.hash(&mut hasher);
	hasher.finish() as u32
}

fn id_str(id: impl Hash) -> CString {
	CString::new(id_u32(id).to_string()).unwrap()
}

pub struct Ui<'a> {
	a: std::marker::PhantomData<&'a mut i32>,
	horizontal: bool,
	horizontal_first: bool,
}

impl<'a> Ui<'a> {
	pub fn new() -> Self {
		Self {
			a: std::marker::PhantomData,
			horizontal: false,
			horizontal_first: false,
		}
	}
	
	fn handle_horizontal(&mut self) {
		if self.horizontal && !self.horizontal_first {unsafe{sys::igSameLine(0.0, -1.0)}} else {self.horizontal_first = false}
	}
	
	fn text_size(&mut self, text: &str) -> [f32; 2] {
		let mut size = sys::ImVec2{x: 0.0, y: 0.0};
		unsafe{sys::igCalcTextSize(&mut size, text.as_ptr() as _, (text.as_ptr() as usize + text.len()) as _, false, 999999.0)};
		[size.x, size.y]
	}
	
	pub fn debug(&mut self) {
		self.label("debug here");
	}
	
	pub fn get_f32(&mut self, key: impl Hash, default: f32) -> f32 {
		unsafe{sys::ImGuiStorage_GetFloat(sys::igGetStateStorage(), id_u32(key), default)}
	}
	
	pub fn set_f32(&mut self, key: impl Hash, value: f32) {
		unsafe{sys::ImGuiStorage_SetFloat(sys::igGetStateStorage(), id_u32(key), value)}
	}
	
	pub fn get_i32(&mut self, key: impl Hash, default: i32) -> i32 {
		unsafe{sys::ImGuiStorage_GetInt(sys::igGetStateStorage(), id_u32(key), default)}
	}
	
	pub fn set_i32(&mut self, key: impl Hash, value: i32) {
		unsafe{sys::ImGuiStorage_SetInt(sys::igGetStateStorage(), id_u32(key), value)}
	}
	
	pub fn mouse_pos(&mut self) -> [f32; 2] {
		unsafe{
			let mut size = sys::ImVec2{x: 0.0, y: 0.0};
			sys::igGetMousePos(&mut size);
			[size.x, size.y]
		}
	}
	
	pub fn modifiers(&mut self) -> crate::Modifiers {
		let io = unsafe{&*sys::igGetIO()};
		crate::Modifiers {
			alt: io.KeyAlt,
			ctrl: io.KeyCtrl,
			shift: io.KeyShift,
		}
	}
	
	pub fn available_size(&mut self) -> [f32; 2] {
		unsafe{
			let mut size = sys::ImVec2{x: 0.0, y: 0.0};
			sys::igGetContentRegionAvail(&mut size);
			[size.x, size.y]
		}
	}
	
	pub fn push_id(&mut self, id: impl Hash, contents: impl FnOnce(&mut Ui)) {
		unsafe{sys::igPushID_Int(id_u32(id) as i32)};
		contents(self);
		unsafe{sys::igPopID()};
	}
	
	pub fn enabled(&mut self, enabled: bool, contents: impl FnOnce(&mut Ui)) {
		unsafe{sys::igBeginDisabled(!enabled)};
		contents(self);
		unsafe{sys::igEndDisabled()};
	}
	
	pub fn indent(&mut self, contents: impl FnOnce(&mut Ui)) {
		unsafe{sys::igIndent(0.0)};
		contents(self);
		unsafe{sys::igUnindent(0.0)};
	}
	
	pub fn set_width(&mut self, width: f32) {
		unsafe{sys::igSetNextItemWidth(width)};
	}
	
	// Elements
	pub fn window(&mut self, args: crate::WindowArgs, contents: impl FnOnce(&mut Ui)) {
		unsafe {
			let mut flags = 0;
			
			match args.pos {
				crate::Arg::Once(v) => {
					sys::igSetNextWindowPos(sys::ImVec2{x: v[0], y: v[1]}, sys::ImGuiCond_FirstUseEver as i32, sys::ImVec2{x: 0.0, y: 0.0})
				}
				
				crate::Arg::Always(v) => {
					flags |= sys::ImGuiWindowFlags_NoMove;
					sys::igSetNextWindowPos(sys::ImVec2{x: v[0], y: v[1]}, sys::ImGuiCond_Always as i32, sys::ImVec2{x: 0.0, y: 0.0})
				}
			};
			
			match args.size {
				crate::Arg::Once(v) => {
					// flags |= sys::ImGuiWindowFlags_AlwaysAutoResize;
					sys::igSetNextWindowSize(sys::ImVec2{x: v[0], y: v[1]}, sys::ImGuiCond_FirstUseEver as i32)
				}
				
				crate::Arg::Always(v) => {
					flags |= sys::ImGuiWindowFlags_NoResize;
					sys::igSetNextWindowSize(sys::ImVec2{x: v[0], y: v[1]}, sys::ImGuiCond_Always as i32)
				}
			};
			
			let title = CString::new(args.title).unwrap();
			sys::igBegin(title.as_ptr(), if let Some(open) = args.open {open} else {0 as _}, flags as i32);
		}
		
		contents(self);
		
		unsafe{sys::igEnd()}
	}
	
	pub fn child(&mut self, id: impl Hash, size: [f32; 2], contents: impl FnOnce(&mut Ui)) {
		if unsafe{sys::igBeginChild_ID(id_u32(id), sys::ImVec2{x: size[0], y: size[1]}, false, 0)} {
			contents(self);
			unsafe{sys::igEndChild()}
		}
	}
	
	pub fn splitter(&mut self, id: impl Hash, default: f32, contents: impl FnOnce(&mut Ui, &mut Ui)) {
		unsafe {
			sys::igBeginTable(id_str(id).as_ptr(), 2, 1, sys::ImVec2{x: 0.0, y: 0.0}, 0.0);
			sys::igTableSetupColumn(id_str("left").as_ptr(), 4, default, 0);
			sys::igTableSetupColumn(id_str("right").as_ptr(), 4, 0.0, 0);
			sys::igTableNextRow(0, 0.0);
			
			sys::igTableNextColumn();
			let size = self.available_size();
			let h = size[1] - (&mut *sys::igGetStyle()).ItemSpacing.x;
			sys::igBeginChild_ID(id_u32("left"), sys::ImVec2{x: 0.0, y: h}, false, 0);
		}
		
		contents(self, &mut Self::new());
		
		unsafe {
			sys::igEndChild();
			sys::igEndTable();
		}
	}
	
	// can easily break shit, but thats what i get for using imgui like this
	pub fn mark_next_splitter(&mut self) {
		unsafe {
			sys::igEndChild();
			sys::igTableNextColumn();
			let size = self.available_size();
			let h = size[1] - (&mut *sys::igGetStyle()).ItemSpacing.x;
			sys::igBeginChild_ID(id_u32("right"), sys::ImVec2{x: 0.0, y: h}, false, 0);
		}
	}
	
	pub fn horizontal<T>(&mut self, contents: impl FnOnce(&mut Ui) -> T) -> T {
		self.horizontal = true;
		self.horizontal_first = true;
		let r = contents(self);
		self.horizontal = false;
		r
	}
	
	pub fn label<S: AsRef<str>>(&mut self, label: S) {
		self.handle_horizontal();
		let label = label.as_ref();
		unsafe{sys::igTextUnformatted(label.as_ptr() as _, (label.as_ptr() as usize + label.len()) as _)};
	}
	
	pub fn button<S: AsRef<str>>(&mut self, label: S) -> Resp {
		self.handle_horizontal();
		let label = CString::new(label.as_ref()).unwrap();
		unsafe{sys::igButton(label.as_ptr(), sys::ImVec2{x: 0.0, y: 0.0})}.into()
	}
	
	pub fn selectable<S: AsRef<str>>(&mut self, label: S, selected: bool) -> Resp {
		self.handle_horizontal();
		let clabel = CString::new(label.as_ref()).unwrap();
		unsafe{sys::igSelectable_Bool(clabel.as_ptr(), selected, 0, sys::ImVec2{x: self.text_size(label.as_ref())[0], y: 0.0})}.into()
	}
	
	pub fn checkbox<S: AsRef<str>>(&mut self, label: S, checked: &mut bool) -> Resp {
		self.handle_horizontal();
		let label = CString::new(label.as_ref()).unwrap();
		unsafe{sys::igCheckbox(label.as_ptr(), checked)}.into()
	}
	
	pub fn input_text<S: AsRef<str>>(&mut self, label: S, string: &mut String) -> Resp {
		self.handle_horizontal();
		string.push('\0');
		let label = CString::new(label.as_ref()).unwrap();
		let r = unsafe{sys::igInputText(label.as_ptr(), string.as_mut_ptr() as *mut _, 256, 0, None, std::ptr::null::<*mut i8>() as *mut _)};
		if let Some(p) = string.find('\0') {string.truncate(p);}
		r.into()
	}
	
	pub fn combo<S: AsRef<str>, S2: AsRef<str>>(&mut self, label: S, preview: S2, contents: impl FnOnce(&mut Ui)) {
		let label = CString::new(label.as_ref()).unwrap();
		let preview = CString::new(preview.as_ref()).unwrap();
		if unsafe{sys::igBeginCombo(label.as_ptr(), preview.as_ptr(), 0)} {
			contents(self);
			unsafe{sys::igEndCombo()};
		}
	}
	
	pub fn helptext<S: AsRef<str>>(&mut self, text: S) {
		self.label("❓");
		if unsafe{sys::igIsItemHovered(0)} {
			let text = CString::new(text.as_ref()).unwrap();
			unsafe{sys::igSetTooltip(text.as_ptr())}
		}
	}
	
	pub fn color_edit_rgb<S: AsRef<str>>(&mut self, label: S, color: &mut [f32; 3]) -> Resp {
		self.handle_horizontal();
		let label = CString::new(label.as_ref()).unwrap();
		unsafe{sys::igColorEdit3(label.as_ptr(), color.as_mut_ptr(), (
			sys::ImGuiColorEditFlags_NoInputs |
			sys::ImGuiColorEditFlags_PickerHueWheel) as i32)}.into()
	}
	
	pub fn color_edit_rgba<S: AsRef<str>>(&mut self, label: S, color: &mut [f32; 4]) -> Resp {
		self.handle_horizontal();
		let label = CString::new(label.as_ref()).unwrap();
		unsafe{sys::igColorEdit4(label.as_ptr(), color.as_mut_ptr(), (
			sys::ImGuiColorEditFlags_NoInputs |
			sys::ImGuiColorEditFlags_PickerHueWheel |
			sys::ImGuiColorEditFlags_AlphaBar |
			sys::ImGuiColorEditFlags_AlphaPreviewHalf) as i32)}.into()
	}
}