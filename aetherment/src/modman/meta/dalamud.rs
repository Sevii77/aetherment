// TODO: multi mod support (small mod that changes rounding or smth with a higher priority)
// TODO: gamma support, its offcolor for me :c

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::modman::settings;

pub(crate) static mut SETSTYLE: fn(&str) = |_json| {};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OptionOrStatic<T: OptionValue> {
	OptionSub(String, HashMap<String, T::Value>),
	Option(String),
	OptionMul(String, T::Value),
	Static(T::Value),
}

impl<T: OptionValue> OptionOrStatic<T> {
	fn resolve(&self, meta: &crate::modman::meta::Meta, settings: &settings::CollectionSettings) -> Option<T::Value> {
		match self {
			OptionOrStatic::OptionSub(opt, options) => settings.get(opt).and_then(|v| {
				match v {
					settings::Value::SingleFiles(v) => {
						let o = meta.options.options_iter().find_map(|v| {
							if v.name == *opt {
								if let crate::modman::meta::OptionSettings::SingleFiles(o) = &v.settings {
									return Some(o)
								}
							}
							
							None
						})?;
						
						let mut o2 = o.options.get(*v as usize)?;
						let mut val;
						loop {
							val = options.iter().find_map(|(n, v)| if n == &o2.name {Some(v.clone())} else {None});
							if val.is_some() {break}
							let Some(inherit) = &o2.inherit else {break};
							let Some(o3) = o.options.iter().find_map(|v| if v.name == *inherit {Some(v)} else {None}) else {break};
							o2 = o3;
						}
						
						val
					},
					
					settings::Value::MultiFiles(v) => {
						let o = meta.options.options_iter().find_map(|v| {
							if v.name == *opt {
								if let crate::modman::meta::OptionSettings::MultiFiles(o) = &v.settings {
									return Some(o)
								}
							}
							
							None
						})?;
						
						for (i, o) in o.options.iter().enumerate() {
							if *v & (1 << i) != 0 {
								for (n, v) in options.iter() {
									if *n == o.name {
										return Some(v.clone())
									}
								}
							}
						}
						
						None
					},
					
					_ => None
				}
			}),
			OptionOrStatic::Option(opt) => settings.get(opt).and_then(|a| T::get_value(a)),
			OptionOrStatic::OptionMul(opt, v) => settings.get(opt).and_then(|a| T::get_value(a).map(|a| T::multiplied(a, v.clone()))),
			OptionOrStatic::Static(v) => Some(v.clone()),
		}
	}
}

pub trait OptionValue {
	type Value: Clone;
	
	fn get_value(value: &settings::Value) -> Option<Self::Value>;
	fn multiplied(a: Self::Value, b: Self::Value) -> Self::Value;
}

impl OptionValue for i32 {
	type Value = Self;
	
	fn get_value(_value: &settings::Value) -> Option<Self::Value> {None}
	fn multiplied(a: Self::Value, b: Self::Value) -> Self::Value {a * b}
}

impl OptionValue for f32 {
	type Value = Self;
	
	fn get_value(_value: &settings::Value) -> Option<Self::Value> {None}
	fn multiplied(a: Self::Value, b: Self::Value) -> Self::Value {a * b}
}

impl OptionValue for [f32; 2] {
	type Value = Self;
	
	fn get_value(_value: &settings::Value) -> Option<Self::Value> {None}
	fn multiplied(a: Self::Value, b: Self::Value) -> Self::Value {[a[0] * b[0], a[1] * b[1]]}
}

impl OptionValue for [f32; 4] {
	type Value = Self;
	
	fn get_value(value: &settings::Value) -> Option<Self::Value> {
		match value {
			settings::Value::Rgba(v) => Some(*v),
			settings::Value::Rgb(v) => Some([v[0], v[1], v[2], 1.0]),
			settings::Value::Grayscale(v) => Some([*v, *v, *v, 1.0]),
			settings::Value::Opacity(v) => Some([1.0, 1.0, 1.0, *v]),
			_ => None,
		}
	}
	
	fn multiplied(a: Self::Value, b: Self::Value) -> Self::Value {
		[
			(a[0] * b[0]).min(1.0),
			(a[1] * b[1]).min(1.0),
			(a[2] * b[2]).min(1.0),
			(a[3] * b[3]).min(1.0),
		]
	}
}

