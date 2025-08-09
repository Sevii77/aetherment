use std::collections::{HashMap, HashSet};
use noumenon::format::game::Mtrl;

fn main() {
	let mut args = std::env::args();
	if args.len() != 3 {
		println!("Usage: mtrl GAME_DIR PATH_LIST");
		println!("\tPATH_LIST: Newline seperated list txt (https://rl2.perchbird.dev/download/CurrentPathList.gz)");
		return;
	}
	
	_ = args.next();
	let noumenon = noumenon::get_noumenon(args.next()).expect("Game dir was invalid");
	let paths = std::fs::read_to_string(args.next().unwrap()).expect("Path list was invalid");
	
	let mut sampler_types = HashMap::new();
	let mut used_samplers = HashMap::new();
	for path in paths.split("\n") {
		if !path.ends_with(".mtrl") {continue}
		
		// println!("{path}");
		let Ok(mtrl) = noumenon.file::<Mtrl>(path) else {continue};
		let shader_entry = used_samplers.entry(mtrl.shader.clone())
			.or_insert_with(|| HashSet::new());
		
		for sampler in mtrl.samplers {
			shader_entry.insert(sampler.id);
			
			sampler_types.entry(sampler.id)
				.or_insert_with(|| HashMap::new())
				.entry(mtrl.shader.clone())
				.or_insert_with(|| sampler.texture);
		}
	}
	
	// println!("{used_samplers:#?}");
	// "bgprop.shpk": {
	// 	510652316,
	// 	2863978985,
	// 	465317650,
	// },
	// "bguvscroll.shpk": {
	// 	510652316,
	// 	1768480522,
	// 	465317650,
	// 	2863978985,
	// 	3719555455,
	// 	1824202628,
	// },
	// "verticalfog.shpk": {
	// 	2285931524,
	// },
	// "crystal.shpk": {
	// 	3615719374,
	// 	465317650,
	// 	510652316,
	// 	4174878074,
	// 	2863978985,
	// },
	// "characterstockings.shpk": {
	// 	1449103320,
	// 	207536625,
	// 	2320401078,
	// },
	// "character.shpk": {
	// 	731504677,
	// 	207536625,
	// 	1449103320,
	// 	2816579574,
	// 	290653886,
	// 	2320401078,
	// },
	// "characterinc.shpk": {
	// 	207536625,
	// 	2320401078,
	// 	290653886,
	// 	1449103320,
	// },
	// "bgcolorchange.shpk": {
	// 	510652316,
	// 	465317650,
	// 	2863978985,
	// },
	// "skin.shpk": {
	// 	207536625,
	// 	2320401078,
	// 	290653886,
	// },
	// "characterglass.shpk": {
	// 	207536625,
	// 	1449103320,
	// 	2320401078,
	// 	290653886,
	// },
	// "lightshaft.shpk": {
	// 	557626425,
	// 	1446741167,
	// },
	// "characterlegacy.shpk": {
	// 	290653886,
	// 	207536625,
	// 	2320401078,
	// 	1449103320,
	// },
	// "water.shpk": {
	// 	1464738518,
	// 	541659712,
	// 	2514613837,
	// 	2281064269,
	// 	3862043388,
	// 	3845360663,
	// },
	// "charactertattoo.shpk": {
	// 	207536625,
	// },
	// "bg.shpk": {
	// 	465317650,
	// 	2863978985,
	// 	510652316,
	// 	1768480522,
	// 	1824202628,
	// 	3719555455,
	// },
	// "characterscroll.shpk": {
	// 	2816579574,
	// 	207536625,
	// 	290653886,
	// 	2320401078,
	// 	1449103320,
	// 	4271961042,
	// },
	// "characterocclusion.shpk": {
	// 	207536625,
	// },
	// "charactertransparency.shpk": {
	// 	207536625,
	// 	2320401078,
	// 	1449103320,
	// 	290653886,
	// },
	// "characterreflection.shpk": {
	// 	207536625,
	// 	4271961042,
	// 	2320401078,
	// },
	// "bgcrestchange.shpk": {
	// 	2863978985,
	// 	1768480522,
	// 	510652316,
	// 	465317650,
	// },
	// "hair.shpk": {
	// 	2320401078,
	// 	207536625,
	// },
	// "iris.shpk": {
	// 	2320401078,
	// 	290653886,
	// 	207536625,
	// },
	// "river.shpk": {
	// 	3862043388,
	// 	2281064269,
	// 	2514613837,
	// },
	
	// println!("{sampler_types:#?}");
	// 1768480522: {
	// 	"bguvscroll.shpk": "bg/ex1/02_dra_d2/alx/common/texture/d2a0_a9_wat01_d.tex",
	// 	"bgcrestchange.shpk": "bgcommon/hou/indoor/general/0413/texture/air_crst_dummy_00.tex",
	// 	"bg.shpk": "bgcommon/texture/dummy_d.tex",
	// },
	// 207536625: {
	// 	"characterstockings.shpk": "chara/equipment/e6208/texture/v01_c1101e6208_dwn_norm.tex",
	// 	"charactertattoo.shpk": "chara/human/c1001/obj/face/f0202/texture/c1001f0202_etc_norm.tex",
	// 	"iris.shpk": "chara/common/texture/eye/eye01_norm.tex",
	// 	"hair.shpk": "chara/human/c1001/obj/face/f0202/texture/c1001f0202_etc_norm.tex",
	// 	"characterglass.shpk": "chara/equipment/e5528/texture/v01_c0101e5528_met_norm.tex",
	// 	"characterscroll.shpk": "chara/weapon/w9206/obj/body/b0001/texture/v01_w9206b0001_norm.tex",
	// 	"character.shpk": "chara/equipment/e6221/texture/v01_c0201e6221_top_norm.tex",
	// 	"characterreflection.shpk": "chara/demihuman/d1003/obj/equipment/e0005/texture/v03_d1003e0005_top_norm.tex",
	// 	"skin.shpk": "chara/human/c1001/obj/face/f0202/texture/c1001f0202_fac_norm.tex",
	// 	"characterlegacy.shpk": "chara/weapon/w0501/obj/body/b0003/texture/v07_w0501b0003_n.tex",
	// 	"charactertransparency.shpk": "chara/monster/m8168/obj/body/b0001/texture/v01_m8168b0001_b_n.tex",
	// 	"characterocclusion.shpk": "chara/human/c1001/obj/face/f0202/texture/c1001f0202_etc_norm.tex",
	// 	"characterinc.shpk": "chara/weapon/w1930/obj/body/b0001/texture/v01_w1930b0001_norm.tex",
	// },
	// 2816579574: {
	// 	"characterscroll.shpk": "chara/demihuman/d1071/obj/equipment/e0006/texture/v01_d1071e0006_top_d_flow.tex",
	// 	"character.shpk": "chara/monster/m0888/obj/body/b0001/texture/v01_m0888b0001_flow.tex",
	// },
	// 4271961042: {
	// 	"characterreflection.shpk": "chara/demihuman/d1003/obj/equipment/e0005/texture/v03_d1003e0005_top_base.tex",
	// 	"characterscroll.shpk": "chara/monster/m0877/obj/body/b0001/texture/v01_m0877b0001_catc.tex",
	// },
	// 731504677: {
	// 	"character.shpk": "",
	// },
	// 2320401078: {
	// 	"iris.shpk": "chara/common/texture/eye/eye01_mask.tex",
	// 	"characterglass.shpk": "chara/equipment/e5528/texture/v01_c0101e5528_met_mask.tex",
	// 	"characterscroll.shpk": "chara/weapon/w9206/obj/body/b0001/texture/v01_w9206b0001_mask.tex",
	// 	"character.shpk": "chara/equipment/e6221/texture/v01_c0201e6221_top_mask.tex",
	// 	"charactertransparency.shpk": "chara/monster/m8168/obj/body/b0001/texture/v01_m8168b0001_s.tex",
	// 	"characterlegacy.shpk": "chara/weapon/w0501/obj/body/b0003/texture/v07_w0501b0003_m.tex",
	// 	"characterstockings.shpk": "chara/equipment/e6208/texture/v01_c1101e6208_dwn_mask.tex",
	// 	"hair.shpk": "chara/human/c1001/obj/face/f0202/texture/c1001f0202_etc_mask.tex",
	// 	"characterinc.shpk": "chara/weapon/w1930/obj/body/b0001/texture/v01_w1930b0001_mask.tex",
	// 	"characterreflection.shpk": "chara/demihuman/d1003/obj/equipment/e0005/texture/v03_d1003e0005_top_mask.tex",
	// 	"skin.shpk": "chara/human/c1001/obj/face/f0202/texture/c1001f0202_fac_mask.tex",
	// },
	// 3719555455: {
	// 	"bg.shpk": "bgcommon/texture/dummy_n.tex",
	// 	"bguvscroll.shpk": "bg/ex1/02_dra_d2/alx/common/texture/d2a0_a9_wat01_n.tex",
	// },
	// 3862043388: {
	// 	"river.shpk": "bg/ex1/01_roc_r2/dun/r2d1/texture/r2d1_t0_rivr1_n.tex",
	// 	"water.shpk": "bgcommon/nature/water/texture/_n_wave_000.tex",
	// },
	// 2285931524: {
	// 	"verticalfog.shpk": "bgcommon/nature/verticalfog/texture/_n_verticalfog_000.tex",
	// },
	// 3845360663: {
	// 	"water.shpk": "bgcommon/nature/water/texture/_n_wave_200.tex",
	// },
	// 557626425: {
	// 	"lightshaft.shpk": "bgcommon/nature/lightshaft/texture/_n_lightshaft_s1_000.tex",
	// },
	// 1446741167: {
	// 	"lightshaft.shpk": "bgcommon/nature/lightshaft/texture/_n_lightshaft_s1_000.tex",
	// },
	// 3615719374: {
	// 	"crystal.shpk": "bg/ex5/01_xkt_x6/fld/x6fc/texture/x6fc_a7_ice02_d.tex",
	// },
	// 1449103320: {
	// 	"character.shpk": "chara/equipment/e6221/texture/v01_c0101e6221_top_id.tex",
	// 	"charactertransparency.shpk": "chara/common/texture/id_16.tex",
	// 	"characterinc.shpk": "chara/weapon/w1930/obj/body/b0001/texture/v01_w1930b0001_id.tex",
	// 	"characterlegacy.shpk": "chara/weapon/w0501/obj/body/b0003/texture/v07_w0501b0003_id.tex",
	// 	"characterstockings.shpk": "chara/equipment/e6208/texture/v01_c1101e6208_dwn_id.tex",
	// 	"characterscroll.shpk": "chara/weapon/w9206/obj/body/b0001/texture/v01_w9206b0001_id.tex",
	// 	"characterglass.shpk": "chara/equipment/e5528/texture/v01_c0101e5528_met_id.tex",
	// },
	// 290653886: {
	// 	"characterinc.shpk": "chara/weapon/w1930/obj/body/b0001/texture/v01_w1930b0001_base.tex",
	// 	"skin.shpk": "chara/human/c1001/obj/face/f0202/texture/c1001f0202_fac_base.tex",
	// 	"character.shpk": "chara/accessory/a0167/texture/v01_c0101a0167_nek_base.tex",
	// 	"characterlegacy.shpk": "chara/weapon/w0202/obj/body/b0004/texture/v06_w0202b0004_d.tex",
	// 	"characterglass.shpk": "chara/equipment/e5513/texture/v01_c0101e5513_met_base.tex",
	// 	"iris.shpk": "chara/common/texture/eye/eye09_base.tex",
	// 	"charactertransparency.shpk": "chara/monster/m8168/obj/body/b0001/texture/v01_m8168b0001_d.tex",
	// 	"characterscroll.shpk": "chara/weapon/w9206/obj/body/b0001/texture/v01_w9206b0001_base.tex",
	// },
	// 541659712: {
	// 	"water.shpk": "bgcommon/nature/water/texture/_n_wavelet_000.tex",
	// },
	// 1464738518: {
	// 	"water.shpk": "bgcommon/nature/water/texture/_n_wavelet_000.tex",
	// },
	// 465317650: {
	// 	"bguvscroll.shpk": "bg/ex1/02_dra_d2/alx/common/texture/d2a0_a5_glas2_s.tex",
	// 	"bg.shpk": "bg/ex1/01_roc_r2/dun/r2d1/texture/r2d1_u1_ish01_s.tex",
	// 	"crystal.shpk": "bg/ex1/02_dra_d2/fld/d2f2/texture/d2f2_b0_glass4_s.tex",
	// 	"bgprop.shpk": "bgcommon/texture/dummy_s.tex",
	// 	"bgcrestchange.shpk": "bgcommon/texture/dummy_s.tex",
	// 	"bgcolorchange.shpk": "bgcommon/hou/indoor/general/0584/texture/fun_b0_m0584_0b_s.tex",
	// },
	// 510652316: {
	// 	"bguvscroll.shpk": "bg/ex1/02_dra_d2/alx/common/texture/d2a0_a7_gim07_d.tex",
	// 	"bg.shpk": "bg/ex1/01_roc_r2/dun/r2d1/texture/r2d1_u1_ish01_d.tex",
	// 	"crystal.shpk": "bg/ex1/02_dra_d2/fld/d2f2/texture/d2f2_b0_glass4_d.tex",
	// 	"bgcolorchange.shpk": "bgcommon/hou/indoor/general/0584/texture/fun_b0_m0584_0b_d.tex",
	// 	"bgcrestchange.shpk": "bgcommon/hou/indoor/general/0413/texture/air_com_m0000_1l_d.tex",
	// 	"bgprop.shpk": "bg/ex1/01_roc_r2/twn/common/texture/_r2t1_p1_pla01_d.tex",
	// },
	// 2863978985: {
	// 	"bg.shpk": "bg/ex1/01_roc_r2/dun/r2d1/texture/r2d1_u1_ish01_n.tex",
	// 	"crystal.shpk": "bg/ex1/02_dra_d2/fld/d2f2/texture/d2f2_b0_glass4_n.tex",
	// 	"bguvscroll.shpk": "bgcommon/texture/dummy_n.tex",
	// 	"bgcolorchange.shpk": "bgcommon/hou/indoor/general/0584/texture/fun_b0_m0584_0b_n.tex",
	// 	"bgprop.shpk": "bgcommon/texture/dummy_n.tex",
	// 	"bgcrestchange.shpk": "bgcommon/texture/dummy_n.tex",
	// },
	// 2514613837: {
	// 	"water.shpk": "bgcommon/nature/water/texture/_n_whitecap_000.tex",
	// 	"river.shpk": "bg/ex1/01_roc_r2/dun/r2d1/texture/_n_whitecap_r2d1.tex",
	// },
	// 4174878074: {
	// 	"crystal.shpk": "bgcommon/nature/envmap/texture/_n_envmap_004.tex",
	// },
	// 2281064269: {
	// 	"water.shpk": "bg/ex1/03_abr_a2/dun/a2d2/texture/a2d2_w1_wate1_f.tex",
	// 	"river.shpk": "bg/ex4/03_kld_k5/dun/k5d2/texture/k5d2_w2_wat01_f.tex",
	// },
	// 1824202628: {
	// 	"bg.shpk": "bgcommon/texture/dummy_s.tex",
	// 	"bguvscroll.shpk": "bg/ex1/02_dra_d2/alx/common/texture/d2a0_a9_wat01_s.tex",
	// },
}