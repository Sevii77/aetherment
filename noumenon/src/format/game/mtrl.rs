
use std::{fmt::Debug, io::{Read, Seek, Write}};
use binrw::{binrw, BinRead, BinWrite};

use crate::NullReader;

pub const EXT: &'static [&'static str] = &["mtrl"];

pub type Error = binrw::Error;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Mtrl {
	pub shader: String,
	pub uvsets: Vec<String>,
	pub colorsets: Vec<ColorSet>,
	pub constants: Vec<Constant>,
	pub samplers: Vec<Sampler>,
	pub shader_keys: Vec<(u32, u32)>,
	pub shader_flags: u32,
}

impl BinRead for Mtrl {
	type Args<'a> = ();
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<Self> {
		simple_reader!(reader, endian);
		
		let _version = r!(u32);
		let _file_size = r!(u16);
		let dataset_size = r!(u16);
		let strings_size = r!(u16);
		let shader_name_offset = r!(u16);
		let texture_count = r!(u8);
		let uvset_count = r!(u8);
		let colorset_count = r!(u8);
		let extra_data_size = r!(u8);
		
		let texture_infos = r!(Vec<(u16, u16)>, texture_count); // name offset, flags
		let uvset_infos = r!(Vec<(u16, u16)>, uvset_count); // name offset, index
		let colorset_infos = r!(Vec<(u16, u16)>, colorset_count); // name offset, index
		let strings = r!(Vec<u8>, strings_size);
		
		let _extra_data = r!(Vec<u8>, extra_data_size);
		
		let colorsets = if dataset_size >= 2048 {r!(Vec<[ColorRow; 32]>, colorset_count)} else {Vec::new()};
		let colorset_dyes = if dataset_size >= 2176 {r!(Vec<[u32; 32]>, colorset_count)} else {Vec::new()};
		
		let constant_values_size = r!(u16);
		let shader_key_count = r!(u16);
		let constant_count = r!(u16);
		let sampler_count = r!(u16);
		let shader_flags = r!(u32);
		
		let shader_keys = r!(Vec<(u32, u32)>, shader_key_count);
		let constants = r!(Vec<ConstantDefinitionRaw>, constant_count);
		let samplers = r!(Vec<SamplerRaw>, sampler_count);
		let constant_values = r!(Vec<u8>, constant_values_size);
		
		Ok(Self {
			shader: strings[shader_name_offset as usize..].null_terminated().unwrap(),
			uvsets: (0..uvset_count as usize)
				.map(|i| strings[uvset_infos.iter().find(|v| v.1 as usize == i).unwrap().0 as usize..].null_terminated().unwrap())
				.collect(),
			colorsets: colorsets
				.into_iter()
				.enumerate()
				.map(|(i, v)| ColorSet {
					name: strings[colorset_infos.iter().find(|v| v.1 as usize == i).unwrap().0 as usize..].null_terminated().unwrap(),
					regular: v,
					dyes: colorset_dyes.get(i).copied(),
				}).collect(),
			constants: constants
				.into_iter()
				.map(|v| Constant {
					id: v.id,
					value: constant_values[v.offset as usize..(v.offset + v.size) as usize].to_vec(),
				}).collect(),
			samplers: samplers
				.into_iter()
				.map(|v| Sampler {
					typ: v.typ,
					flags: v.flags,
					texture: strings[texture_infos[v.texture_id as usize].0 as usize..].null_terminated().unwrap(),
				}).collect(),
			shader_keys,
			shader_flags,
		})
	}
}

impl BinWrite for Mtrl {
	type Args<'a> = ();
	
	fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<()> {
		todo!();
	}
}

impl ironworks::file::File for Mtrl {
	fn read(mut data: impl ironworks::FileStream) -> Result<Self, ironworks::Error> {
		Mtrl::read_le(&mut data).map_err(|e| ironworks::Error::Resource(e.into()))
	}
}