// ----------

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
#[serde(rename_all = "PascalCase")]
pub struct Style {
	pub alpha: OptionOrStatic<f32>,
	pub window_padding: OptionOrStatic<[f32; 2]>,
	pub window_rounding: OptionOrStatic<f32>,
	pub window_border_size: OptionOrStatic<f32>,
	pub window_title_align: OptionOrStatic<[f32; 2]>,
	pub window_menu_button_position: OptionOrStatic<i32>,
	pub child_rounding: OptionOrStatic<f32>,
	pub child_border_size: OptionOrStatic<f32>,
	pub popup_rounding: OptionOrStatic<f32>,
	pub popup_border_size: OptionOrStatic<f32>,
	pub frame_padding: OptionOrStatic<[f32; 2]>,
	pub frame_rounding: OptionOrStatic<f32>,
	pub frame_border_size: OptionOrStatic<f32>,
	pub item_spacing: OptionOrStatic<[f32; 2]>,
	pub item_inner_spacing: OptionOrStatic<[f32; 2]>,
	pub cell_padding: OptionOrStatic<[f32; 2]>,
	pub touch_extra_padding: OptionOrStatic<[f32; 2]>,
	pub indent_spacing: OptionOrStatic<f32>,
	pub scrollbar_size: OptionOrStatic<f32>,
	pub scrollbar_rounding: OptionOrStatic<f32>,
	pub grab_min_size: OptionOrStatic<f32>,
	pub grab_rounding: OptionOrStatic<f32>,
	pub log_slider_deadzone: OptionOrStatic<f32>,
	pub tab_rounding: OptionOrStatic<f32>,
	pub tab_border_size: OptionOrStatic<f32>,
	pub button_text_align: OptionOrStatic<[f32; 2]>,
	pub selectable_text_align: OptionOrStatic<[f32; 2]>,
	pub display_safe_area_padding: OptionOrStatic<[f32; 2]>,
	pub colors: HashMap<String, OptionOrStatic<[f32; 4]>>,
}

impl Default for Style {
	fn default() -> Self {
		Self {
			alpha: OptionOrStatic::Static(1.0),
			window_padding: OptionOrStatic::Static([8.0, 8.0]),
			window_rounding: OptionOrStatic::Static(4.0),
			window_border_size: OptionOrStatic::Static(0.0),
			window_title_align: OptionOrStatic::Static([0.0, 0.5]),
			window_menu_button_position: OptionOrStatic::Static(1),
			child_rounding: OptionOrStatic::Static(0.0),
			child_border_size: OptionOrStatic::Static(1.0),
			popup_rounding: OptionOrStatic::Static(0.0),
			popup_border_size: OptionOrStatic::Static(0.0),
			frame_padding: OptionOrStatic::Static([4.0, 3.0]),
			frame_rounding: OptionOrStatic::Static(4.0),
			frame_border_size: OptionOrStatic::Static(0.0),
			item_spacing: OptionOrStatic::Static([8.0, 4.0]),
			item_inner_spacing: OptionOrStatic::Static([4.0, 4.0]),
			cell_padding: OptionOrStatic::Static([4.0, 2.0]),
			touch_extra_padding: OptionOrStatic::Static([0.0, 0.0]),
			indent_spacing: OptionOrStatic::Static(21.0),
			scrollbar_size: OptionOrStatic::Static(16.0),
			scrollbar_rounding: OptionOrStatic::Static(9.0),
			grab_min_size: OptionOrStatic::Static(13.0),
			grab_rounding: OptionOrStatic::Static(3.0),
			log_slider_deadzone: OptionOrStatic::Static(4.0),
			tab_rounding: OptionOrStatic::Static(4.0),
			tab_border_size: OptionOrStatic::Static(0.0),
			button_text_align: OptionOrStatic::Static([0.5, 0.5]),
			selectable_text_align: OptionOrStatic::Static([0.0, 0.0]),
			display_safe_area_padding: OptionOrStatic::Static([3.0, 3.0]),
			colors: HashMap::new(),
		}
	}
}

