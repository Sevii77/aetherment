use std::{collections::HashMap, fmt::Debug, io::{Read, Seek, Write}};
use binrw::{binrw, BinRead, BinWrite};
use crate::NullReader;

pub const EXT: &'static [&'static str] = &["mtrl"];

pub type Error = binrw::Error;

pub fn shader_param_name(id: u32) -> Option<String> {
	let names = std::cell::LazyCell::new(|| {
		let names = [
			// https://github.com/Ottermandias/Penumbra.GameData/blob/main/Files/ShaderStructs/Names.cs\
			"g_SphereMapIndex",
			"g_TileIndex",
			"g_AlphaAperture",
			"g_AlphaMultiParam",
			"g_AlphaOffset",
			"g_AlphaThreshold",
			"g_AmbientOcclusionMask",
			"g_AngleClip",
			"g_BackScatterPower",
			"g_Color",
			"g_ColorUVScale",
			"g_DetailColor",
			"g_DetailColorUvScale",
			"g_DetailID",
			"g_DetailNormalScale",
			"g_DiffuseColor",
			"g_EmissiveColor",
			"g_EnvMapPower",
			"g_FarClip",
			"g_Fresnel",
			"g_FresnelValue0",
			"g_FurLength",
			"g_GlassIOR",
			"g_Gradation",
			"g_HairBackScatterRoughnessOffsetRate",
			"g_HairScatterColorShift",
			"g_HairSecondaryRoughnessOffsetRate",
			"g_HairSpecularBackScatterShift",
			"g_HairSpecularPrimaryShift",
			"g_HairSpecularSecondaryShift",
			"g_HeightMapScale",
			"g_HeightScale",
			"g_InclusionAperture",
			"g_Intensity",
			"g_IrisOptionColorRate",
			"g_IrisRingColor",
			"g_IrisRingEmissiveIntensity",
			"g_IrisRingForceColor",
			"g_IrisThickness",
			"g_LayerColor",
			"g_LayerDepth",
			"g_LayerIrregularity",
			"g_LayerScale",
			"g_LayerVelocity",
			"g_LightingType",
			"g_LipFresnelValue0",
			"g_LipRoughnessScale",
			"g_LipShininess",
			"g_MultiDetailColor",
			"g_MultiDiffuseColor",
			"g_MultiEmissiveColor",
			"g_MultiHeightScale",
			"g_MultiNormalScale",
			"g_MultiSpecularColor",
			"g_MultiSSAOMask",
			"g_MultiWaveScale",
			"g_MultiWhitecapDistortion",
			"g_MultiWhitecapScale",
			"g_NearClip",
			"g_NormalScale",
			"g_NormalScale1",
			"g_NormalUVScale",
			"g_OutlineColor",
			"g_OutlineWidth",
			"g_PrefersFailure",
			"g_Ray",
			"g_ReflectionPower",
			"g_RefractionColor",
			"g_ScatteringLevel",
			"g_ShaderID",
			"g_ShadowAlphaThreshold",
			"g_ShadowOffset",
			"g_ShadowPosOffset",
			"g_SheenAperture",
			"g_SheenRate",
			"g_SheenTintRate",
			"g_Shininess",
			"g_SpecularColor",
			"g_SpecularColorMask",
			"g_SpecularMask",
			"g_SpecularPower",
			"g_SpecularUVScale",
			"g_SSAOMask",
			"g_SubSurfacePower",
			"g_SubSurfaceProfileID",
			"g_SubSurfaceWidth",
			"g_TexAnim",
			"g_TextureMipBias",
			"g_TexU",
			"g_TexV",
			"g_TileAlpha",
			"g_TileScale",
			"g_ToonIndex",
			"g_ToonLightScale",
			"g_ToonReflectionScale",
			"g_ToonSpecIndex",
			"g_Transparency",
			"g_TransparencyDistance",
			"g_UseSubSurfaceRate",
			"g_WaveletDistortion",
			"g_WaveletNoiseParam",
			"g_WaveletOffset",
			"g_WaveletScale",
			"g_WaveSpeed",
			"g_WaveTime",
			"g_WaveTime1",
			"g_WhitecapColor",
			"g_WhitecapDistance",
			"g_WhitecapDistortion",
			"g_WhitecapNoiseScale",
			"g_WhitecapScale",
			"g_WhitecapSpeed",
			"g_WhiteEyeColor",
			
			// directly from shpk files
			"g_AmbientExtra",
			"g_AmbientParam",
			"g_AmbientParamArray",
			"g_AnimSampler",
			"g_AuraParam",
			"g_BGAmbientParameter",
			"g_BGSelectionModelCommonParameter",
			"g_BGSelectionModelParameter",
			"g_BushInstancingData",
			"g_BushNoInstancingData",
			"g_CameraParameter",
			"g_CloudShadowMatrix",
			"g_CloudShadowSampler",
			"g_CommonParameter",
			"g_CompositeCommonSampler",
			"g_ConnectionVertex",
			"g_CustomizeParameter",
			"g_DecalColor",
			"g_DecalParameter",
			"g_DirectionalShadowParameter",
			"g_DissolveParam",
			"g_DissolveSampler",
			"g_DofLutSampler",
			"g_DynamicWaveCompostInfo",
			"g_DynamicWaveObjectParams",
			"g_DynamicWaveTypeInfo",
			"g_EadgBias",
			"g_FakeSpecularParam",
			"g_FogParameter",
			"g_FogWeightLutSampler",
			"g_GeometryParam",
			"g_GlassOffscreenParam",
			"g_GrassCommonParam",
			"g_GrassGridParam",
			"g_InputConnectionVertex",
			"g_InputConnectionVertexPrev",
			"g_InstanceData",
			"g_InstanceParameter",
			"g_InstancingData",
			"g_InstancingMatrix",
			"g_JointMatrixArray",
			"g_JointMatrixArrayPrev",
			"g_LightDirection",
			"g_LightDrawParam",
			"g_LightParam",
			"g_MaterialParam",
			"g_MaterialParameter",
			"g_MaterialParameterDynamic",
			"g_ModelParameter",
			"g_OmniShadowParam",
			"g_PSParam",
			"g_PS_DecalSpecificParameters",
			"g_PS_DocumentParameters",
			"g_PS_DofCocParam",
			"g_PS_InstanceExtraParameters",
			"g_PS_ModelLightParameters",
			"g_PS_ModelSpecificParameters",
			"g_PS_Parameters",
			"g_PS_ShadowDistance",
			"g_PS_ShadowParameters",
			"g_PS_UvTransform",
			"g_PS_ViewProjectionInverseMatrix",
			"g_Parameter",
			"g_PbrParameterCommon",
			"g_PlateEadg",
			"g_PrevInstancingMatrix",
			"g_PreviousInstanceData",
			"g_PreviousInstancingData",
			"g_PreviousWavingParam",
			"g_PreviousWindParam",
			"g_RoofMatrix",
			"g_RoofParameter",
			"g_RoofProjectionMatrix",
			"g_RoofSampler",
			"g_Sampler",
			"g_Sampler0",
			"g_Sampler1",
			"g_SamplerAttenuation",
			"g_SamplerAuraTexture",
			"g_SamplerAuraTexture1",
			"g_SamplerAuraTexture2",
			"g_SamplerCatchlight",
			"g_SamplerCaustics",
			"g_SamplerCharaToon",
			"g_SamplerColor1",
			"g_SamplerColor2",
			"g_SamplerColor3",
			"g_SamplerColor4",
			"g_SamplerColorMap",
			"g_SamplerColorMap0",
			"g_SamplerColorMap1",
			"g_SamplerDecal",
			"g_SamplerDepth",
			"g_SamplerDepthWithWater",
			"g_SamplerDetailColorMap",
			"g_SamplerDetailNormalMap",
			"g_SamplerDiffuse",
			"g_SamplerDissolveTexture",
			"g_SamplerDissolveTexture1",
			"g_SamplerDistortion",
			"g_SamplerDither",
			"g_SamplerDynamicWave",
			"g_SamplerDynamicWavePrev",
			"g_SamplerDynamicWavePrev2",
			"g_SamplerEnvMap",
			"g_SamplerFinalColor",
			"g_SamplerFlow",
			"g_SamplerFresnel",
			"g_SamplerGBuffer",
			"g_SamplerGBuffer1",
			"g_SamplerGBuffer2",
			"g_SamplerGBuffer3",
			"g_SamplerGBuffer4",
			"g_SamplerGahter",
			"g_SamplerGradationMap",
			"g_SamplerIndex",
			"g_SamplerLight",
			"g_SamplerLightDiffuse",
			"g_SamplerLightSpecular",
			"g_SamplerMask",
			"g_SamplerNoise",
			"g_SamplerNormal",
			"g_SamplerNormal2",
			"g_SamplerNormalMap",
			"g_SamplerNormalMap0",
			"g_SamplerNormalMap1",
			"g_SamplerOcclusion",
			"g_SamplerOmniShadowDynamic",
			"g_SamplerOmniShadowIndexTable",
			"g_SamplerOmniShadowStatic",
			"g_SamplerPalette",
			"g_SamplerRawDynamicWave",
			"g_SamplerReflection",
			"g_SamplerReflectionArray",
			"g_SamplerReflectionMap",
			"g_SamplerReflection_",
			"g_SamplerRefractionMap",
			"g_SamplerShadow",
			"g_SamplerShadowMask",
			"g_SamplerSkinDiffuse",
			"g_SamplerSkinMask",
			"g_SamplerSkinNormal",
			"g_SamplerSpecularMap",
			"g_SamplerSpecularMap0",
			"g_SamplerSpecularMap1",
			"g_SamplerSphareMapCustum",
			"g_SamplerSphereMap",
			"g_SamplerSubsurfaceKernel",
			"g_SamplerTable",
			"g_SamplerTileNormal",
			"g_SamplerTileOrb",
			"g_SamplerToneMapLut",
			"g_SamplerVPosition",
			"g_SamplerViewPosition",
			"g_SamplerWaveMap",
			"g_SamplerWaveMap1",
			"g_SamplerWaveletMap0",
			"g_SamplerWaveletMap1",
			"g_SamplerWaveletNoise",
			"g_SamplerWhitecapMap",
			"g_SamplerWind0",
			"g_SamplerWind1",
			"g_SamplerWrinklesMask",
			"g_ScreenParameter",
			"g_ShaderTypeParameter",
			"g_ShadingParameters",
			"g_ShadowBiasParameter",
			"g_ShadowMaskParameter",
			"g_ShapeDeformIndex",
			"g_ShapeDeformParam",
			"g_ShapeDeformVertex",
			"g_SkinMaterialParameter",
			"g_SkySampler",
			"g_ToneMapParameter",
			"g_ToneMapSampler",
			"g_UnderWaterParam",
			"g_VSParam",
			"g_VS_PerInstanceParameters",
			"g_VS_ViewMatrix",
			"g_VS_ViewProjectionMatrix",
			"g_WaterParameter",
			"g_WavingParam",
			"g_WetnessParameter",
			"g_WindInfo",
			"g_WindParam",
			"g_WorldMatrix",
			"g_WorldViewMatrix",
			"g_WorldViewProjMatrix",
			"g_WrinklessWeightRate",
		];
		
		names.into_iter()
			.map(|v| (crate::crc32(v.as_bytes()), v))
			.collect::<HashMap<_, _>>()
	});
	
	names.get(&id).map(|v| v.to_string())
}