impl crate::format::external::Bytes<Error> for Mtrl {
	fn read<T>(reader: &mut T) -> Result<Self, Error>
	where T: Read + Seek {
		Ok(Mtrl::read_le(reader)?)
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		self.write_le(writer)?;
		
		Ok(())
	}
}

// ----------

#[derive(Debug, Clone)]
pub struct Constant {
	pub id: u32,
	pub value: Vec<u8>,
}

impl Constant {
	/// Panics if T is not the same size as value byte count
	pub fn value_as<T: Sized>(&mut self) -> &mut T {
		assert!(size_of::<T>() == self.value.len(), "T is not the same size as value");
		unsafe{std::mem::transmute_copy(&mut self.value.as_ptr())}
	}
}

#[derive(Debug, Clone)]
pub struct Sampler {
	pub typ: u32,
	pub texture: String,
	pub flags: u32,
}

// Normal = 207536625,
// Mask = 2320401078,
// ColorsetIndex = 1449103320,
// Diffuse = 290653886,

// outdated old endwalker stuff, stuff is missing
// #[binrw]
// #[brw(little, repr = u32)]
// #[repr(u32)]
// #[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
// pub enum SamplerType {
// 	CharaNormal = 207536625, // chara/equipment/e6015/texture/v01_c0201e6015_sho_n.tex
// 	CharaDiffuse = 290653886, // chara/human/c0801/obj/face/f0003/texture/c0801f0003_fac_d.tex
// 	BgUnk465317650 = 465317650, // bg/ex4/02_mid_m5/twn/m5t1/texture/m5t1_k1_flor5_s.tex (some funky stuff idk)
// 	BgDiffuse = 510652316, // bg/ex4/02_mid_m5/twn/m5t1/texture/m5t1_k1_flor5_d.tex
// 	WaterUnk541659712 = 541659712, // bgcommon/nature/water/texture/_n_wavelet_000.tex (used by water shader together with 1464738518, so probably only uses certain layer(s))
// 	LiftShaftUnk557626425 = 557626425, // bgcommon/nature/lightshaft/texture/_n_lightshaft_s0_000.tex
// 	CharaSpecular = 731504677, // chara/monster/m0060/obj/body/b0001/texture/v01_m0060b0001_s.tex
// 	LiftShaftUnk1446741167 = 1446741167, // bgcommon/nature/lightshaft/texture/_n_lightshaft_s1_000.tex (noise layer shader by RGB, idk)
// 	WaterUnk1464738518 = 1464738518, // bgcommon/nature/water/texture/_n_wavelet_000.tex (see 541659712)
// 	BgUnk1768480522 = 1768480522, // bgcommon/texture/dummy_d.tex
// 	BgUnk1824202628 = 1824202628, // bgcommon/texture/dummy_s.tex
// 	WaterUnk2281064269 = 2281064269, // bg/ffxiv/est_e1/hou/e1h1/texture/e1h1_w1_wate1_f.tex (ironworks doesnt like it, but probably water splashes/fog/mist/the stuff at the bottom of waterfalls)
// 	Fog = 2285931524, // bgcommon/nature/verticalfog/texture/_n_verticalfog_000.tex (3 different noise layers for RGB)
// 	Multi = 2320401078, // chara/equipment/e6015/texture/v01_c0201e6015_sho_m.tex
// 	WaterUnk2514613837 = 2514613837, // bgcommon/nature/water/texture/_n_whitecap_000.tex
// 	BgNormal = 2863978985, // bg/ex4/02_mid_m5/twn/m5t1/texture/m5t1_k1_flor5_n.tex (this texture sucks but its normal for bg)
// 	BgUnk3719555455 = 3719555455, // bgcommon/texture/dummy_n.tex
// 	WaterNormal = 3862043388, // bgcommon/nature/water/texture/_n_wave_000.tex (normal but the alpha layer seems to have a different purpose)
// 	Cubemap = 4174878074, // bgcommon/nature/envmap/texture/_n_envmap_000.tex (ironworks fail, probably cubemap)
// 	Reflection = 4271961042, // chara/monster/m0536/obj/body/b0001/texture/v01_m0536b0001_d.tex (used by iris, also by ozma which uses the iris shader lol)
// }

