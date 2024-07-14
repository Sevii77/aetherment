use std::borrow::Cow;
use serde::{Deserialize, Serialize};
use crate::{modman::{settings::Value as SettingsValue, meta::OptionSettings, Path}, render_helper::EnumTools};

#[derive(Debug)]
pub enum CompositeError {
	NoFirstLayer,
	NoFileResolverReturn{layer: usize},
	Modifier{layer: usize, modifier: ModifierError},
}

impl From<CompositeError> for super::CompositeError {
	fn from(value: CompositeError) -> Self {
		super::CompositeError::Tex(value)
	}
}

#[derive(Debug)]
pub enum ModifierError {
	NoFileResolverReturn,
	CullPoint,
	Color,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Tex {
	pub layers: Vec<Layer>,
}

impl Tex {
	pub fn composite_raw_hashmap(&self, settings: &crate::modman::settings::CollectionSettings, textures: std::collections::HashMap<&Path, &noumenon::format::game::Tex>) -> Result<Vec<u8>, CompositeError> {
		self.composite_raw(settings, move |path| textures.get(path).map(|v| Cow::Borrowed(*v))).map(|v| v.2)
	}
	
	pub fn composite_raw<'a>(&'a self, settings: &crate::modman::settings::CollectionSettings, textures_handler: impl Fn(&Path) -> Option<Cow<'a, noumenon::format::game::Tex>>) -> Result<(u32, u32, Vec<u8>), CompositeError> {
		let mut layers = self.layers.iter().rev();
		
		let layer = layers.next().ok_or(CompositeError::NoFirstLayer)?;
		let tex = &textures_handler(&layer.path).ok_or(CompositeError::NoFileResolverReturn{layer: 0})?;
		let (width, height) = (tex.header.width as u32, tex.header.height as u32);
		let mut data = tex.data.clone();
		
		let apply_modifiers = |layer: &Layer, data: &mut [u8]| -> Result<(), ModifierError> {
			for modifier in layer.modifiers.iter().rev() {
				match modifier {
					Modifier::AlphaMask{path, cull_point} => {
						let cull_point = cull_point.get_value(settings).ok_or(ModifierError::CullPoint)?;
						let tex = &textures_handler(path).ok_or(ModifierError::NoFileResolverReturn)?;
						let (w, h) = (tex.header.width as u32, tex.header.height as u32);
						let mask_data = get_resized(tex, w, h, width, height);
						
						for (i, pixel) in data.chunks_exact_mut(4).enumerate() {
							if mask_data[i * 4] as f32 / 255.0 < cull_point {
								pixel[0] = 0;
								pixel[1] = 0;
								pixel[2] = 0;
								pixel[3] = 0;
							}
						}
					}
					
					Modifier::AlphaMaskAlphaStretch{path, cull_point} => {
						let cull_point = cull_point.get_value(settings).ok_or(ModifierError::CullPoint)?;
						let tex = &textures_handler(path).ok_or(ModifierError::NoFileResolverReturn)?;
						let (w, h) = (tex.header.width as u32, tex.header.height as u32);
						let mask_data = get_resized(tex, w, h, width, height);
						
						let mut lowest = 255;
						for (i, pixel) in data.chunks_exact_mut(4).enumerate() {
							if mask_data[i * 4] as f32 / 255.0 < cull_point {
								pixel[0] = 0;
								pixel[1] = 0;
								pixel[2] = 0;
								pixel[3] = 0;
							}
							
							if pixel[3] > 0 {
								lowest = lowest.min(pixel[3]);
							}
						}
						
						for pixel in data.chunks_exact_mut(4) {
							if pixel[3] > 0 {
								pixel[3] = ((pixel[3] - lowest) as f32 / (1 - lowest) as f32 * 255.0) as u8;
							}
						}
					}
					
					Modifier::Color{value} => {
						let color = value.get_value(settings).ok_or(ModifierError::Color)?;
						for pixel in data.chunks_exact_mut(4) {
							pixel[0] = (pixel[0] as f32 * color[0]).min(255.0) as u8;
							pixel[1] = (pixel[1] as f32 * color[1]).min(255.0) as u8;
							pixel[2] = (pixel[2] as f32 * color[2]).min(255.0) as u8;
							pixel[3] = (pixel[3] as f32 * color[3]).min(255.0) as u8;
						}
					}
				}
			}
			
			Ok(())
		};
		
