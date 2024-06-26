#![allow(dead_code)]

// TODO: rewrite the reader/writer to use Binread/Binwrite

use std::{borrow::Cow, io::{Cursor, Read, Seek, Write}, collections::BTreeMap};
use binrw::{binrw, BinRead, BinWrite};
use half::f16;
use crate::{Error, NullReader, format::game::Result};

pub const EXT: &'static [&'static str] = &["mtrl"];

#[derive(Clone, Debug)]
pub struct ShaderParams {
	pub params: BTreeMap<ShaderParamId, ShaderParam>,
	pub keys: BTreeMap<ShaderKeyId, ShaderKey>,
	pub samplers: BTreeMap<SamplerType, Sampler>,
	pub unk: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct ShaderParam {
	pub enabled: bool,
	pub vals: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct ShaderKey {
	pub enabled: bool,
	pub val: u32,
}

#[derive(Clone, Debug)]
pub struct Sampler {
	pub enabled: bool,
	pub path: String,
	pub flags: u32,
}

// all shader param ids and the shaders its used in
#[binrw]
#[brw(little, repr = u32)]
#[repr(u32)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShaderParamId {
	Unk133014596 = 133014596, // bg.shpk
	Unk167817377 = 167817377, // verticalfog.shpk
	Unk213950055 = 213950055, // water.shpk, river.shpk
	Unk268000012 = 268000012, // water.shpk
	Unk289124319 = 289124319, // river.shpk
	Unk309952860 = 309952860, // water.shpk
	Unk337060565 = 337060565, // bgcolorchange.shpk, bg.shpk, crystal.shpk, bguvscroll.shpk, bgcrestchange.shpk
	Unk349757757 = 349757757, // lightshaft.shpk
	Unk364318261 = 364318261, // hair.shpk, character.shpk, characterglass.shpk
	Unk371521601 = 371521601, // river.shpk, water.shpk
	Unk390837838 = 390837838, // skin.shpk
	Unk396699942 = 396699942, // lightshaft.shpk
	Unk574239529 = 574239529, // water.shpk, river.shpk
	Unk699138595 = 699138595, // bg.shpk, bguvscroll.shpk, character.shpk, iris.shpk, skin.shpk, hair.shpk
	Unk704260801 = 704260801, // water.shpk, river.shpk
	Unk710394828 = 710394828, // water.shpk, river.shpk
	Unk740963549 = 740963549, // bg.shpk, bguvscroll.shpk, hair.shpk, bgcrestchange.shpk, iris.shpk, bgcolorchange.shpk, skin.shpk, crystal.shpk
	Unk778088561 = 778088561, // skin.shpk
	Unk792844182 = 792844182, // water.shpk
	Unk817435417 = 817435417, // bg.shpk
	Unk824928705 = 824928705, // river.shpk
	Unk876196728 = 876196728, // water.shpk
	Unk903613295 = 903613295, // verticalfog.shpk
	Unk906496720 = 906496720, // hair.shpk, character.shpk, iris.shpk, skin.shpk
	Unk926302173 = 926302173, // river.shpk
	Unk950420322 = 950420322, // bgcolorchange.shpk, bgcrestchange.shpk, bguvscroll.shpk, bg.shpk, skin.shpk, crystal.shpk
	Unk1066058257 = 1066058257, // bguvscroll.shpk, bg.shpk
	Unk1082825950 = 1082825950, // water.shpk, river.shpk
	Unk1112929012 = 1112929012, // skin.shpk
	Unk1139120744 = 1139120744, // bg.shpk, bguvscroll.shpk
	Unk1255709111 = 1255709111, // water.shpk
	Unk1407730043 = 1407730043, // water.shpk, river.shpk
	Unk1465565106 = 1465565106, // skin.shpk, character.shpk, hair.shpk, iris.shpk, characterglass.shpk
	Unk1495703619 = 1495703619, // lightshaft.shpk
	Unk1529014372 = 1529014372, // river.shpk
	Unk1536774237 = 1536774237, // water.shpk
	Unk1562817122 = 1562817122, // water.shpk
	Unk1617630358 = 1617630358, // iris.shpk
	Unk1627729957 = 1627729957, // water.shpk
	Unk1659128399 = 1659128399, // iris.shpk, skin.shpk
	Unk1714175910 = 1714175910, // iris.shpk
	Unk1886515549 = 1886515549, // water.shpk, river.shpk
	Unk1910233729 = 1910233729, // lightshaft.shpk
	Unk1914183202 = 1914183202, // verticalfog.shpk
	Unk2033894819 = 2033894819, // bg.shpk, bguvscroll.shpk
	Unk2189155593 = 2189155593, // lightshaft.shpk
	Unk2262174904 = 2262174904, // bguvscroll.shpk, bg.shpk
	Unk2274043692 = 2274043692, // skin.shpk
	Unk2365826946 = 2365826946, // bguvscroll.shpk, bg.shpk
	Unk2394542758 = 2394542758, // water.shpk, river.shpk
	Unk2408251504 = 2408251504, // bgcolorchange.shpk, bg.shpk, bguvscroll.shpk, bgcrestchange.shpk
	Unk2456716813 = 2456716813, // bg.shpk, bgcolorchange.shpk, bguvscroll.shpk, bgcrestchange.shpk
	Unk2471513915 = 2471513915, // river.shpk
	Unk2494828270 = 2494828270, // verticalfog.shpk
	Unk2516865232 = 2516865232, // river.shpk
	Unk2548576516 = 2548576516, // river.shpk
	Unk2569562539 = 2569562539, // river.shpk, water.shpk, hair.shpk, iris.shpk, skin.shpk
	Unk2590599703 = 2590599703, // bguvscroll.shpk
	Unk2615686474 = 2615686474, // water.shpk
	Unk2736828825 = 2736828825, // river.shpk, water.shpk
	Unk2750039980 = 2750039980, // water.shpk
	Unk2781883474 = 2781883474, // bg.shpk, bguvscroll.shpk
	Unk2784956570 = 2784956570, // water.shpk
	Unk2838061039 = 2838061039, // verticalfog.shpk
	Unk2858904847 = 2858904847, // bg.shpk, bguvscroll.shpk
	Unk3007164738 = 3007164738, // river.shpk
	Unk3036724004 = 3036724004, // skin.shpk, bg.shpk, hair.shpk, iris.shpk, character.shpk
	Unk3042205627 = 3042205627, // hair.shpk, bgcolorchange.shpk, bguvscroll.shpk, water.shpk, crystal.shpk, bgcrestchange.shpk, bg.shpk, river.shpk, characterglass.shpk
	Unk3086627810 = 3086627810, // bgcrestchange.shpk, bgcolorchange.shpk, bguvscroll.shpk, bg.shpk
	Unk3111546299 = 3111546299, // water.shpk, river.shpk
	Unk3122018048 = 3122018048, // water.shpk, river.shpk
	Unk3147419510 = 3147419510, // bguvscroll.shpk, bg.shpk
	Unk3166335201 = 3166335201, // verticalfog.shpk
	Unk3217843714 = 3217843714, // verticalfog.shpk
	Unk3219771693 = 3219771693, // crystal.shpk, bgcrestchange.shpk, bguvscroll.shpk, bgcolorchange.shpk, bg.shpk
	Unk3224367609 = 3224367609, // lightshaft.shpk
	Unk3264604780 = 3264604780, // river.shpk, water.shpk
	Unk3494687889 = 3494687889, // verticalfog.shpk
	Unk3531364537 = 3531364537, // lightshaft.shpk, verticalfog.shpk
	Unk3541428008 = 3541428008, // river.shpk, water.shpk
	Unk3593234462 = 3593234462, // water.shpk
	Unk3653987228 = 3653987228, // water.shpk, river.shpk
	Unk3819586170 = 3819586170, // river.shpk, water.shpk
	Unk3838218227 = 3838218227, // river.shpk
	Unk4009059935 = 4009059935, // crystal.shpk
	Unk4063627017 = 4063627017, // river.shpk, water.shpk
}

impl std::fmt::Display for ShaderParamId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&format!("{:?}", self))
	}
}