pub const USED_SAMPLERS: [u32; 24] = [
	1768480522,
	207536625,
	2816579574,
	4271961042,
	731504677,
	2320401078,
	3719555455,
	3862043388,
	2285931524,
	3845360663,
	557626425,
	1446741167,
	3615719374,
	1449103320,
	290653886,
	541659712,
	1464738518,
	465317650,
	510652316,
	2863978985,
	2514613837,
	4174878074,
	2281064269,
	1824202628,
];

pub const USED_SHADERS: [&'static str; 23] = [
	"bgprop.shpk",
	"bguvscroll.shpk",
	"verticalfog.shpk",
	"crystal.shpk",
	"characterstockings.shpk",
	"character.shpk",
	"characterinc.shpk",
	"bgcolorchange.shpk",
	"skin.shpk",
	"characterglass.shpk",
	"lightshaft.shpk",
	"characterlegacy.shpk",
	"water.shpk",
	"charactertattoo.shpk",
	"bg.shpk",
	"characterscroll.shpk",
	"characterocclusion.shpk",
	"charactertransparency.shpk",
	"characterreflection.shpk",
	"bgcrestchange.shpk",
	"hair.shpk",
	"iris.shpk",
	"river.shpk",
];

pub const USED_SHADER_SAMPLERS: &[(&'static str, &'static [u32])] = &[
	("bgprop.shpk", &[
		510652316,
		2863978985,
		465317650,
	]),
	("bguvscroll.shpk", &[
		510652316,
		1768480522,
		465317650,
		2863978985,
		3719555455,
		1824202628,
	]),
	("verticalfog.shpk", &[
		2285931524,
	]),
	("crystal.shpk", &[
		3615719374,
		465317650,
		510652316,
		4174878074,
		2863978985,
	]),
	("characterstockings.shpk", &[
		1449103320,
		207536625,
		2320401078,
	]),
	("character.shpk", &[
		731504677,
		207536625,
		1449103320,
		2816579574,
		290653886,
		2320401078,
	]),
	("characterinc.shpk", &[
		207536625,
		2320401078,
		290653886,
		1449103320,
	]),
	("bgcolorchange.shpk", &[
		510652316,
		465317650,
		2863978985,
	]),
	("skin.shpk", &[
		207536625,
		2320401078,
		290653886,
	]),
	("characterglass.shpk", &[
		207536625,
		1449103320,
		2320401078,
		290653886,
	]),
	("lightshaft.shpk", &[
		557626425,
		1446741167,
	]),
	("characterlegacy.shpk", &[
		290653886,
		207536625,
		2320401078,
		1449103320,
	]),
	("water.shpk", &[
		1464738518,
		541659712,
		2514613837,
		2281064269,
		3862043388,
		3845360663,
	]),
	("charactertattoo.shpk", &[
		207536625,
	]),
	("bg.shpk", &[
		465317650,
		2863978985,
		510652316,
		1768480522,
		1824202628,
		3719555455,
	]),
	("characterscroll.shpk", &[
		2816579574,
		207536625,
		290653886,
		2320401078,
		1449103320,
		4271961042,
	]),
	("characterocclusion.shpk", &[
		207536625,
	]),
	("charactertransparency.shpk", &[
		207536625,
		2320401078,
		1449103320,
		290653886,
	]),
	("characterreflection.shpk", &[
		207536625,
		4271961042,
		2320401078,
	]),
	("bgcrestchange.shpk", &[
		2863978985,
		1768480522,
		510652316,
		465317650,
	]),
	("hair.shpk", &[
		2320401078,
		207536625,
	]),
	("iris.shpk", &[
		2320401078,
		290653886,
		207536625,
	]),
	("river.shpk", &[
		3862043388,
		2281064269,
		2514613837,
	]),
];