		apply_modifiers(layer, &mut data).map_err(|modifier| CompositeError::Modifier{layer: 0, modifier})?;
		
		for (i, layer) in layers.enumerate() {
			let tex = &textures_handler(&layer.path).ok_or(CompositeError::NoFileResolverReturn{layer: i + 1})?;
			let (w, h) = (tex.header.width as u32, tex.header.height as u32);
			let mut layer_data = get_resized(tex, w, h, width, height);
			
			apply_modifiers(layer, &mut layer_data).map_err(|modifier| CompositeError::Modifier{layer: i + 1, modifier})?;
			
			let f = match layer.blend {
				Blend::Normal => |_a: f32, b: f32|
					b,
				
				Blend::Multiply => |a: f32, b: f32|
					a * b,
				
				Blend::Screen => |a: f32, b: f32|
					1.0 - (1.0 - a) * (1.0 - b),
				
				Blend::Overlay => |a: f32, b: f32| 
					if a < 0.5 {
						2.0 * a * b
					} else {
						1.0 - 2.0 * (1.0 - a) * (1.0 - b)
					},
				
				Blend::HardLight => |a: f32, b: f32| 
					if b < 0.5 {
						2.0 * a * b
					} else {
						1.0 - 2.0 * (1.0 - a) * (1.0 - b)
					},
				
				Blend::SoftLightPhotoshop => |a: f32, b: f32| 
					if b < 0.5 {
						2.0 * a * b + a * a * (1.0 - 2.0 * b)
					} else {
						2.0 * a * (1.0 - b) + a.sqrt() * (2.0 * b - 1.0)
					},
			};
			
			for (base_pixel, layer_pixel) in data.chunks_exact_mut(4).zip(layer_data.chunks_exact(4)) {
				if layer_pixel[3] > 0 {
					let base_a = base_pixel[3] as f32 / 255.0;
					let layer_a = layer_pixel[3] as f32 / 255.0;
					let a = layer_a + base_a * (1.0 - layer_a);
					
					base_pixel[0] = ((f(base_pixel[0] as f32 / 255.0, layer_pixel[0] as f32 / 255.0) * layer_a * 255.0 + base_pixel[0] as f32 * base_a * (1.0 - layer_a)) / a) as u8;
					base_pixel[1] = ((f(base_pixel[1] as f32 / 255.0, layer_pixel[1] as f32 / 255.0) * layer_a * 255.0 + base_pixel[1] as f32 * base_a * (1.0 - layer_a)) / a) as u8;
					base_pixel[2] = ((f(base_pixel[2] as f32 / 255.0, layer_pixel[2] as f32 / 255.0) * layer_a * 255.0 + base_pixel[2] as f32 * base_a * (1.0 - layer_a)) / a) as u8;
					base_pixel[3] = (a * 255.0) as u8;
				}
			}
		}
		
		Ok((width, height, data))
	}
}

impl super::Composite for Tex {
	fn get_files(&self) -> Vec<&str> {
		let mut files = Vec::new();
		for layer in &self.layers {
			if let Path::Mod(path) = &layer.path {
				files.push(path.as_str());
			}
			
			for modifier in &layer.modifiers {
				match modifier {
					Modifier::AlphaMask{path, ..} | Modifier::AlphaMaskAlphaStretch{path, ..} => {
						if let Path::Mod(path) = path {
							files.push(path.as_str());
						}
					}
					
					_ => {}
				}
			}
		}
		
		files
	}
	
	fn get_files_game(&self) -> Vec<&str> {
		let mut files = Vec::new();
		for layer in &self.layers {
			if let Path::Game(path) = &layer.path {
				files.push(path.as_str());
			}
			
			for modifier in &layer.modifiers {
				match modifier {
					Modifier::AlphaMask{path, ..} | Modifier::AlphaMaskAlphaStretch{path, ..} => {
						if let Path::Game(path) = path {
							files.push(path.as_str());
						}
					}
					
					_ => {}
				}
			}
		}
		
		files
	}
	