#[derive(Debug, Clone)]
pub struct ColorSet {
	pub name: String,
	pub regular: [ColorRow; 32],
	pub dyes: Option<[u32; 32]>,
}

#[derive(Debug, Clone)]
pub struct ColorRow {
	pub diffuse: glam::Vec3,
	pub _diffuse_alpha: f32,
	pub specular: glam::Vec3,
	pub _specular_alpha: f32,
	pub emmisive: glam::Vec3,
	pub _emmisive_alpha: f32,
	pub sheen_rate: f32,
	pub sheen_tint_rate: f32,
	pub sheen_aperature: f32,
	pub _unknown15: f32,
	pub roughness: f32,
	pub _unknown17: f32,
	pub metalic: f32,
	pub anisotropy: f32,
	pub _unknown20: f32,
	pub sphere_map_mask: f32,
	pub _unknown22: f32,
	pub _unknown23: f32,
	pub shader_id: u16,
	pub tile_index: i16,
	pub tile_alpha: f32,
	pub sphere_map_index: u16,
	pub tile_transform: glam::Mat2,
}

impl Default for ColorRow {
	fn default() -> Self {
		Self {
			diffuse: glam::Vec3::ONE,
			_diffuse_alpha: 1.0,
			specular: glam::Vec3::ONE,
			_specular_alpha: 0.0,
			emmisive: glam::Vec3::ZERO,
			_emmisive_alpha: 1.0,
			sheen_rate: 0.1,
			sheen_tint_rate: 0.2,
			sheen_aperature: 5.0,
			_unknown15: 0.0,
			roughness: 0.0,
			_unknown17: 0.0,
			metalic: 0.0,
			anisotropy: 0.0,
			_unknown20: 0.0,
			sphere_map_mask: 0.0,
			_unknown22: 0.0,
			_unknown23: 0.0,
			shader_id: 0,
			tile_index: 0,
			tile_alpha: 1.0,
			sphere_map_index: 0,
			tile_transform: glam::Mat2::IDENTITY * 16.0,
		}
	}
}

impl BinRead for ColorRow {
	type Args<'a> = ();
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<Self> {
		simple_reader!(reader, endian);
		
		Ok(Self {
			diffuse: glam::vec3(r!(f16), r!(f16), r!(f16)),
			_diffuse_alpha: r!(f16),
			specular: glam::vec3(r!(f16), r!(f16), r!(f16)),
			_specular_alpha: r!(f16),
			emmisive: glam::vec3(r!(f16), r!(f16), r!(f16)),
			_emmisive_alpha: r!(f16),
			sheen_rate: r!(f16),
			sheen_tint_rate: r!(f16),
			sheen_aperature: r!(f16),
			_unknown15: r!(f16),
			roughness: r!(f16),
			_unknown17: r!(f16),
			metalic: r!(f16),
			anisotropy: r!(f16),
			_unknown20: r!(f16),
			sphere_map_mask: r!(f16),
			_unknown22: r!(f16),
			_unknown23: r!(f16),
			shader_id: r!(u16),
			tile_index: r!(i16),
			tile_alpha: r!(f16),
			sphere_map_index: r!(u16),
			tile_transform: glam::Mat2::from_cols_array(&[r!(f16), r!(f16), r!(f16), r!(f16)]),
		})
	}
}

impl BinWrite for ColorRow {
	type Args<'a> = ();
	
	fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<()> {
		todo!();
	}
}

// ----------

#[binrw]
#[derive(Debug, Clone)]
struct ConstantDefinitionRaw {
	id: u32,
	offset: u16,
	size: u16,
}

#[binrw]
#[derive(Debug, Clone)]
struct SamplerRaw {
	typ: u32,
	flags: u32,
	texture_id: u8,
	_padding: [u8; 3],
}