// ----------

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
		let colorset_dyes = if dataset_size >= 2176 {r!(Vec<[ColorDyeRow; 32]>, colorset_count)} else {Vec::new()};
		
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
					dyes: colorset_dyes.get(i).map(|v| v.to_owned()),
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
					id: v.id,
					texture: if v.texture_id == 255 {String::new()} else {strings[texture_infos[v.texture_id as usize].0 as usize..].null_terminated().unwrap()},
					u_address_mode: (v.flags & 0x3).into(),
					v_address_mode: (v.flags >> 2 & 0x3).into(),
					lod_bias: ((v.flags as i32) << 12 >> 22) as f32 / 64.0,
					min_lod: v.flags >> 20 & 0xF,
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

impl crate::format::external::Bytes for Mtrl {
	fn read<T>(reader: &mut T) -> Result<Self, crate::Error>
	where T: Read + Seek {
		Ok(Mtrl::read_le(reader)?)
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), crate::Error> where
	T: Write + Seek {
		self.write_le(writer)?;
		
		Ok(())
	}
}

impl super::Extension for Mtrl {
	const EXT: &[&str] = EXT;
}

// ----------

#[derive(Debug, Clone)]
pub struct Constant {
	pub id: u32,
	pub value: Vec<u8>,
}

impl Constant {
	pub fn value_as<T: bytemuck::NoUninit + bytemuck::AnyBitPattern>(&mut self) -> &mut [T] {
		bytemuck::cast_slice_mut(&mut self.value)
	}
}