	fn get_options(&self) -> Vec<&str> {
		let mut options = Vec::new();
		for layer in &self.layers {
			if let Path::Option(v, _) = &layer.path {
				options.push(v.as_str())
			}
			
			for modifier in &layer.modifiers {
				let option = match modifier {
					Modifier::AlphaMask{path, cull_point} | Modifier::AlphaMaskAlphaStretch{path, cull_point} => {
						if let Path::Option(v, _) = path {Some(v.as_str())} else {
						if let OptionOrStatic::Option(v) = cull_point {Some(v.option_id())} else {None}}
					}
					
					Modifier::Color{value} => {
						if let OptionOrStatic::Option(v) = value {Some(v.option_id())} else {None}
					}
				};
				
				if let Some(option) = option {
					options.push(option);
				}
			}
		}
		
		options
	}
	
	fn composite<'a>(&self, settings: &crate::modman::settings::CollectionSettings, file_resolver: &dyn Fn(&crate::modman::Path) -> Option<Cow<'a, Vec<u8>>>) -> Result<Vec<u8>, super::CompositeError> {
		let textures_handler = |path: &crate::modman::Path| -> Option<Cow<noumenon::format::game::Tex>> {
			Some(Cow::Owned(noumenon::format::game::Tex::read(&mut std::io::Cursor::new(file_resolver(path)?.as_ref())).ok()?))
		};
		
		let (width, height, pixels) = self.composite_raw(settings, textures_handler)?;
		
		let mut data = std::io::Cursor::new(Vec::new());
		noumenon::format::game::Tex {
			header: noumenon::format::game::tex::Header {
				flags: 0x00800000,
				format: noumenon::format::game::tex::Format::A8R8G8B8,
				width: width as u16,
				height: height as u16,
				depths: 0,
				mip_levels: 1,
				lod_offsets: [0, 1, 2],
				mip_offsets: [80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			},
			data: pixels,
		}.write(&mut data)?;
		
		Ok(data.into_inner())
	}
}

fn get_resized(tex: &noumenon::format::game::Tex, width: u32, height: u32, target_width: u32, target_height: u32) -> Vec<u8> {
	if width != target_width || height != target_height {
		image::imageops::resize(tex, target_width, target_height, image::imageops::FilterType::Nearest).into_vec()
	} else {
		tex.data.clone()
	}
}

// ----------

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Layer {
	pub name: String,
	pub path: Path,
	pub modifiers: Vec<Modifier>,
	pub blend: Blend,
}

impl std::hash::Hash for Layer {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.name.hash(state);
		self.path.hash(state);
		self.blend.hash(state);
	}
}

// ----------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Blend {
	Normal,
	Multiply,
	Screen,
	Overlay,
	HardLight,
	SoftLightPhotoshop,
}

impl EnumTools for Blend {
	type Iterator = std::array::IntoIter<Self, 6>;
	
	fn to_str(&self) -> &'static str {
		match self {
			Self::Normal => "Normal",
			Self::Multiply => "Multiply",
			Self::Screen => "Screen",
			Self::Overlay => "Overlay",
			Self::HardLight => "Hard Light",
			Self::SoftLightPhotoshop => "Soft Light (Photoshop)",
		}
	}
	
	fn iter() -> Self::Iterator {
		[
			Self::Normal,
			Self::Multiply,
			Self::Screen,
			Self::Overlay,
			Self::HardLight,
			Self::SoftLightPhotoshop
		].into_iter()
	}
}

// ----------

#[derive(Debug, Clone, PartialEq, Hash, Deserialize, Serialize)]
pub enum Modifier {
	/// Culls pixels based on the red channel of the mask texture.
	AlphaMask {
		path: Path,
		cull_point: OptionOrStatic<MaskOption>,
	},
	
	/// Culls pixels based on the red channel of the mask texture, then stretches the alpha channel of the texture.
	AlphaMaskAlphaStretch {
		path: Path,
		cull_point: OptionOrStatic<MaskOption>,
	},
	
	/// Multiplies the color channels of the texture by the color.
	Color {
		value: OptionOrStatic<ColorOption>,
	}
}

impl EnumTools for Modifier {
	type Iterator = std::array::IntoIter<Self, 3>;
	