#[binrw]
#[brw(little, repr = u32)]
#[repr(u32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShaderKeyId {
	Unk190435047 = 190435047,
	Unk229123851 = 229123851,
	Unk274491986 = 274491986,
	Unk291288682 = 291288682,
	Unk440654153 = 440654153,
	Unk612525193 = 612525193,
	Unk681055795 = 681055795,
	Skin = 940355280,
	Unk1244803460 = 1244803460,
	Unk1330578998 = 1330578998,
	Unk1465690188 = 1465690188,
	Unk1854727813 = 1854727813,
	Unk2846092837 = 2846092837,
	Unk2883218939 = 2883218939,
	Unk3048326218 = 3048326218,
	Unk3054951514 = 3054951514,
	Unk3217219831 = 3217219831,
	Unk3367837167 = 3367837167,
	Unk3420444140 = 3420444140,
	Unk3531043187 = 3531043187,
	Unk3762391338 = 3762391338,
	Unk3967836472 = 3967836472,
	Unk4113354501 = 4113354501,
	Unk4176438622 = 4176438622,
	Unk4219131364 = 4219131364,
}

impl std::fmt::Display for ShaderKeyId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&format!("{:?}", self))
	}
}

#[binrw]
#[brw(little, repr = u32)]
#[repr(u32)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SamplerType {
	CharaNormal = 207536625, // chara/equipment/e6015/texture/v01_c0201e6015_sho_n.tex
	CharaDiffuse = 290653886, // chara/human/c0801/obj/face/f0003/texture/c0801f0003_fac_d.tex
	BgUnk465317650 = 465317650, // bg/ex4/02_mid_m5/twn/m5t1/texture/m5t1_k1_flor5_s.tex (some funky stuff idk)
	BgDiffuse = 510652316, // bg/ex4/02_mid_m5/twn/m5t1/texture/m5t1_k1_flor5_d.tex
	WaterUnk541659712 = 541659712, // bgcommon/nature/water/texture/_n_wavelet_000.tex (used by water shader together with 1464738518, so probably only uses certain layer(s))
	LiftShaftUnk557626425 = 557626425, // bgcommon/nature/lightshaft/texture/_n_lightshaft_s0_000.tex
	CharaSpecular = 731504677, // chara/monster/m0060/obj/body/b0001/texture/v01_m0060b0001_s.tex
	LiftShaftUnk1446741167 = 1446741167, // bgcommon/nature/lightshaft/texture/_n_lightshaft_s1_000.tex (noise layer shader by RGB, idk)
	WaterUnk1464738518 = 1464738518, // bgcommon/nature/water/texture/_n_wavelet_000.tex (see 541659712)
	BgUnk1768480522 = 1768480522, // bgcommon/texture/dummy_d.tex
	BgUnk1824202628 = 1824202628, // bgcommon/texture/dummy_s.tex
	WaterUnk2281064269 = 2281064269, // bg/ffxiv/est_e1/hou/e1h1/texture/e1h1_w1_wate1_f.tex (ironworks doesnt like it, but probably water splashes/fog/mist/the stuff at the bottom of waterfalls)
	Fog = 2285931524, // bgcommon/nature/verticalfog/texture/_n_verticalfog_000.tex (3 different noise layers for RGB)
	Multi = 2320401078, // chara/equipment/e6015/texture/v01_c0201e6015_sho_m.tex
	WaterUnk2514613837 = 2514613837, // bgcommon/nature/water/texture/_n_whitecap_000.tex
	BgNormal = 2863978985, // bg/ex4/02_mid_m5/twn/m5t1/texture/m5t1_k1_flor5_n.tex (this texture sucks but its normal for bg)
	BgUnk3719555455 = 3719555455, // bgcommon/texture/dummy_n.tex
	WaterNormal = 3862043388, // bgcommon/nature/water/texture/_n_wave_000.tex (normal but the alpha layer seems to have a different purpose)
	Cubemap = 4174878074, // bgcommon/nature/envmap/texture/_n_envmap_000.tex (ironworks fail, probably cubemap)
	Reflection = 4271961042, // chara/monster/m0536/obj/body/b0001/texture/v01_m0536b0001_d.tex (used by iris, also by ozma which uses the iris shader lol)
}

impl std::fmt::Display for SamplerType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&format!("{:?}", self))
	}
}

#[derive(Clone, Debug)]
pub enum Shader {
	Bg(Bg),
	BgUvScroll(BgUvScroll),
	BgColorChange(BgColorChange),
	BgCrestChange(BgCrestChange),
	Crystal(Crystal),
	LightShaft(LightShaft),
	VerticalFog(VerticalFog),
	River(River),
	Water(Water),
	Character(Character),
	CharacterGlass(CharacterGlass),
	Skin(Skin),
	Iris(Iris),
	Hair(Hair),
}

impl Shader {
	pub fn shader_name(&self) -> &'static str {
		match self {
			Shader::Bg(_) => "bg.shpk",
			Shader::BgUvScroll(_) => "bguvscroll.shpk",
			Shader::BgColorChange(_) => "bgcolorchange.shpk",
			Shader::BgCrestChange(_) => "bgcrestchange.shpk",
			Shader::Crystal(_) => "crystal.shpk",
			Shader::LightShaft(_) => "lightshaft.shpk",
			Shader::VerticalFog(_) => "verticalfog.shpk",
			Shader::River(_) => "river.shpk",
			Shader::Water(_) => "water.shpk",
			Shader::Character(_) => "character.shpk",
			Shader::CharacterGlass(_) => "characterglass.shpk",
			Shader::Skin(_) => "skin.shpk",
			Shader::Iris(_) => "iris.shpk",
			Shader::Hair(_) => "hair.shpk",
		}
	}
	
	pub fn new(shader_name: &str) -> Option<Self> {
		match shader_name {
			"bg.shpk" => Some(Shader::Bg(Bg::default())),
			"bguvscroll.shpk" => Some(Shader::BgUvScroll(BgUvScroll::default())),
			"bgcolorchange.shpk" => Some(Shader::BgColorChange(BgColorChange::default())),
			"bgcrestchange.shpk" => Some(Shader::BgCrestChange(BgCrestChange::default())),
			"crystal.shpk" => Some(Shader::Crystal(Crystal::default())),
			"lightshaft.shpk" => Some(Shader::LightShaft(LightShaft::default())),
			"verticalfog.shpk" => Some(Shader::VerticalFog(VerticalFog::default())),
			"river.shpk" => Some(Shader::River(River::default())),
			"water.shpk" => Some(Shader::Water(Water::default())),
			"character.shpk" => Some(Shader::Character(Character::default())),
			"characterglass.shpk" => Some(Shader::CharacterGlass(CharacterGlass::default())),
			"skin.shpk" => Some(Shader::Skin(Skin::default())),
			"iris.shpk" => Some(Shader::Iris(Iris::default())),
			"hair.shpk" => Some(Shader::Hair(Hair::default())),
			_ => None,
		}
	}
	
	pub fn inner(&self) -> &ShaderParams {
		match self {
			Shader::Bg(v) => &v.0,
			Shader::BgUvScroll(v) => &v.0,
			Shader::BgColorChange(v) => &v.0,
			Shader::BgCrestChange(v) => &v.0,
			Shader::Crystal(v) => &v.0,
			Shader::LightShaft(v) => &v.0,
			Shader::VerticalFog(v) => &v.0,
			Shader::River(v) => &v.0,
			Shader::Water(v) => &v.0,
			Shader::Character(v) => &v.0,
			Shader::CharacterGlass(v) => &v.0,
			Shader::Skin(v) => &v.0,
			Shader::Iris(v) => &v.0,
			Shader::Hair(v) => &v.0,
		}
	}
	
	pub fn inner_mut(&mut self) -> &mut ShaderParams {
		match self {
			Shader::Bg(v) => &mut v.0,
			Shader::BgUvScroll(v) => &mut v.0,
			Shader::BgColorChange(v) => &mut v.0,
			Shader::BgCrestChange(v) => &mut v.0,
			Shader::Crystal(v) => &mut v.0,
			Shader::LightShaft(v) => &mut v.0,
			Shader::VerticalFog(v) => &mut v.0,
			Shader::River(v) => &mut v.0,
			Shader::Water(v) => &mut v.0,
			Shader::Character(v) => &mut v.0,
			Shader::CharacterGlass(v) => &mut v.0,
			Shader::Skin(v) => &mut v.0,
			Shader::Iris(v) => &mut v.0,
			Shader::Hair(v) => &mut v.0,
		}
	}
	
	pub fn into_inner(self) -> ShaderParams {
		match self {
			Shader::Bg(v) => v.0,
			Shader::BgUvScroll(v) => v.0,
			Shader::BgColorChange(v) => v.0,
			Shader::BgCrestChange(v) => v.0,
			Shader::Crystal(v) => v.0,
			Shader::LightShaft(v) => v.0,
			Shader::VerticalFog(v) => v.0,
			Shader::River(v) => v.0,
			Shader::Water(v) => v.0,
			Shader::Character(v) => v.0,
			Shader::CharacterGlass(v) => v.0,
			Shader::Skin(v) => v.0,
			Shader::Iris(v) => v.0,
			Shader::Hair(v) => v.0,
		}
	}
	
	pub fn shader_names() -> [&'static str; 14] {
		[
			"bg.shpk",
			"bguvscroll.shpk",
			"bgcolorchange.shpk",
			"bgcrestchange.shpk",
			"crystal.shpk",
			"lightshaft.shpk",
			"verticalfog.shpk",
			"river.shpk",
			"water.shpk",
			"character.shpk",
			"characterglass.shpk",
			"skin.shpk",
			"iris.shpk",
			"hair.shpk"
		]
	}
}