#[derive(Debug, Clone)]
pub struct Sampler {
	pub id: u32, 
	pub texture: String,
	pub u_address_mode: AddressMode,
	pub v_address_mode: AddressMode,
	pub lod_bias: f32,
	pub min_lod: u32,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressMode {
	Wrap   = 0,
	Mirror = 1,
	Clamp  = 2,
	Border = 3,
}

impl From<u32> for AddressMode {
	fn from(value: u32) -> Self {
		match value {
			0 => Self::Wrap,
			1 => Self::Mirror,
			2 => Self::Clamp,
			3 => Self::Border,
			_ => unreachable!()
		}
	}
}

#[derive(Debug, Clone)]
pub struct ColorSet {
	pub name: String,
	pub regular: [ColorRow; 32],
	pub dyes: Option<[ColorDyeRow; 32]>,
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
	pub tile_index: u16,
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
			tile_index: (r!(f16) * 64.0) as u16,
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

#[derive(Debug, Clone)]
pub struct ColorDyeRow {
	pub template: u16,
	pub channel: u8,
	pub diffuse: bool,
	pub specular: bool,
	pub emmisive: bool,
	pub scalar3: bool,
	pub metalic: bool,
	pub roughness: bool,
	pub sheen_rate: bool,
	pub sheen_tint_rate: bool,
	pub sheen_aperature: bool,
	pub anisotropy: bool,
	pub sphere_map_index: bool,
	pub sphere_map_mask: bool,
}

// impl Default for ColorDyeRow {
// 	fn default() -> Self {
// 		Self {
// 			
// 		}
// 	}
// }

impl BinRead for ColorDyeRow {
	type Args<'a> = ();
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<Self> {
		let data = u32::read_options(reader, endian, ())?;
		
		Ok(Self {
			template: (data >> 16 & 0x7FF) as u16,
			channel: (data >> 27 & 0x3) as u8,
			diffuse: (data & 0x0001) != 0,
			specular: (data & 0x0002) != 0,
			emmisive: (data & 0x0004) != 0,
			scalar3: (data & 0x0008) != 0,
			metalic: (data & 0x0010) != 0,
			roughness: (data & 0x0020) != 0,
			sheen_rate: (data & 0x0040) != 0,
			sheen_tint_rate: (data & 0x0080) != 0,
			sheen_aperature: (data & 0x0100) != 0,
			anisotropy: (data & 0x0200) != 0,
			sphere_map_index: (data & 0x0400) != 0,
			sphere_map_mask: (data & 0x0800) != 0,
		})
	}
}

impl BinWrite for ColorDyeRow {
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
	id: u32,
	flags: u32,
	texture_id: u8,
	_padding: [u8; 3],
}