	fn to_str(&self) -> &'static str {
		match self {
			Self::AlphaMask{..} => "Alpha Mask",
			Self::AlphaMaskAlphaStretch{..} => "Alpha Mask (Alpha Stretch)",
			Self::Color{..} => "Color",
		}
	}
	
	fn iter() -> Self::Iterator {
		[
			Self::AlphaMask{path: Path::Mod(String::new()), cull_point: OptionOrStatic::Static(1.0)},
			Self::AlphaMaskAlphaStretch{path: Path::Mod(String::new()), cull_point: OptionOrStatic::Static(1.0)},
			Self::Color{value: OptionOrStatic::Static([1.0, 1.0, 1.0, 1.0])},
		].into_iter()
	}
}

// ----------

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum OptionOrStatic<T: OptionSetting + Sized + Default> {
	Option(T),
	Static(T::Value),
}

impl<T: OptionSetting + Sized + Default> OptionOrStatic<T> {
	pub fn get_value(&self, settings: &crate::modman::settings::CollectionSettings) -> Option<T::Value> {
		match self {
			Self::Static(v) => Some(v.clone()),
			Self::Option(t) => t.get_value(settings.get(t.option_id())?),
		}
	}
}

// this is NOT a proper hash!!! it only hashes the pointer of the value so it can be used in drag n drop elements
impl<T: OptionSetting + Sized + Default> std::hash::Hash for OptionOrStatic<T> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		match self {
			Self::Option(t) => (t as *const _ as usize).hash(state),
			Self::Static(t) => (t as *const _ as usize).hash(state),
		}
	}
}

impl<T: OptionSetting + Sized + Default> EnumTools for OptionOrStatic<T> {
	type Iterator = std::array::IntoIter<Self, 2>;
	
	fn to_str(&self) -> &'static str {
		match self {
			Self::Option(_) => "Option",
			Self::Static(_) => "Static",
		}
	}
	
	fn iter() -> Self::Iterator {
		[
			Self::Option(T::default()),
			Self::Static(T::Value::default()),
		].into_iter()
	}
}

// ----------

pub trait OptionSetting {
	type Value: Clone + Default + PartialEq;
	
	fn option_id(&self) -> &str;
	fn option_id_mut(&mut self) -> &mut String;
	fn get_value(&self, settings_value: &SettingsValue) -> Option<Self::Value>;
	fn valid_option(&self, option: &OptionSettings) -> bool;
}

// ----------

#[derive(Debug, Clone, Default, PartialEq, Hash, Deserialize, Serialize)]
pub struct ColorOption(pub String);
impl OptionSetting for ColorOption {
	type Value = [f32; 4];
	
	fn option_id(&self) -> &str {
		&self.0
	}
	
	fn option_id_mut(&mut self) -> &mut String {
		&mut self.0
	}
	
	fn get_value(&self, settings_value: &SettingsValue) -> Option<Self::Value> {
		match settings_value {
			SettingsValue::Rgba(v) => Some(*v),
			SettingsValue::Rgb(v) => Some([v[0], v[1], v[2], 1.0]),
			SettingsValue::Grayscale(v) => Some([*v, *v, *v, 1.0]),
			SettingsValue::Opacity(v) => Some([1.0, 1.0, 1.0, *v]),
			_ => None,
		}
	}
	
	fn valid_option(&self, option: &OptionSettings) -> bool {
		match option {
			OptionSettings::Rgb(_) |
			OptionSettings::Rgba(_) |
			OptionSettings::Grayscale(_) |
			OptionSettings::Opacity(_) => true,
			_ => false,
		}
	}
}

// ----------

#[derive(Debug, Clone, Default, PartialEq, Hash, Deserialize, Serialize)]
pub struct MaskOption(pub String);
impl OptionSetting for MaskOption {
	type Value = f32;
	
	fn option_id(&self) -> &str {
		&self.0
	}
	
	fn option_id_mut(&mut self) -> &mut String {
		&mut self.0
	}
	
	fn get_value(&self, settings_value: &SettingsValue) -> Option<Self::Value> {
		match settings_value {
			SettingsValue::Mask(v) => Some(*v),
			_ => None,
		}
	}
	
	fn valid_option(&self, option: &OptionSettings) -> bool {
		match option {
			OptionSettings::Mask(_) => true,
			_ => false,
		}
	}
}