// all defaults are based on the most common value for the shader + param, not the actual default
#[derive(Clone, Debug)]
pub struct Bg(ShaderParams);
impl Default for Bg {
	fn default() -> Self {
		Self(ShaderParams {
			params: BTreeMap::from([
				(ShaderParamId::Unk133014596, ShaderParam {enabled: false, vals: vec![0.0, 0.0, 0.0, 1.0]}), // 37754
				(ShaderParamId::Unk337060565, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 34715
				(ShaderParamId::Unk699138595, ShaderParam {enabled: false, vals: vec![0.5]}), // 4249
				(ShaderParamId::Unk740963549, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 37588
				(ShaderParamId::Unk817435417, ShaderParam {enabled: false, vals: vec![0.0, 1.0]}), // 7
				(ShaderParamId::Unk950420322, ShaderParam {enabled: false, vals: vec![0.0, 0.0, 0.0]}), // 33062
				(ShaderParamId::Unk1066058257, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 38451
				(ShaderParamId::Unk1139120744, ShaderParam {enabled: false, vals: vec![0.015]}), // 28734
				(ShaderParamId::Unk2033894819, ShaderParam {enabled: false, vals: vec![1.0]}), // 36947
				(ShaderParamId::Unk2262174904, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 13474
				(ShaderParamId::Unk2365826946, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0, 1.0]}), // 38640
				(ShaderParamId::Unk2408251504, ShaderParam {enabled: false, vals: vec![0.015]}), // 24969
				(ShaderParamId::Unk2456716813, ShaderParam {enabled: false, vals: vec![1.0]}), // 37549
				(ShaderParamId::Unk2781883474, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0, 1.0]}), // 38622
				(ShaderParamId::Unk2858904847, ShaderParam {enabled: false, vals: vec![0.0, 0.0, 0.0]}), // 37912
				(ShaderParamId::Unk3036724004, ShaderParam {enabled: false, vals: vec![0.0]}), // 255
				(ShaderParamId::Unk3042205627, ShaderParam {enabled: false, vals: vec![1.0]}), // 36222
				(ShaderParamId::Unk3086627810, ShaderParam {enabled: false, vals: vec![1.0]}), // 37652
				(ShaderParamId::Unk3147419510, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0, 1.0]}), // 37306
				(ShaderParamId::Unk3219771693, ShaderParam {enabled: false, vals: vec![1.0]}), // 35960
			]),
			keys: BTreeMap::from([
				(ShaderKeyId::Unk190435047, ShaderKey {enabled: false, val: 131274511}), // 91
				(ShaderKeyId::Unk274491986, ShaderKey {enabled: false, val: 4160862297}), // 5
				(ShaderKeyId::Unk440654153, ShaderKey {enabled: false, val: 752097944}), // 18
				(ShaderKeyId::Unk681055795, ShaderKey {enabled: false, val: 3713329004}), // 1
				(ShaderKeyId::Unk1330578998, ShaderKey {enabled: false, val: 3180618906}), // 7782
				(ShaderKeyId::Unk1465690188, ShaderKey {enabled: false, val: 671594654}), // 1585
				(ShaderKeyId::Unk2846092837, ShaderKey {enabled: false, val: 1923787182}), // 4573
				(ShaderKeyId::Unk3048326218, ShaderKey {enabled: false, val: 2186107714}), // 1
				(ShaderKeyId::Unk3054951514, ShaderKey {enabled: false, val: 502437980}), // 6141
				(ShaderKeyId::Unk3217219831, ShaderKey {enabled: false, val: 1999632171}), // 88
				(ShaderKeyId::Unk3420444140, ShaderKey {enabled: false, val: 2792035745}), // 381
				(ShaderKeyId::Unk4176438622, ShaderKey {enabled: false, val: 3869682983}), // 1
			]),
			samplers: BTreeMap::from([
				(SamplerType::BgUnk465317650, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 35200
				(SamplerType::BgDiffuse, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 38867
				(SamplerType::BgUnk1768480522, Sampler {enabled: false, path: String::with_capacity(128), flags: 832}), // 31604
				(SamplerType::BgUnk1824202628, Sampler {enabled: false, path: String::with_capacity(128), flags: 832}), // 31807
				(SamplerType::BgNormal, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 33266
				(SamplerType::BgUnk3719555455, Sampler {enabled: false, path: String::with_capacity(128), flags: 832}), // 31807
			]),
			unk: vec![17, 1065353216, 1065353216, 1065353216, 1065353216, 0, 0, 0, 1065353216], // 21937
		})
	}
}

#[derive(Clone, Debug)]
pub struct BgUvScroll(ShaderParams);
impl Default for BgUvScroll {
	fn default() -> Self {
		Self(ShaderParams {
			params: BTreeMap::from([
				(ShaderParamId::Unk337060565, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 254
				(ShaderParamId::Unk699138595, ShaderParam {enabled: false, vals: vec![0.5]}), // 64
				(ShaderParamId::Unk740963549, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 207
				(ShaderParamId::Unk950420322, ShaderParam {enabled: false, vals: vec![0.0, 0.0, 0.0]}), // 64
				(ShaderParamId::Unk1066058257, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 268
				(ShaderParamId::Unk1139120744, ShaderParam {enabled: false, vals: vec![0.015]}), // 197
				(ShaderParamId::Unk2033894819, ShaderParam {enabled: false, vals: vec![1.0]}), // 261
				(ShaderParamId::Unk2262174904, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 142
				(ShaderParamId::Unk2365826946, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0, 1.0]}), // 283
				(ShaderParamId::Unk2408251504, ShaderParam {enabled: false, vals: vec![0.015]}), // 179
				(ShaderParamId::Unk2456716813, ShaderParam {enabled: false, vals: vec![1.0]}), // 267
				(ShaderParamId::Unk2590599703, ShaderParam {enabled: false, vals: vec![0.0, -1.0, 0.0, 0.0]}), // 17
				(ShaderParamId::Unk2781883474, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0, 1.0]}), // 283
				(ShaderParamId::Unk2858904847, ShaderParam {enabled: false, vals: vec![0.0, 0.0, 0.0]}), // 203
				(ShaderParamId::Unk3042205627, ShaderParam {enabled: false, vals: vec![1.0]}), // 241
				(ShaderParamId::Unk3086627810, ShaderParam {enabled: false, vals: vec![1.0]}), // 266
				(ShaderParamId::Unk3147419510, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0, 1.0]}), // 278
				(ShaderParamId::Unk3219771693, ShaderParam {enabled: false, vals: vec![1.0]}), // 177
			]),
			keys: BTreeMap::from([
				(ShaderKeyId::Unk1330578998, ShaderKey {enabled: false, val: 3180618906}), // 3
				(ShaderKeyId::Unk1465690188, ShaderKey {enabled: false, val: 671594654}), // 45
				(ShaderKeyId::Unk2846092837, ShaderKey {enabled: false, val: 1923787182}), // 83
				(ShaderKeyId::Unk3054951514, ShaderKey {enabled: false, val: 3835352875}), // 116
			]),
			samplers: BTreeMap::from([
				(SamplerType::BgUnk465317650, Sampler {enabled: false, path: String::with_capacity(128), flags: 832}), // 148
				(SamplerType::BgDiffuse, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 282
				(SamplerType::BgUnk1768480522, Sampler {enabled: false, path: String::with_capacity(128), flags: 832}), // 165
				(SamplerType::BgUnk1824202628, Sampler {enabled: false, path: String::with_capacity(128), flags: 832}), // 205
				(SamplerType::BgNormal, Sampler {enabled: false, path: String::with_capacity(128), flags: 832}), // 156
				(SamplerType::BgUnk3719555455, Sampler {enabled: false, path: String::with_capacity(128), flags: 832}), // 169
			]),
			unk: vec![0, 1065353216, 1065353216, 1065353216, 1065353216, 0, 0, 0, 1065353216], // 62
		})
	}
}

