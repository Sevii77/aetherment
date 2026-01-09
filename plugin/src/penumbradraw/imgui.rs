use std::hash::{Hash, Hasher};
use super::imgui_bindings as sys;

impl Into<sys::ImVec2> for [f32; 2] {
	fn into(self) -> sys::ImVec2 {
		unsafe{std::mem::transmute(self)}
	}
}

pub fn dummy(size: [f32; 2]) {
	unsafe{sys::igDummy(size.into())};
}

pub fn same_line() {
	unsafe{sys::igSameLine(0.0, -1.0)};
}

pub fn push_id(id: impl core::hash::Hash, contents: impl FnOnce()) {
	let mut hash = std::hash::DefaultHasher::new();
	id.hash(&mut hash);
	let id = hash.finish();
	let id = (id >> 32) as u32 ^ id as u32;
	
	unsafe{sys::igPushID_Int(u32::cast_signed(id))};
	contents();
	unsafe{sys::igPopID()};
}

pub fn label(label: impl AsRef<str>) {
	let label = label.as_ref();
	unsafe{sys::igTextUnformatted(label.as_ptr() as _, (label.as_ptr() as usize + label.len()) as _)};
}

pub fn button(label: impl AsRef<str>) -> bool {
	let label = format!("{}\0", label.as_ref());
	unsafe{sys::igButton(label.as_ptr() as _, [0.0; 2].into())}
}

pub fn checkbox(val: &mut bool, label: impl AsRef<str>) -> bool {
	let label = format!("{}\0", label.as_ref());
	unsafe{sys::igCheckbox(label.as_ptr() as _, val)}
}

pub fn selectable_label(checked: bool, label: impl AsRef<str>) -> bool {
	let label = format!("{}\0", label.as_ref());
	unsafe{sys::igSelectable_Bool(label.as_ptr() as _, checked, 0 as _, [0.0; 2].into())}
}

pub fn selectable_value<C: PartialEq<S>, S: Into<C>>(current: &mut C, selected: S, label: impl AsRef<str>) -> bool {
	let r = selectable_label(*current == selected, label);
	if r {
		*current = selected.into();
	}
	
	r
}

pub fn slider<N: egui::emath::Numeric>(value: &mut N, range: std::ops::RangeInclusive<N>, label: impl AsRef<str>) -> bool {
	let label = format!("{}\0", label.as_ref());
	let r;
	if N::INTEGRAL {
		let mut v = value.to_f64() as i32;
		r = unsafe{sys::igSliderInt(label.as_ptr() as _, &mut v, range.start().to_f64() as i32, range.end().to_f64() as i32, 0 as _, 0 as _)};
		*value = N::from_f64(v as f64);
	} else {
		let mut v = value.to_f64() as f32;
		r = unsafe{sys::igSliderFloat(label.as_ptr() as _, &mut v, range.start().to_f64() as f32, range.end().to_f64() as f32, 0 as _, 0 as _)};
		*value = N::from_f64(v as f64);
	}
	
	r
}

pub fn combo(preview: impl AsRef<str>, label: impl AsRef<str>, contents: impl FnOnce()) {
	let preview = format!("{}\0", preview.as_ref());
	let label = format!("{}\0", label.as_ref());
	if unsafe{sys::igBeginCombo(label.as_ptr() as _, preview.as_ptr() as _, 0 as _)} {
		push_id(&label, contents);
		unsafe{sys::igEndCombo()};
	}
}

pub fn text_edit_singleline(text: &mut String, label: impl AsRef<str>) -> bool {
	unsafe extern "C" fn input_text_resize(data: *mut sys::ImGuiInputTextCallbackData) -> i32 {
		let data = &mut *data;
		match data.EventFlag {
			sys::ImGuiInputTextFlags_CallbackResize => {
				let s = &mut *(data.UserData as *mut String);
				s.reserve_exact((data.BufTextLen as usize).saturating_sub(s.len()));
				data.Buf = s.as_mut_ptr() as _;
				
				0
			}
			
			_ => 0
		}
	}
	
	text.push('\0');
	
	let label = format!("{}\0", label.as_ref());
	let r = unsafe{sys::igInputText(
		label.as_ptr() as _,
		text.as_mut_ptr() as _,
		text.capacity(),
		sys::ImGuiInputTextFlags_CallbackResize as _,
		Some(input_text_resize),
		text as *mut _ as _,
	)};
	
	let bytes = unsafe{std::slice::from_raw_parts(text.as_ptr(), text.capacity())};
	if let Some(p) = bytes.iter().position(|v| *v == 0) {
		text.truncate(p);
	}
	
	r
}

pub fn color_edit(color: &mut impl aetherment::ui_ext::ColorEditValue, label: impl AsRef<str>) -> bool {
	const FLAGS: sys::ImGuiColorEditFlags = sys::ImGuiColorEditFlags_DisplayRGB | sys::ImGuiColorEditFlags_DisplayHex | sys::ImGuiColorEditFlags_NoInputs |
		sys::ImGuiColorEditFlags_DisplayHSV | sys::ImGuiColorEditFlags_Uint8 | sys::ImGuiColorEditFlags_PickerHueWheel;
	
	const FLAGS_ALPHA: sys::ImGuiColorEditFlags = FLAGS | sys::ImGuiColorEditFlags_AlphaPreviewHalf | sys::ImGuiColorEditFlags_AlphaBar;
	
	let label = format!("{}\0", label.as_ref());
	
	let mut rgba = color.get_srgba();
	let r = if color.has_alpha() {
		unsafe{sys::igColorEdit4(label.as_ptr() as _, rgba.as_mut_ptr(), FLAGS_ALPHA)}
	} else {
		unsafe{sys::igColorEdit3(label.as_ptr() as _, rgba.as_mut_ptr(), FLAGS_ALPHA)}
	};
	
	color.set_srgba(rgba);
	
	r
}

pub fn tabbar(tabs: &[&str]) -> usize {
	let mut hash = std::hash::DefaultHasher::new();
	for tab in tabs {
		tab.hash(&mut hash);
	}
	let id = hash.finish();
	let id = format!("##{id}\0");
	
	let mut active = 0;
	if unsafe{sys::igBeginTabBar(id.as_ptr() as _, 0 as _)} {
		for (i, tab) in tabs.iter().enumerate() {
			let label = format!("{tab}\0");
			if unsafe{sys::igBeginTabItem(label.as_ptr() as _, 0 as _, 0)} {
				active = i;
				unsafe{sys::igEndTabItem()};
			}
		}
		
		unsafe{sys::igEndTabBar()};
	}
	
	active
}

pub fn hover(contents: impl FnOnce()) {
	if unsafe{sys::igIsItemHovered(0 as _)} {
		unsafe{sys::igBeginTooltip()};
		contents();
		unsafe{sys::igEndTooltip()};
	}
}

pub fn hover_text(text: impl AsRef<str>) {
	hover(|| label(text))
}