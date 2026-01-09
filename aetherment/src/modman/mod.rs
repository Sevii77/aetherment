use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::EnumTools;

#[cfg(any(feature = "plugin", feature = "client"))] pub mod backend;
pub mod meta;
pub mod settings;
pub mod composite;
pub mod requirement;
pub mod modpack;
pub mod manager;

// ----------

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum OptionOrStatic<T: OptionValue> {
	OptionSub(String, HashMap<String, T::Value>),
	Option(String),
	OptionMul(String, T::Value),
	OptionGradiant(String, String, T::Value),
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
			OptionOrStatic::OptionGradiant(opt, opt2, v) => Some(T::gradiant(
				T::get_value(settings.get(opt)?)?,
				T::get_value(settings.get(opt2)?)?,
				v.clone()
			)),
			OptionOrStatic::Static(v) => Some(v.clone()),
		}
	}
}

pub trait OptionValue {
	type Value: Clone;
	
	fn get_value(value: &settings::Value) -> Option<Self::Value>;
	fn multiplied(a: Self::Value, b: Self::Value) -> Self::Value;
	fn gradiant(a: Self::Value, b: Self::Value, scale: Self::Value) -> Self::Value;
}

impl OptionValue for i32 {
	type Value = Self;
	
	fn get_value(_value: &settings::Value) -> Option<Self::Value> {None}
	fn multiplied(a: Self::Value, b: Self::Value) -> Self::Value {a * b}
	fn gradiant(a: Self::Value, b: Self::Value, scale: Self::Value) -> Self::Value {a * (1 - scale) + b * scale}
}

impl OptionValue for f32 {
	type Value = Self;
	
	fn get_value(_value: &settings::Value) -> Option<Self::Value> {None}
	fn multiplied(a: Self::Value, b: Self::Value) -> Self::Value {a * b}
	fn gradiant(a: Self::Value, b: Self::Value, scale: Self::Value) -> Self::Value {a * (1.0 - scale) + b * scale}
}

impl OptionValue for [f32; 2] {
	type Value = Self;
	
	fn get_value(_value: &settings::Value) -> Option<Self::Value> {None}
	fn multiplied(a: Self::Value, b: Self::Value) -> Self::Value {[a[0] * b[0], a[1] * b[1]]}
	fn gradiant(a: Self::Value, b: Self::Value, scale: Self::Value) -> Self::Value {
		[
			a[0] * (1.0 - scale[0]) + b[0] * scale[0],
			a[1] * (1.0 - scale[1]) + b[1] * scale[1],
		]
	}
}

impl OptionValue for [f32; 3] {
	type Value = Self;
	
	fn get_value(value: &settings::Value) -> Option<Self::Value> {
		match value {
			settings::Value::Rgba(v) => Some([v[0], v[1], v[2]]),
			settings::Value::Rgb(v) => Some(*v),
			settings::Value::Grayscale(v) => Some([*v, *v, *v]),
			_ => None,
		}
	}
	
	fn multiplied(a: Self::Value, b: Self::Value) -> Self::Value {
		[
			(a[0] * b[0]).clamp(0.0, 1.0),
			(a[1] * b[1]).clamp(0.0, 1.0),
			(a[2] * b[2]).clamp(0.0, 1.0),
		]
	}
	
	fn gradiant(a: Self::Value, b: Self::Value, scale: Self::Value) -> Self::Value {
		[
			a[0] * (1.0 - scale[0]) + b[0] * scale[0],
			a[1] * (1.0 - scale[1]) + b[1] * scale[1],
			a[2] * (1.0 - scale[2]) + b[2] * scale[2],
		]
	}
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
			(a[0] * b[0]).clamp(0.0, 1.0),
			(a[1] * b[1]).clamp(0.0, 1.0),
			(a[2] * b[2]).clamp(0.0, 1.0),
			(a[3] * b[3]).clamp(0.0, 1.0),
		]
	}
	
	fn gradiant(a: Self::Value, b: Self::Value, scale: Self::Value) -> Self::Value {
		[
			a[0] * (1.0 - scale[0]) + b[0] * scale[0],
			a[1] * (1.0 - scale[1]) + b[1] * scale[1],
			a[2] * (1.0 - scale[2]) + b[2] * scale[2],
			a[3] * (1.0 - scale[3]) + b[3] * scale[3],
		]
	}
}

// ----------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Path {
	Mod(String),
	Game(String),
	Option(String, String),
}

impl Path {
	pub fn resolve_option(&self, meta: &meta::Meta, settings: &settings::CollectionSettings) -> Option<String> {
		let Self::Option(id, sub_id) = self else {return None};
		let Some(setting) = settings.get(id) else {return None};
		let crate::modman::settings::Value::Path(i) = setting else {return None};
		let Some(option) = meta.options.options_iter().find(|v| v.name == *id) else {return None};
		let meta::OptionSettings::Path(v) = &option.settings else {return None};
		let Some((_, paths)) = v.options.get(*i as usize) else {return None};
		let Some(path) = paths.iter().find(|v| v.0 == *sub_id) else {return None};
		match &path.1 {
			crate::modman::meta::ValuePathPath::Mod(path) => {
				return Some(path.to_owned())
			}
		}
	}
}

impl EnumTools for Path {
	type Iterator = std::array::IntoIter<Self, 3>;
	
	fn to_str(&self) -> &'static str {
		match self {
			Self::Mod(_) => "Mod",
			Self::Game(_) => "Game",
			Self::Option(_, _) => "Option",
		}
	}
	
	fn iter() -> Self::Iterator {
		[
			Self::Mod(String::new()),
			Self::Game(String::new()),
			Self::Option(String::new(), String::new()),
		].into_iter()
	}
}