#[derive(Clone, Debug)]
pub struct BgColorChange(ShaderParams);
impl Default for BgColorChange {
	fn default() -> Self {
		Self(ShaderParams {
			params: BTreeMap::from([
				(ShaderParamId::Unk337060565, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 725
				(ShaderParamId::Unk740963549, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 311
				(ShaderParamId::Unk950420322, ShaderParam {enabled: false, vals: vec![0.0, 0.0, 0.0]}), // 734
				(ShaderParamId::Unk2408251504, ShaderParam {enabled: false, vals: vec![0.0]}), // 296
				(ShaderParamId::Unk2456716813, ShaderParam {enabled: false, vals: vec![1.0]}), // 670
				(ShaderParamId::Unk3042205627, ShaderParam {enabled: false, vals: vec![1.0]}), // 724
				(ShaderParamId::Unk3086627810, ShaderParam {enabled: false, vals: vec![1.0]}), // 726
				(ShaderParamId::Unk3219771693, ShaderParam {enabled: false, vals: vec![1.0]}), // 735
			]),
			keys: BTreeMap::from([
				(ShaderKeyId::Unk1330578998, ShaderKey {enabled: false, val: 3180618906}), // 6
				(ShaderKeyId::Unk2846092837, ShaderKey {enabled: false, val: 1923787182}), // 13
			]),
			samplers: BTreeMap::from([
				(SamplerType::BgUnk465317650, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 723
				(SamplerType::BgDiffuse, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 737
				(SamplerType::BgNormal, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 682
			]),
			unk: vec![1041, 1065353216, 1065353216, 1065353216, 1065353216, 0, 0, 0, 1065353216], // 669
		})
	}
}

#[derive(Clone, Debug)]
pub struct BgCrestChange(ShaderParams);
impl Default for BgCrestChange {
	fn default() -> Self {
		Self(ShaderParams {
			params: BTreeMap::from([
				(ShaderParamId::Unk337060565, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 40
				(ShaderParamId::Unk740963549, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 41
				(ShaderParamId::Unk950420322, ShaderParam {enabled: false, vals: vec![0.0, 0.0, 0.0]}), // 41
				(ShaderParamId::Unk2408251504, ShaderParam {enabled: false, vals: vec![0.015]}), // 20
				(ShaderParamId::Unk2456716813, ShaderParam {enabled: false, vals: vec![1.0]}), // 30
				(ShaderParamId::Unk3042205627, ShaderParam {enabled: false, vals: vec![1.0]}), // 39
				(ShaderParamId::Unk3086627810, ShaderParam {enabled: false, vals: vec![1.0]}), // 41
				(ShaderParamId::Unk3219771693, ShaderParam {enabled: false, vals: vec![1.0]}), // 41
			]),
			keys: BTreeMap::from([
				(ShaderKeyId::Unk2846092837, ShaderKey {enabled: false, val: 1923787182}), // 8
				(ShaderKeyId::Unk3054951514, ShaderKey {enabled: false, val: 502437980}), // 1
			]),
			samplers: BTreeMap::from([
				(SamplerType::BgUnk465317650, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 35
				(SamplerType::BgDiffuse, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 41
				(SamplerType::BgUnk1768480522, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 41
				(SamplerType::BgNormal, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 33
			]),
			unk: vec![4113, 1065353216, 1065353216, 1065353216, 1065353216, 0, 0, 0, 1065353216], // 24
		})
	}
}

#[derive(Clone, Debug)]
pub struct Crystal(ShaderParams);
impl Default for Crystal {
	fn default() -> Self {
		Self(ShaderParams {
			params: BTreeMap::from([
				(ShaderParamId::Unk337060565, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 29
				(ShaderParamId::Unk740963549, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 32
				(ShaderParamId::Unk950420322, ShaderParam {enabled: false, vals: vec![0.0, 0.0, 0.0]}), // 24
				(ShaderParamId::Unk3042205627, ShaderParam {enabled: false, vals: vec![1.0]}), // 47
				(ShaderParamId::Unk3219771693, ShaderParam {enabled: false, vals: vec![1.0]}), // 50
				(ShaderParamId::Unk4009059935, ShaderParam {enabled: false, vals: vec![1.0]}), // 30
			]),
			keys: BTreeMap::from([
				(ShaderKeyId::Unk1330578998, ShaderKey {enabled: false, val: 3180618906}), // 6
				(ShaderKeyId::Unk2846092837, ShaderKey {enabled: false, val: 1923787182}), // 1
			]),
			samplers: BTreeMap::from([
				(SamplerType::BgUnk465317650, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 93
				(SamplerType::BgDiffuse, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 97
				(SamplerType::BgNormal, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 90
				(SamplerType::Cubemap, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 97
			]),
			unk: vec![17, 1065353216, 1065353216, 1065353216, 1065353216, 0, 0, 0, 1065353216], // 85
		})
	}
}

#[derive(Clone, Debug)]
pub struct LightShaft(ShaderParams);
impl Default for LightShaft {
	fn default() -> Self {
		Self(ShaderParams {
			params: BTreeMap::from([
				(ShaderParamId::Unk349757757, ShaderParam {enabled: false, vals: vec![0.0, 0.0]}), // 117
				(ShaderParamId::Unk396699942, ShaderParam {enabled: false, vals: vec![1.0]}), // 184
				(ShaderParamId::Unk1495703619, ShaderParam {enabled: false, vals: vec![0.0, 0.0, 0.05]}), // 85
				(ShaderParamId::Unk1910233729, ShaderParam {enabled: false, vals: vec![1.0]}), // 391
				(ShaderParamId::Unk2189155593, ShaderParam {enabled: false, vals: vec![0.0, 0.0, 1.0]}), // 616
				(ShaderParamId::Unk3224367609, ShaderParam {enabled: false, vals: vec![0.0, 0.05, 0.0]}), // 92
				(ShaderParamId::Unk3531364537, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 147
			]),
			keys: BTreeMap::from([
				(ShaderKeyId::Unk229123851, ShaderKey {enabled: false, val: 3321983381}), // 13
				(ShaderKeyId::Unk1330578998, ShaderKey {enabled: false, val: 3180618906}), // 3
				(ShaderKeyId::Unk1465690188, ShaderKey {enabled: false, val: 671594654}), // 3
				(ShaderKeyId::Unk2846092837, ShaderKey {enabled: false, val: 1923787182}), // 3
			]),
			samplers: BTreeMap::from([
				(SamplerType::LiftShaftUnk557626425, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 813
				(SamplerType::LiftShaftUnk1446741167, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 813
			]),
			unk: Vec::new(),
		})
	}
}

#[derive(Clone, Debug)]
pub struct VerticalFog(ShaderParams);
impl Default for VerticalFog {
	fn default() -> Self {
		Self(ShaderParams {
			params: BTreeMap::from([
				(ShaderParamId::Unk167817377, ShaderParam {enabled: false, vals: vec![0.5]}), // 109
				(ShaderParamId::Unk903613295, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 48
				(ShaderParamId::Unk1914183202, ShaderParam {enabled: false, vals: vec![10.0, 0.0]}), // 53
				(ShaderParamId::Unk2494828270, ShaderParam {enabled: false, vals: vec![0.5]}), // 94
				(ShaderParamId::Unk2838061039, ShaderParam {enabled: false, vals: vec![10.0]}), // 102
				(ShaderParamId::Unk3166335201, ShaderParam {enabled: false, vals: vec![0.1]}), // 52
				(ShaderParamId::Unk3217843714, ShaderParam {enabled: false, vals: vec![0.01]}), // 67
				(ShaderParamId::Unk3494687889, ShaderParam {enabled: false, vals: vec![0.01]}), // 54
				(ShaderParamId::Unk3531364537, ShaderParam {enabled: false, vals: vec![0.0, 0.0, 0.0, 1.0]}), // 39
			]),
			keys: BTreeMap::new(),
			samplers: BTreeMap::from([
				(SamplerType::Fog, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 201
			]),
			unk: Vec::new(),
		})
	}
}

#[derive(Clone, Debug)]
pub struct River(ShaderParams);
impl Default for River {
	fn default() -> Self {
		Self(ShaderParams {
			params: BTreeMap::from([
				(ShaderParamId::Unk213950055, ShaderParam {enabled: false, vals: vec![0.5]}), // 210
				(ShaderParamId::Unk289124319, ShaderParam {enabled: false, vals: vec![8.0, 8.0]}), // 182
				(ShaderParamId::Unk371521601, ShaderParam {enabled: false, vals: vec![0.02]}), // 26
				(ShaderParamId::Unk574239529, ShaderParam {enabled: false, vals: vec![0.125]}), // 137
				(ShaderParamId::Unk704260801, ShaderParam {enabled: false, vals: vec![0.2033108, 0.22137025, 0.240198]}), // 45
				(ShaderParamId::Unk710394828, ShaderParam {enabled: false, vals: vec![3.3333333]}), // 69
				(ShaderParamId::Unk824928705, ShaderParam {enabled: false, vals: vec![4.0, 4.0]}), // 162
				(ShaderParamId::Unk926302173, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 2.0, 2.0]}), // 195
				(ShaderParamId::Unk1082825950, ShaderParam {enabled: false, vals: vec![6.0, 3.0, 3.0]}), // 102
				(ShaderParamId::Unk1407730043, ShaderParam {enabled: false, vals: vec![1.0]}), // 184
				(ShaderParamId::Unk1529014372, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 2.0, 2.0]}), // 218
				(ShaderParamId::Unk1886515549, ShaderParam {enabled: false, vals: vec![0.75]}), // 184
				(ShaderParamId::Unk2394542758, ShaderParam {enabled: false, vals: vec![0.066]}), // 66
				(ShaderParamId::Unk2471513915, ShaderParam {enabled: false, vals: vec![0.02, 0.04]}), // 162
				(ShaderParamId::Unk2516865232, ShaderParam {enabled: false, vals: vec![0.1, 0.1]}), // 164
				(ShaderParamId::Unk2548576516, ShaderParam {enabled: false, vals: vec![0.01, 0.02]}), // 160
				(ShaderParamId::Unk2569562539, ShaderParam {enabled: false, vals: vec![64.0]}), // 197
				(ShaderParamId::Unk2736828825, ShaderParam {enabled: false, vals: vec![0.5]}), // 86
				(ShaderParamId::Unk3007164738, ShaderParam {enabled: false, vals: vec![4.0, 4.0]}), // 194
				(ShaderParamId::Unk3042205627, ShaderParam {enabled: false, vals: vec![1.0]}), // 190
				(ShaderParamId::Unk3111546299, ShaderParam {enabled: false, vals: vec![50.0]}), // 94
				(ShaderParamId::Unk3122018048, ShaderParam {enabled: false, vals: vec![0.38068897, 0.3789938, 0.33637682]}), // 18
				(ShaderParamId::Unk3264604780, ShaderParam {enabled: false, vals: vec![0.5]}), // 186
				(ShaderParamId::Unk3541428008, ShaderParam {enabled: false, vals: vec![0.06585331, 0.10489703, 0.203401]}), // 11
				(ShaderParamId::Unk3653987228, ShaderParam {enabled: false, vals: vec![1.0]}), // 229
				(ShaderParamId::Unk3819586170, ShaderParam {enabled: false, vals: vec![0.25]}), // 219
				(ShaderParamId::Unk3838218227, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0, 1.0]}), // 183
				(ShaderParamId::Unk4063627017, ShaderParam {enabled: false, vals: vec![0.5]}), // 171
			]),
			keys: BTreeMap::from([
				(ShaderKeyId::Unk291288682, ShaderKey {enabled: false, val: 4029502289}), // 32
				(ShaderKeyId::Unk681055795, ShaderKey {enabled: false, val: 3713329004}), // 1
				(ShaderKeyId::Unk1244803460, ShaderKey {enabled: false, val: 3889752702}), // 1
				(ShaderKeyId::Unk1854727813, ShaderKey {enabled: false, val: 3530971738}), // 27
				(ShaderKeyId::Unk2883218939, ShaderKey {enabled: false, val: 429953500}), // 27
				(ShaderKeyId::Unk3048326218, ShaderKey {enabled: false, val: 2186107714}), // 2
				(ShaderKeyId::Unk3054951514, ShaderKey {enabled: false, val: 502437980}), // 27
				(ShaderKeyId::Unk3762391338, ShaderKey {enabled: false, val: 652478584}), // 190
				(ShaderKeyId::Unk3967836472, ShaderKey {enabled: false, val: 4039188000}), // 27
				(ShaderKeyId::Unk4176438622, ShaderKey {enabled: false, val: 138432195}), // 95
			]),
			samplers: BTreeMap::from([
				(SamplerType::WaterUnk2281064269, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 10
				(SamplerType::WaterUnk2514613837, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 250
				(SamplerType::WaterNormal, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 250
			]),
			unk: vec![0, 1065353216, 1065353216, 1065353216, 1065353216, 0, 0, 0, 0], // 248
		})
	}
}

#[derive(Clone, Debug)]
pub struct Water(ShaderParams);
impl Default for Water {
	fn default() -> Self {
		Self(ShaderParams {
			params: BTreeMap::from([
				(ShaderParamId::Unk213950055, ShaderParam {enabled: false, vals: vec![0.5]}), // 429
				(ShaderParamId::Unk268000012, ShaderParam {enabled: false, vals: vec![0.1, 0.1]}), // 372
				(ShaderParamId::Unk309952860, ShaderParam {enabled: false, vals: vec![0.003, 0.15, 0.001]}), // 476
				(ShaderParamId::Unk371521601, ShaderParam {enabled: false, vals: vec![0.033333335]}), // 58
				(ShaderParamId::Unk574239529, ShaderParam {enabled: false, vals: vec![0.125]}), // 248
				(ShaderParamId::Unk704260801, ShaderParam {enabled: false, vals: vec![0.2033108, 0.22137025, 0.240198, 0.3]}), // 212
				(ShaderParamId::Unk710394828, ShaderParam {enabled: false, vals: vec![3.3333333]}), // 145
				(ShaderParamId::Unk792844182, ShaderParam {enabled: false, vals: vec![2.0, 0.2]}), // 475
				(ShaderParamId::Unk876196728, ShaderParam {enabled: false, vals: vec![0.01, 0.01, 0.04, 0.04]}), // 475
				(ShaderParamId::Unk1082825950, ShaderParam {enabled: false, vals: vec![15.0]}), // 268
				(ShaderParamId::Unk1255709111, ShaderParam {enabled: false, vals: vec![3.3333333, 3.3333333]}), // 395
				(ShaderParamId::Unk1407730043, ShaderParam {enabled: false, vals: vec![1.0]}), // 295
				(ShaderParamId::Unk1536774237, ShaderParam {enabled: false, vals: vec![0.25]}), // 447
				(ShaderParamId::Unk1562817122, ShaderParam {enabled: false, vals: vec![2.0]}), // 282
				(ShaderParamId::Unk1627729957, ShaderParam {enabled: false, vals: vec![0.0, 0.0]}), // 394
				(ShaderParamId::Unk1886515549, ShaderParam {enabled: false, vals: vec![0.5]}), // 369
				(ShaderParamId::Unk2394542758, ShaderParam {enabled: false, vals: vec![0.033]}), // 123
				(ShaderParamId::Unk2569562539, ShaderParam {enabled: false, vals: vec![64.0]}), // 430
				(ShaderParamId::Unk2615686474, ShaderParam {enabled: false, vals: vec![0.0]}), // 421
				(ShaderParamId::Unk2736828825, ShaderParam {enabled: false, vals: vec![0.5]}), // 191
				(ShaderParamId::Unk2750039980, ShaderParam {enabled: false, vals: vec![0.02]}), // 272
				(ShaderParamId::Unk2784956570, ShaderParam {enabled: false, vals: vec![0.02]}), // 288
				(ShaderParamId::Unk3042205627, ShaderParam {enabled: false, vals: vec![1.0]}), // 321
				(ShaderParamId::Unk3111546299, ShaderParam {enabled: false, vals: vec![0.0]}), // 191
				(ShaderParamId::Unk3122018048, ShaderParam {enabled: false, vals: vec![0.35402504, 0.44222504, 0.48999998]}), // 28
				(ShaderParamId::Unk3264604780, ShaderParam {enabled: false, vals: vec![0.5]}), // 391
				(ShaderParamId::Unk3541428008, ShaderParam {enabled: false, vals: vec![0.0324, 0.0676, 0.09]}), // 18
				(ShaderParamId::Unk3593234462, ShaderParam {enabled: false, vals: vec![0.5, 1.0, 0.5, 1.0]}), // 473
				(ShaderParamId::Unk3653987228, ShaderParam {enabled: false, vals: vec![1.0]}), // 399
				(ShaderParamId::Unk3819586170, ShaderParam {enabled: false, vals: vec![0.25]}), // 400
				(ShaderParamId::Unk4063627017, ShaderParam {enabled: false, vals: vec![0.1]}), // 210
			]),
			keys: BTreeMap::from([
				(ShaderKeyId::Unk291288682, ShaderKey {enabled: false, val: 4029502289}), // 3
				(ShaderKeyId::Unk681055795, ShaderKey {enabled: false, val: 3713329004}), // 145
				(ShaderKeyId::Unk1244803460, ShaderKey {enabled: false, val: 3889752702}), // 1
				(ShaderKeyId::Unk1854727813, ShaderKey {enabled: false, val: 3530971738}), // 3
				(ShaderKeyId::Unk2883218939, ShaderKey {enabled: false, val: 429953500}), // 3
				(ShaderKeyId::Unk3048326218, ShaderKey {enabled: false, val: 2186107714}), // 93
				(ShaderKeyId::Unk3054951514, ShaderKey {enabled: false, val: 502437980}), // 4
				(ShaderKeyId::Unk3762391338, ShaderKey {enabled: false, val: 652478584}), // 77
				(ShaderKeyId::Unk3967836472, ShaderKey {enabled: false, val: 4039188000}), // 3
				(ShaderKeyId::Unk4176438622, ShaderKey {enabled: false, val: 138432195}), // 254
				(ShaderKeyId::Unk4219131364, ShaderKey {enabled: false, val: 289469532}), // 19
			]),
			samplers: BTreeMap::from([
				(SamplerType::WaterUnk541659712, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 19
				(SamplerType::WaterUnk1464738518, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 19
				(SamplerType::WaterUnk2281064269, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 61
				(SamplerType::WaterUnk2514613837, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 144
				(SamplerType::WaterNormal, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 477
			]),
			unk: vec![0, 1065353216, 1065353216, 1065353216, 1065353216, 0, 0, 0, 0], // 465
		})
	}
}

#[derive(Clone, Debug)]
pub struct Character(ShaderParams);
impl Default for Character {
	fn default() -> Self {
		Self(ShaderParams {
			params: BTreeMap::from([
				(ShaderParamId::Unk364318261, ShaderParam {enabled: false, vals: vec![0.0]}), // 6382
				(ShaderParamId::Unk699138595, ShaderParam {enabled: false, vals: vec![0.5]}), // 28819
				(ShaderParamId::Unk906496720, ShaderParam {enabled: false, vals: vec![1.0]}), // 196
				(ShaderParamId::Unk1465565106, ShaderParam {enabled: false, vals: vec![0.25]}), // 24723
				(ShaderParamId::Unk3036724004, ShaderParam {enabled: false, vals: vec![0.0]}), // 5183
			]),
			keys: BTreeMap::from([
				(ShaderKeyId::Unk3054951514, ShaderKey {enabled: false, val: 1611594207}), // 16111
				(ShaderKeyId::Unk3367837167, ShaderKey {enabled: false, val: 2687453224}), // 849
				(ShaderKeyId::Unk3531043187, ShaderKey {enabled: false, val: 4083110193}), // 10005
				(ShaderKeyId::Unk4113354501, ShaderKey {enabled: false, val: 2815623008}), // 29125
			]),
			samplers: BTreeMap::from([
				(SamplerType::CharaNormal, Sampler {enabled: false, path: String::with_capacity(128), flags: 1015829}), // 18021
				(SamplerType::CharaDiffuse, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 9624
				(SamplerType::CharaSpecular, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 10327
				(SamplerType::Multi, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016661}), // 9205
			]),
			unk: vec![4], // 17644
		})
	}
}

#[derive(Clone, Debug)]
pub struct CharacterGlass(ShaderParams);
impl Default for CharacterGlass {
	fn default() -> Self {
		Self(ShaderParams {
			params: BTreeMap::from([
				(ShaderParamId::Unk364318261, ShaderParam {enabled: false, vals: vec![0.0]}), // 20
				(ShaderParamId::Unk1465565106, ShaderParam {enabled: false, vals: vec![0.25]}), // 133
				(ShaderParamId::Unk3042205627, ShaderParam {enabled: false, vals: vec![1.0]}), // 162
			]),
			keys: BTreeMap::from([
				(ShaderKeyId::Unk3054951514, ShaderKey {enabled: false, val: 1611594207}), // 5
				(ShaderKeyId::Unk3367837167, ShaderKey {enabled: false, val: 2687453224}), // 1
				(ShaderKeyId::Unk3531043187, ShaderKey {enabled: false, val: 4083110193}), // 63
				(ShaderKeyId::Unk4113354501, ShaderKey {enabled: false, val: 2815623008}), // 158
			]),
			samplers: BTreeMap::from([
				(SamplerType::CharaNormal, Sampler {enabled: false, path: String::with_capacity(128), flags: 1015829}), // 151
				(SamplerType::Multi, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016661}), // 136
			]),
			unk: vec![5], // 122
		})
	}
}

#[derive(Clone, Debug)]
pub struct Skin(ShaderParams);
impl Default for Skin {
	fn default() -> Self {
		Self(ShaderParams {
			params: BTreeMap::from([
				(ShaderParamId::Unk390837838, ShaderParam {enabled: false, vals: vec![1.0, 1.0, 1.0]}), // 316
				(ShaderParamId::Unk699138595, ShaderParam {enabled: false, vals: vec![0.5]}), // 346
				(ShaderParamId::Unk740963549, ShaderParam {enabled: false, vals: vec![1.4, 1.4, 1.4]}), // 383
				(ShaderParamId::Unk778088561, ShaderParam {enabled: false, vals: vec![20.0, 20.0]}), // 314
				(ShaderParamId::Unk906496720, ShaderParam {enabled: false, vals: vec![1.0]}), // 411
				(ShaderParamId::Unk950420322, ShaderParam {enabled: false, vals: vec![0.0, 0.0, 0.0]}), // 411
				(ShaderParamId::Unk1112929012, ShaderParam {enabled: false, vals: vec![63.0]}), // 398
				(ShaderParamId::Unk1465565106, ShaderParam {enabled: false, vals: vec![0.0]}), // 398
				(ShaderParamId::Unk1659128399, ShaderParam {enabled: false, vals: vec![0.02, 0.02, 0.02]}), // 311
				(ShaderParamId::Unk2274043692, ShaderParam {enabled: false, vals: vec![32.0]}), // 396
				(ShaderParamId::Unk2569562539, ShaderParam {enabled: false, vals: vec![3.0]}), // 403
				(ShaderParamId::Unk3036724004, ShaderParam {enabled: false, vals: vec![0.0]}), // 2
			]),
			keys: BTreeMap::from([
				(ShaderKeyId::Skin, ShaderKey {enabled: false, val: 735790577}), // 49
				(ShaderKeyId::Unk3054951514, ShaderKey {enabled: false, val: 1611594207}), // 31
				(ShaderKeyId::Unk3531043187, ShaderKey {enabled: false, val: 1480746461}), // 293
				(ShaderKeyId::Unk4113354501, ShaderKey {enabled: false, val: 2815623008}), // 334
			]),
			samplers: BTreeMap::from([
				(SamplerType::CharaNormal, Sampler {enabled: false, path: String::with_capacity(128), flags: 983893}), // 299
				(SamplerType::CharaDiffuse, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016661}), // 321
				(SamplerType::Multi, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016661}), // 311
			]),
			unk: vec![0], // 363
		})
	}
}

#[derive(Clone, Debug)]
pub struct Iris(ShaderParams);
impl Default for Iris {
	fn default() -> Self {
		Self(ShaderParams {
			params: BTreeMap::from([
				(ShaderParamId::Unk699138595, ShaderParam {enabled: false, vals: vec![0.0]}), // 303
				(ShaderParamId::Unk740963549, ShaderParam {enabled: false, vals: vec![1.4, 1.4, 1.4]}), // 290
				(ShaderParamId::Unk906496720, ShaderParam {enabled: false, vals: vec![1.0]}), // 322
				(ShaderParamId::Unk1465565106, ShaderParam {enabled: false, vals: vec![0.5]}), // 322
				(ShaderParamId::Unk1617630358, ShaderParam {enabled: false, vals: vec![0.5]}), // 1
				(ShaderParamId::Unk1659128399, ShaderParam {enabled: false, vals: vec![0.4, 0.4, 0.4]}), // 312
				(ShaderParamId::Unk1714175910, ShaderParam {enabled: false, vals: vec![0.5]}), // 1
				(ShaderParamId::Unk2569562539, ShaderParam {enabled: false, vals: vec![200.0]}), // 300
				(ShaderParamId::Unk3036724004, ShaderParam {enabled: false, vals: vec![0.0]}), // 263
			]),
			keys: BTreeMap::from([
				(ShaderKeyId::Unk3054951514, ShaderKey {enabled: false, val: 1611594207}), // 21
				(ShaderKeyId::Unk4113354501, ShaderKey {enabled: false, val: 2815623008}), // 21
			]),
			samplers: BTreeMap::from([
				(SamplerType::CharaNormal, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 300
				(SamplerType::Multi, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 310
				(SamplerType::Reflection, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016640}), // 311
			]),
			unk: vec![0], // 308
		})
	}
}

#[derive(Clone, Debug)]
pub struct Hair(ShaderParams);
impl Default for Hair {
	fn default() -> Self {
		Self(ShaderParams {
			params: BTreeMap::from([
				(ShaderParamId::Unk364318261, ShaderParam {enabled: false, vals: vec![0.35]}), // 891
				(ShaderParamId::Unk699138595, ShaderParam {enabled: false, vals: vec![0.75]}), // 886
				(ShaderParamId::Unk740963549, ShaderParam {enabled: false, vals: vec![1.4, 1.4, 1.4]}), // 818
				(ShaderParamId::Unk906496720, ShaderParam {enabled: false, vals: vec![1.0]}), // 885
				(ShaderParamId::Unk1465565106, ShaderParam {enabled: false, vals: vec![0.25]}), // 895
				(ShaderParamId::Unk2569562539, ShaderParam {enabled: false, vals: vec![4.0]}), // 876
				(ShaderParamId::Unk3036724004, ShaderParam {enabled: false, vals: vec![0.0]}), // 774
				(ShaderParamId::Unk3042205627, ShaderParam {enabled: false, vals: vec![0.5]}), // 895
			]),
			keys: BTreeMap::from([
				(ShaderKeyId::Unk612525193, ShaderKey {enabled: false, val: 1851494160}), // 299
				(ShaderKeyId::Unk3054951514, ShaderKey {enabled: false, val: 1611594207}), // 9
				(ShaderKeyId::Unk3367837167, ShaderKey {enabled: false, val: 2687453224}), // 4
				(ShaderKeyId::Unk4113354501, ShaderKey {enabled: false, val: 2815623008}), // 10
			]),
			samplers: BTreeMap::from([
				(SamplerType::CharaNormal, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016661}), // 545
				(SamplerType::Multi, Sampler {enabled: false, path: String::with_capacity(128), flags: 1016661}), // 553
			]),
			unk: vec![0], // 869
		})
	}
}

#[binrw]
#[brw(little)]
#[derive(Debug)]
struct Data {
	_sig: u32,
	_size: u16,
	colorset_data_size: u16,
	strings_size: u16,
	shader_offset: u16,
	texture_count: u8,
	uvset_count: u8,
	colorset_count: u8,
	unk_size: u8,
	
	#[br(count = texture_count)]
	textures: Vec<(u16, u16)>, // 2nd u16 is either 0 or 32768. only CharaDiffuse, CharaNormal, CharaSpecular, and Multi seem to be able to be 32768
	#[br(count = uvset_count)]
	uvsets: Vec<(u16, u16)>, // 2nd u16 seems to be index
	#[br(count = colorset_count)]
	colorsets: Vec<(u16, u16)>, // 2nd u16 seems to be index
	
	#[br(count = strings_size)]
	strings: Vec<u8>,
	
	// shader: {len}
	// characterglass.shpk: {4}
	// lightshaft.shpk: {0}
	// verticalfog.shpk: {0}
	// bgcolorchange.shpk: {36}
	// river.shpk: {36}
	// crystal.shpk: {36}
	// bgcrestchange.shpk: {36}
	// character.shpk: {4}
	// hair.shpk: {4}
	// skin.shpk: {4}
	// bg.shpk: {36, 4, 20}
	// water.shpk: {36}
	// iris.shpk: {4}
	// bguvscroll.shpk: {36}
	#[br(count = unk_size / 4)]
	unk: Vec<u32>,
	
	#[br(if(colorset_data_size > 0))]
	colorset_datas: [[u16; 16]; 16],
	
	#[br(if(colorset_data_size == 544))]
	colorsetdye_datas: [u16; 16],
	
	shader_param_values_size: u16,
	shader_keys_count: u16,
	shader_params_count: u16,
	sampler_count: u16,
	flags: u32,
	
	#[br(count = shader_keys_count)]
	shader_keys: Vec<(ShaderKeyId, u32)>,
	#[br(count = shader_params_count)]
	shader_params: Vec<(ShaderParamId, u16, u16)>,
	#[br(count = sampler_count)]
	samplers: Vec<(SamplerType, u32, u32)>,
	#[br(count = shader_param_values_size / 4)]
	shader_param_values: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct ColorsetRow {
	pub diffuse: [f32; 3],
	pub specular_strength: f32,
	pub specular: [f32; 3],
	pub gloss_strength: f32,
	pub emissive: [f32; 3],
	pub tile_index: i32,
	pub tile_repeat_x: f32,
	pub tile_skew_x: f32,
	pub tile_skew_y: f32,
	pub tile_repeat_y: f32,
}

impl Default for ColorsetRow {
	fn default() -> Self {
		Self {
			diffuse: [1.0; 3],
			specular_strength: 1.0,
			specular: [1.0; 3],
			gloss_strength: 1.0,
			emissive: [1.0; 3],
			tile_index: 0,
			tile_repeat_x: 1.0,
			tile_repeat_y: 1.0,
			tile_skew_x: 0.0,
			tile_skew_y: 0.0,
		}
	}
}

#[derive(Clone, Debug)]
pub struct ColorsetDyeRow {
	pub template: i32,
	pub diffuse: bool,
	pub specular: bool,
	pub emisive: bool,
	pub gloss: bool,
	pub specular_strength: bool,
}

impl Default for ColorsetDyeRow {
	fn default() -> Self {
		Self {
			template: 0,
			diffuse: false,
			specular: false,
			emisive: false,
			gloss: false,
			specular_strength: false,
		}
	}
}

#[derive(Clone, Debug)]
pub struct Mtrl {
	pub flags: u32,
	pub uvsets: Vec<String>,
	pub colorsets: Vec<String>,
	pub colorset_rows: Option<[ColorsetRow; 16]>,
	pub colorsetdye_rows: Option<[ColorsetDyeRow; 16]>,
	pub shader: Shader,
}

impl ironworks::file::File for Mtrl {
	fn read<'a>(data: impl Into<Cow<'a, [u8]>>) -> Result<Self> {
		Ok(Mtrl::read(&mut Cursor::new(&data.into())).unwrap())
	}
}

impl Mtrl {
	pub fn read<T>(reader: &mut T) -> Result<Self, Error> where
	T: Read + Seek {
		let data = <Data as BinRead>::read(reader)?;
		
		Ok(Mtrl {
			flags: data.flags,
			uvsets: data.uvsets.into_iter() // TODO: dont assume its in the right order use v.1
				.map(|v| data.strings[v.0 as usize..].null_terminated())
				.collect::<Result<Vec<_>, _>>()?,
			colorsets: data.colorsets.into_iter() // TODO: dont assume its in the right order use v.1
				.map(|v| data.strings[v.0 as usize..].null_terminated())
				.collect::<Result<Vec<_>, _>>()?,
			// unk: data.unk,
			colorset_rows: if data.colorset_data_size > 0 {
				let datas: [ColorsetRow; 16] = data.colorset_datas.into_iter().map(|v| ColorsetRow {
						diffuse: [
							f16::from_bits(v[0]).to_f32(),
							f16::from_bits(v[1]).to_f32(),
							f16::from_bits(v[2]).to_f32(),
						],
						specular_strength: f16::from_bits(v[3]).to_f32(),
						specular: [
							f16::from_bits(v[4]).to_f32(),
							f16::from_bits(v[5]).to_f32(),
							f16::from_bits(v[6]).to_f32(),
						],
						gloss_strength: f16::from_bits(v[7]).to_f32(),
						emissive: [
							f16::from_bits(v[8]).to_f32(),
							f16::from_bits(v[9]).to_f32(),
							f16::from_bits(v[10]).to_f32(),
						],
						tile_index: (f16::from_bits(v[11]).to_f32() * 64.0) as i32,
						tile_repeat_x: f16::from_bits(v[12]).to_f32(),
						tile_skew_x: f16::from_bits(v[13]).to_f32(),
						tile_skew_y: f16::from_bits(v[14]).to_f32(),
						tile_repeat_y: f16::from_bits(v[15]).to_f32(),
					}).collect::<Vec<ColorsetRow>>().try_into().unwrap();
				
				Some(datas)
			} else {None},
			colorsetdye_rows: if data.colorset_data_size == 544 {
				let datas: [ColorsetDyeRow; 16] = data.colorsetdye_datas.into_iter().map(|v| ColorsetDyeRow {
						template: (v >> 5) as i32,
						diffuse: (v & 0x01) == 0x01,
						specular: (v & 0x02) == 0x02,
						emisive: (v & 0x04) == 0x04,
						gloss: (v & 0x08) == 0x08,
						specular_strength: (v & 0x10) == 0x10,
					}).collect::<Vec<ColorsetDyeRow>>().try_into().unwrap();
				
				Some(datas)
			} else {None},
			shader: {
				let mut shader = Shader::new(&data.strings[data.shader_offset as usize..].null_terminated()?).ok_or("Invalid shader")?;
				let inner = shader.inner_mut();
				
				for (typ, offset, size) in data.shader_params {
					inner.params.insert(typ, ShaderParam {
						enabled: true,
						vals: data.shader_param_values[offset as usize / 4..offset as usize / 4 + size as usize / 4].to_vec(),
					});
				}
				
				for (typ, val) in data.shader_keys {
					inner.keys.insert(typ, ShaderKey {
						enabled: true,
						val,
					});
				}
				
				for (typ, flags, offset) in data.samplers {
					inner.samplers.insert(typ, Sampler {
						enabled: true,
						path: if offset == 255 {String::new()} else {data.strings[data.textures[offset as usize].0 as usize..].null_terminated()?},
						flags: flags,
					});
				}
				
				inner.unk = data.unk;
				
				shader
			},
			
			// shader_keys: data.shader_keys,
		})
	}
	
	pub fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		let mut strings = Vec::<u8>::new();
		
		strings.extend(self.shader.shader_name().bytes());
		strings.push(0);
		
		let mut textures = Vec::<(u16, u16)>::new();
		let mut samplers = Vec::<(u32, u32, u32)>::new();
		for (typ, sampler) in &self.shader.inner().samplers {
			if !sampler.enabled {continue}
			
			samplers.push((typ.clone() as u32, sampler.flags, textures.len() as u32));
			textures.push((strings.len() as u16, 0)); // we just assume the 2nd u16 is 0, since that seems to be the case most often than not
			strings.extend(sampler.path.as_bytes());
			strings.push(0);
		}
		
		let mut uvsets = Vec::<(u16, u16)>::new();
		for (i, s) in self.uvsets.iter().enumerate() {
			uvsets.push((strings.len() as u16, i as u16));
			strings.extend(s.as_bytes());
			strings.push(0);
		}
		
		let mut colorsets = Vec::<(u16, u16)>::new();
		for (i, s) in self.colorsets.iter().enumerate() {
			colorsets.push((strings.len() as u16, i as u16));
			strings.extend(s.as_bytes());
			strings.push(0);
		}
		
		16973824u32.write_le(writer)?;
		0u16.write_le(writer)?; // we go back to write the size after we write everything
		if self.colorset_rows.is_some() {544u16} else if self.colorset_rows.is_some() {512} else {0}.write_le(writer)?;
		(strings.len() as u16).write_le(writer)?;
		0u16.write_le(writer)?;
		(textures.len() as u8).write_le(writer)?;
		(uvsets.len() as u8).write_le(writer)?;
		(colorsets.len() as u8).write_le(writer)?;
		(self.shader.inner().unk.len() as u8 * 4).write_le(writer)?;
		textures.write_le(writer)?;
		uvsets.write_le(writer)?;
		colorsets.write_le(writer)?;
		strings.write_le(writer)?;
		self.shader.inner().unk.write_le(writer)?;
		if let Some(rows) = &self.colorset_rows {
			for row in rows {
				f16::from_f32(row.diffuse[0]).to_bits().write_le(writer)?;
				f16::from_f32(row.diffuse[1]).to_bits().write_le(writer)?;
				f16::from_f32(row.diffuse[2]).to_bits().write_le(writer)?;
				f16::from_f32(row.specular_strength).to_bits().write_le(writer)?;
				f16::from_f32(row.specular[0]).to_bits().write_le(writer)?;
				f16::from_f32(row.specular[1]).to_bits().write_le(writer)?;
				f16::from_f32(row.specular[2]).to_bits().write_le(writer)?;
				f16::from_f32(row.gloss_strength).to_bits().write_le(writer)?;
				f16::from_f32(row.emissive[0]).to_bits().write_le(writer)?;
				f16::from_f32(row.emissive[1]).to_bits().write_le(writer)?;
				f16::from_f32(row.emissive[2]).to_bits().write_le(writer)?;
				f16::from_f32(row.tile_index as f32 / 64.0).to_bits().write_le(writer)?;
				f16::from_f32(row.tile_repeat_x).to_bits().write_le(writer)?;
				f16::from_f32(row.tile_skew_x).to_bits().write_le(writer)?;
				f16::from_f32(row.tile_skew_y).to_bits().write_le(writer)?;
				f16::from_f32(row.tile_repeat_y).to_bits().write_le(writer)?;
			}
		}
		if let Some(rows) = &self.colorsetdye_rows {
			for row in rows {
				(
					((row.template as u16) << 5) +
					if row.diffuse {0x01} else {0} +
					if row.specular {0x02} else {0} +
					if row.emisive {0x04} else {0} +
					if row.gloss {0x08} else {0} +
					if row.specular_strength {0x10} else {0}
				).write_le(writer)?;
			}
		}
		
		let mut shader_param_values = Vec::<f32>::new();
		let mut shader_params = Vec::<(u32, u16, u16)>::new();
		for (typ, param) in &self.shader.inner().params {
			if !param.enabled {continue}
			
			shader_params.push((typ.clone() as u32, (shader_param_values.len() as u16) * 4, (param.vals.len() as u16) * 4));
			shader_param_values.extend(param.vals.iter());
		}
		
		let mut shader_keys = Vec::<(u32, u32)>::new();
		for (typ, key) in &self.shader.inner().keys {
			if !key.enabled {continue}
			
			shader_keys.push((typ.clone() as u32, key.val));
		}
		
		((shader_param_values.len() as u16) * 4).write_le(writer)?;
		(shader_keys.len() as u16).write_le(writer)?;
		(shader_params.len() as u16).write_le(writer)?;
		(samplers.len() as u16).write_le(writer)?;
		self.flags.write_le(writer)?;
		shader_keys.write_le(writer)?;
		shader_params.write_le(writer)?;
		samplers.write_le(writer)?;
		shader_param_values.write_le(writer)?;
		
		// write the size now
		let len = writer.stream_position()? as u16;
		writer.seek(std::io::SeekFrom::Start(4))?;
		len.write_le(writer)?;
		
		Ok(())
	}
}