impl Style {
	pub fn apply(&self, name: &str, meta: &crate::modman::meta::Meta, settings: &settings::CollectionSettings, gamma: f32) {
		let model = StyleModel {
			name: name.to_owned(),
			built_in_colors: HashMap::new(),
			version: 1,
			alpha: self.alpha.resolve(meta, settings).unwrap_or_default(),
			window_padding: self.window_padding.resolve(meta, settings).unwrap_or_default().into(),
			window_rounding: self.window_rounding.resolve(meta, settings).unwrap_or_default(),
			window_border_size: self.window_border_size.resolve(meta, settings).unwrap_or_default(),
			window_title_align: self.window_title_align.resolve(meta, settings).unwrap_or_default().into(),
			window_menu_button_position: self.window_menu_button_position.resolve(meta, settings).unwrap_or_default(),
			child_rounding: self.child_rounding.resolve(meta, settings).unwrap_or_default(),
			child_border_size: self.child_border_size.resolve(meta, settings).unwrap_or_default(),
			popup_rounding: self.popup_rounding.resolve(meta, settings).unwrap_or_default(),
			popup_border_size: self.popup_border_size.resolve(meta, settings).unwrap_or_default(),
			frame_padding: self.frame_padding.resolve(meta, settings).unwrap_or_default().into(),
			frame_rounding: self.frame_rounding.resolve(meta, settings).unwrap_or_default(),
			frame_border_size: self.frame_border_size.resolve(meta, settings).unwrap_or_default(),
			item_spacing: self.item_spacing.resolve(meta, settings).unwrap_or_default().into(),
			item_inner_spacing: self.item_inner_spacing.resolve(meta, settings).unwrap_or_default().into(),
			cell_padding: self.cell_padding.resolve(meta, settings).unwrap_or_default().into(),
			touch_extra_padding: self.touch_extra_padding.resolve(meta, settings).unwrap_or_default().into(),
			indent_spacing: self.indent_spacing.resolve(meta, settings).unwrap_or_default(),
			scrollbar_size: self.scrollbar_size.resolve(meta, settings).unwrap_or_default(),
			scrollbar_rounding: self.scrollbar_rounding.resolve(meta, settings).unwrap_or_default(),
			grab_min_size: self.grab_min_size.resolve(meta, settings).unwrap_or_default(),
			grab_rounding: self.grab_rounding.resolve(meta, settings).unwrap_or_default(),
			log_slider_deadzone: self.log_slider_deadzone.resolve(meta, settings).unwrap_or_default(),
			tab_rounding: self.tab_rounding.resolve(meta, settings).unwrap_or_default(),
			tab_border_size: self.tab_border_size.resolve(meta, settings).unwrap_or_default(),
			button_text_align: self.button_text_align.resolve(meta, settings).unwrap_or_default().into(),
			selectable_text_align: self.selectable_text_align.resolve(meta, settings).unwrap_or_default().into(),
			display_safe_area_padding: self.display_safe_area_padding.resolve(meta, settings).unwrap_or_default().into(),
			colors: self.colors.iter().map(|(k, v)| (k.to_owned(), {let c = v.resolve(meta, settings).unwrap_or([1.0; 4]); [c[0].powf(gamma).clamp(0.0, 1.0), c[1].powf(gamma).clamp(0.0, 1.0), c[2].powf(gamma).clamp(0.0, 1.0), c[3]]}.into())).collect()
		};
		
		if let Ok(json) = serde_json::to_string(&model) {
			unsafe{SETSTYLE(&json)};
		}
	}
}

// ----------

#[derive(Debug, Clone, Serialize)]
struct StyleModel {
	name: String,
	#[serde(rename = "dol")] built_in_colors: HashMap<String, V4>,
	#[serde(rename = "ver")] version: i32,
	#[serde(rename = "a")] alpha: f32,
	#[serde(rename = "b")] window_padding: V2,
	#[serde(rename = "c")] window_rounding: f32,
	#[serde(rename = "d")] window_border_size: f32,
	#[serde(rename = "e")] window_title_align: V2,
	#[serde(rename = "f")] window_menu_button_position: i32,
	#[serde(rename = "g")] child_rounding: f32,
	#[serde(rename = "h")] child_border_size: f32,
	#[serde(rename = "i")] popup_rounding: f32,
	#[serde(rename = "ab")] popup_border_size: f32,
	#[serde(rename = "j")] frame_padding: V2,
	#[serde(rename = "k")] frame_rounding: f32,
	#[serde(rename = "l")] frame_border_size: f32,
	#[serde(rename = "m")] item_spacing: V2,
	#[serde(rename = "n")] item_inner_spacing: V2,
	#[serde(rename = "o")] cell_padding: V2,
	#[serde(rename = "p")] touch_extra_padding: V2,
	#[serde(rename = "q")] indent_spacing: f32,
	#[serde(rename = "r")] scrollbar_size: f32,
	#[serde(rename = "s")] scrollbar_rounding: f32,
	#[serde(rename = "t")] grab_min_size: f32,
	#[serde(rename = "u")] grab_rounding: f32,
	#[serde(rename = "v")] log_slider_deadzone: f32,
	#[serde(rename = "w")] tab_rounding: f32,
	#[serde(rename = "x")] tab_border_size: f32,
	#[serde(rename = "y")] button_text_align: V2,
	#[serde(rename = "z")] selectable_text_align: V2,
	#[serde(rename = "aa")] display_safe_area_padding: V2,
	#[serde(rename = "col")] colors: HashMap<String, V4>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct V2 {
	x: f32,
	y: f32,
}

impl Into<V2> for [f32; 2] {
	fn into(self) -> V2 {
		V2{x: self[0], y: self[1]}
	}
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct V4 {
	x: f32,
	y: f32,
	z: f32,
	w: f32,
}

impl Into<V4> for [f32; 4] {
	fn into(self) -> V4 {
		V4{x: self[0], y: self[1], z: self[2], w: self[3]}
	}
}