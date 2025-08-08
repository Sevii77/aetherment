use std::collections::HashMap;
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
	
	fn key(s: &str) -> &str {
		let Some(f) = s.find('/') else {return s};
		let Some(f2) = s[f + 1..].find('/') else {return s};
		&s[..f + f2 + 1]
	}
	
	let mut sampler_types = HashMap::new();
	for path in paths.split("\n") {
		if !path.ends_with(".mtrl") {continue}
		
		// println!("{path}");
		let key = key(path);
		let Ok(mtrl) = noumenon.file::<Mtrl>(path) else {continue};
		for sampler in mtrl.samplers {
			sampler_types.entry(sampler.typ)
				.or_insert_with(|| HashMap::new())
				.entry(key)
				.or_insert_with(|| sampler.texture);
		}
	}
	
	println!("{sampler_types:#?}");
	// 290653886: {
	// 	"chara/accessory": "chara/accessory/a0151/texture/v01_c0101a0151_ear_d.tex",
	// 	"chara/equipment": "chara/equipment/e0028/texture/v04_c0101e0028_top_d.tex",
	// 	"chara/monster": "chara/monster/m0119/obj/body/b0008/texture/v01_m0119b0008_d.tex",
	// 	"chara/weapon": "chara/weapon/w0202/obj/body/b0004/texture/v06_w0202b0004_d.tex",
	// 	"chara/demihuman": "chara/demihuman/d0001/obj/equipment/e0000/texture/v01_d0001e0000_met_d.tex",
	// 	"chara/human": "chara/common/texture/eye/eye09_base.tex",
	// },
	// 3615719374: {
	// 	"bgcommon/hou": "bgcommon/hou/indoor/general/1498/texture/fun_b0_m1498_0b_d.tex",
	// 	"bg/ex5": "bg/ex5/01_xkt_x6/fld/x6fc/texture/x6fc_a7_ice02_d.tex",
	// },
	// 1464738518: {
	// 	"bg/ex3": "bgcommon/nature/water/texture/_n_wavelet_000.tex",
	// 	"bg/ex4": "bgcommon/nature/water/texture/_n_wavelet_000.tex",
	// 	"bg/ex2": "bgcommon/nature/water/texture/_n_wavelet_000.tex",
	// 	"bg/ffxiv": "bgcommon/nature/water/texture/_n_wavelet_000.tex",
	// 	"bg/ex5": "bgcommon/nature/water/texture/_n_wavelet_000.tex",
	// },
	// 4271961042: {
	// 	"chara/monster": "chara/monster/m0877/obj/body/b0001/texture/v01_m0877b0001_catc.tex",
	// 	"chara/equipment": "chara/equipment/e9242/texture/v01_c0201e9242_top_catc.tex",
	// 	"chara/demihuman": "chara/demihuman/d1003/obj/equipment/e0005/texture/v03_d1003e0005_top_base.tex",
	// },
	// 1824202628: {
	// 	"bg/ex2": "bg/ex2/01_gyr_g3/fld/g3fc/texture/g3fc_a2_mab05_s.tex",
	// 	"bg/ex1": "bgcommon/texture/dummy_s.tex",
	// 	"bgcommon/hou": "bgcommon/hou/indoor/general/0140/texture/fun_b0_m0140_0b_s.tex",
	// 	"bgcommon/nature": "bgcommon/texture/dummy_s.tex",
	// 	"bg/ffxiv": "bg/ffxiv/sea_s1/twn/common/texture/s1t0_v0_metl1_s.tex",
	// 	"bg/ex4": "bgcommon/texture/dummy_s.tex",
	// 	"bgcommon/world": "bgcommon/world/bah/001/texture/w_bah_001_swhu2_s.tex",
	// 	"bg/ex5": "bgcommon/texture/dummy_s.tex",
	// 	"bg/ex3": "bg/ex3/01_nvt_n4/evt/n4ed/texture/n4ed_q1_snw08_s.tex",
	// 	"bgcommon/mji": "bgcommon/mji/gath/0003/texture/gth_f0_m0003_0d_s.tex",
	// },
	// 3862043388: {
	// 	"bgcommon/mji": "bgcommon/nature/water/texture/_n_wave_000.tex",
	// 	"bgcommon/nature": "bgcommon/nature/water/texture/_n_wave_002.tex",
	// 	"bg/ex1": "bg/ex1/01_roc_r2/dun/r2d1/texture/r2d1_t0_rivr1_n.tex",
	// 	"bgcommon/world": "bgcommon/nature/water/texture/_n_wave_000.tex",
	// 	"bgcommon/hou": "bgcommon/hou/outdoor/general/0289/texture/_n_wave_101.tex",
	// 	"bg/ex2": "bgcommon/nature/water/texture/_n_wave_000.tex",
	// 	"bg/ffxiv": "bgcommon/nature/water/texture/_n_wave_000.tex",
	// 	"bg/ex4": "bgcommon/nature/water/texture/_n_wave_001.tex",
	// 	"bg/ex5": "bgcommon/nature/water/texture/_n_wave_000.tex",
	// 	"bg/ex3": "bgcommon/nature/water/texture/_n_wave_002.tex",
	// },
	// 2514613837: {
	// 	"bg/ex3": "bgcommon/nature/water/texture/_n_whitecap_001.tex",
	// 	"bgcommon/nature": "bgcommon/nature/water/texture/_n_whitecap_001.tex",
	// 	"bg/ex4": "bgcommon/nature/water/texture/_n_whitecap_001.tex",
	// 	"bgcommon/mji": "bgcommon/nature/water/texture/_n_whitecap_001.tex",
	// 	"bg/ffxiv": "bgcommon/nature/water/texture/_n_whitecap_w_000.tex",
	// 	"bg/ex2": "bgcommon/nature/water/texture/_n_whitecap_000.tex",
	// 	"bgcommon/world": "bgcommon/world/itm/050/texture/_n_whitecap_w_000.tex",
	// 	"bg/ex1": "bg/ex1/01_roc_r2/dun/r2d1/texture/_n_whitecap_r2d1.tex",
	// 	"bgcommon/hou": "bgcommon/hou/outdoor/general/0289/texture/_n_whitecap_r_000.tex",
	// 	"bg/ex5": "bgcommon/nature/water/texture/_n_whitecap_000.tex",
	// },
	// 2816579574: {
	// 	"chara/weapon": "chara/weapon/w9001/obj/body/b0431/texture/v01_w9001b0431_flow.tex",
	// 	"chara/monster": "chara/monster/m0888/obj/body/b0001/texture/v01_m0888b0001_flow.tex",
	// 	"chara/demihuman": "chara/demihuman/d1006/obj/equipment/e0002/texture/v01_d1006e0002_top_flow.tex",
	// 	"chara/equipment": "chara/equipment/e6197/texture/v01_c0101e6197_sho_flow.tex",
	// 	"chara/accessory": "chara/accessory/a0171/texture/v01_c0101a0171_nek_flow.tex",
	// },
	// 3719555455: {
	// 	"bg/ffxiv": "bg/ffxiv/sea_s1/twn/common/texture/s1t0_v0_metl1_n.tex",
	// 	"bg/ex1": "bgcommon/texture/dummy_n.tex",
	// 	"bg/ex2": "bg/ex2/01_gyr_g3/fld/g3fc/texture/g3fc_a2_mab05_n.tex",
	// 	"bgcommon/nature": "bgcommon/nature/earth/texture/earth02_n.tex",
	// 	"bgcommon/hou": "bgcommon/hou/indoor/general/0140/texture/fun_b0_m0140_0b_n.tex",
	// 	"bg/ex3": "bg/ex3/01_nvt_n4/evt/n4ed/texture/n4ed_q1_snw08_n.tex",
	// 	"bg/ex5": "bgcommon/texture/dummy_n.tex",
	// 	"bg/ex4": "bgcommon/texture/dummy_n.tex",
	// 	"bgcommon/world": "bgcommon/world/bah/001/texture/w_bah_001_swhu2_n.tex",
	// 	"bgcommon/mji": "bgcommon/mji/gath/0003/texture/gth_f0_m0003_0d_n.tex",
	// },
	// 465317650: {
	// 	"bg/ex4": "bg/ex4/01_nvt_n5/rad/n5r7/texture/n5r7_a1_ston1_s.tex",
	// 	"bgcommon/collision": "bgcommon/texture/dummy_s.tex",
	// 	"bgcommon/hou": "bgcommon/hou/indoor/general/0720/texture/fun_b0_m0720_0a_s.tex",
	// 	"bgcommon/mji": "bgcommon/mji/gath/0007/texture/gth_f0_m0007_0a_s.tex",
	// 	"bg/ex5": "bgcommon/texture/dummy_s.tex",
	// 	"bg/ex1": "bg/ex1/01_roc_r2/dun/r2d1/texture/r2d1_u1_ish01_s.tex",
	// 	"bg/ex2": "bg/ex2/01_gyr_g3/evt/g3e3/texture/g3e3_a1_pill1_s.tex",
	// 	"bg/ex3": "bg/ex3/01_nvt_n4/twn/n4t1/texture/n4t1_l1_lig02_s.tex",
	// 	"bgcommon/nature": "bgcommon/texture/dummy_s.tex",
	// 	"bg/ffxiv": "bg/ffxiv/sea_s1/twn/common/texture/s1t0_v0_metl1_s.tex",
	// 	"bgcommon/world": "bgcommon/world/sys/002/texture/w_sys_002_kei2_s.tex",
	// },
	// 557626425: {
	// 	"bg/ex1": "bgcommon/nature/lightshaft/texture/_n_lightshaft_s1_000.tex",
	// 	"bgcommon/mji": "bgcommon/mji/buil/common/texture/com_10_m0001_d.tex",
	// 	"bg/ffxiv": "bg/ffxiv/wil_w1/dun/w1d5/texture/w1d5_u8_lsf09_d.tex",
	// 	"bg/ex5": "bg/ex5/01_xkt_x6/twn/x6t1/texture/x6t1_j1_scr01_d.tex",
	// 	"bgcommon/world": "bgcommon/nature/lightshaft/texture/_n_lightshaft_s0_000.tex",
	// 	"bg/ex4": "bg/ex4/01_nvt_n5/rad/n5r7/texture/_n5r7_a0_lsf00_d.tex",
	// 	"bg/ex2": "bgcommon/nature/lightshaft/texture/_n_lightshaft_s0_000.tex",
	// 	"bg/ex3": "bg/ex3/01_nvt_n4/dun/n4d9/texture/n4d9_a0_lsf01_i.tex",
	// 	"bgcommon/hou": "bgcommon/nature/lightshaft/texture/_n_lightshaft_s0_000.tex",
	// },
	// 2285931524: {
	// 	"bg/ffxiv": "bgcommon/nature/verticalfog/texture/_n_verticalfog_000.tex",
	// 	"bg/ex4": "bg/ex4/01_nvt_n5/rad/n5r7/texture/n5r7_a0_vfg01_d.tex",
	// 	"bg/ex2": "bgcommon/nature/verticalfog/texture/_n_verticalfog_000.tex",
	// 	"bg/ex3": "bg/ex3/01_nvt_n4/goe/n4g9/texture/n4g9_a3_nat01_d.tex",
	// 	"bg/ex1": "bgcommon/nature/verticalfog/texture/_n_verticalfog_000.tex",
	// 	"bg/ex5": "bgcommon/nature/verticalfog/texture/_n_verticalfog_000.tex",
	// },
	// 731504677: {
	// 	"chara/human": "",
	// },
	// 207536625: {
	// 	"chara/monster": "chara/monster/m0119/obj/body/b0008/texture/v01_m0119b0008_n.tex",
	// 	"chara/accessory": "chara/accessory/a0143/texture/v01_c0101a0143_nek_n.tex",
	// 	"chara/equipment": "chara/equipment/e6221/texture/v01_c0201e6221_top_norm.tex",
	// 	"chara/weapon": "chara/weapon/w0501/obj/body/b0003/texture/v07_w0501b0003_n.tex",
	// 	"chara/demihuman": "chara/demihuman/d0001/obj/equipment/e0000/texture/v01_d0001e0000_met_n.tex",
	// 	"chara/human": "dummy.tex",
	// },
	// 1768480522: {
	// 	"bg/ex5": "bgcommon/texture/dummy_d.tex",
	// 	"bg/ex2": "bg/ex2/01_gyr_g3/fld/g3fc/texture/g3fc_a2_mab05_d.tex",
	// 	"bg/ex1": "bgcommon/texture/dummy_d.tex",
	// 	"bgcommon/mji": "bgcommon/mji/gath/0003/texture/gth_f0_m0003_0d_d.tex",
	// 	"bg/ex3": "bg/ex3/01_nvt_n4/evt/n4ed/texture/n4ed_q1_snw08_d.tex",
	// 	"bgcommon/nature": "bgcommon/nature/earth/texture/earth02_d.tex",
	// 	"bgcommon/hou": "bgcommon/hou/indoor/general/0140/texture/fun_b0_m0140_0b_d.tex",
	// 	"bgcommon/world": "bgcommon/world/bah/001/texture/w_bah_001_swhu2_d.tex",
	// 	"bg/ex4": "bgcommon/texture/dummy_d.tex",
	// 	"bg/ffxiv": "bg/ffxiv/sea_s1/twn/common/texture/s1t0_v0_metl1_d.tex",
	// },
	// 541659712: {
	// 	"bg/ex3": "bgcommon/nature/water/texture/_n_wavelet_000.tex",
	// 	"bg/ffxiv": "bgcommon/nature/water/texture/_n_wavelet_000.tex",
	// 	"bg/ex2": "bgcommon/nature/water/texture/_n_wavelet_000.tex",
	// 	"bg/ex4": "bgcommon/nature/water/texture/_n_wavelet_000.tex",
	// 	"bg/ex5": "bgcommon/nature/water/texture/_n_wavelet_000.tex",
	// },
	// 1449103320: {
	// 	"chara/monster": "chara/monster/m0119/obj/body/b0008/texture/v01_m0119b0008_id.tex",
	// 	"chara/equipment": "chara/equipment/e6221/texture/v01_c0101e6221_top_id.tex",
	// 	"chara/accessory": "chara/accessory/a0143/texture/v01_c0101a0143_nek_id.tex",
	// 	"chara/demihuman": "chara/demihuman/d0001/obj/equipment/e0000/texture/v01_d0001e0000_met_id.tex",
	// 	"chara/weapon": "chara/weapon/w0501/obj/body/b0003/texture/v07_w0501b0003_id.tex",
	// 	"chara/human": "dummy.tex",
	// },
	// 2863978985: {
	// 	"bg/ffxiv": "bg/ffxiv/sea_s1/twn/common/texture/s1t0_v0_metl1_n.tex",
	// 	"bg/ex4": "bg/ex4/01_nvt_n5/rad/n5r7/texture/n5r7_a1_ston1_n.tex",
	// 	"bgcommon/mji": "bgcommon/mji/gath/0007/texture/gth_f0_m0007_0a_n.tex",
	// 	"bg/ex2": "bg/ex2/01_gyr_g3/evt/g3e3/texture/g3e3_a1_pill1_n.tex",
	// 	"bg/ex5": "bgcommon/texture/dummy_n.tex",
	// 	"bg/ex3": "bg/ex3/01_nvt_n4/twn/n4t1/texture/n4t1_l1_lig02_n.tex",
	// 	"bgcommon/world": "bgcommon/world/sys/002/texture/w_sys_002_kei2_n.tex",
	// 	"bgcommon/collision": "bgcommon/texture/dummy_n.tex",
	// 	"bgcommon/hou": "bgcommon/hou/indoor/general/0720/texture/fun_b0_m0720_0a_n.tex",
	// 	"bg/ex1": "bg/ex1/01_roc_r2/dun/r2d1/texture/r2d1_u1_ish01_n.tex",
	// 	"bgcommon/nature": "bgcommon/nature/earth/texture/earth01_n.tex",
	// },
	// 4174878074: {
	// 	"bg/ex2": "bg/ex2/01_gyr_g3/fld/g3f1/texture/g3f1_b3_env01_e.tex",
	// 	"bg/ffxiv": "bg/ffxiv/wil_w1/hou/w1h1/texture/w1h1_a1_gold1_e.tex",
	// 	"bg/ex1": "bgcommon/nature/envmap/texture/_n_envmap_004.tex",
	// 	"bg/ex4": "bgcommon/nature/envmap/texture/_n_envmap_004.tex",
	// 	"bgcommon/hou": "bgcommon/hou/indoor/aqa/common/texture/_n_envmap_002.tex",
	// 	"bg/ex5": "bgcommon/nature/envmap/texture/_n_envmap_004.tex",
	// 	"bg/ex3": "bgcommon/nature/envmap/texture/_n_envmap_004.tex",
	// 	"bgcommon/world": "bgcommon/nature/envmap/texture/_n_envmap_004.tex",
	// },
	// 1446741167: {
	// 	"bg/ex4": "bg/ex4/01_nvt_n5/rad/n5r7/texture/_n5r7_a0_lsf00_d.tex",
	// 	"bg/ex1": "bgcommon/nature/lightshaft/texture/_n_lightshaft_s1_000.tex",
	// 	"bg/ex5": "bg/ex5/01_xkt_x6/twn/x6t1/texture/x6t1_j1_scr02_d.tex",
	// 	"bg/ex3": "bg/ex3/01_nvt_n4/dun/n4d9/texture/n4d9_a0_lsf02_i.tex",
	// 	"bgcommon/world": "bgcommon/nature/lightshaft/texture/_n_lightshaft_s1_000.tex",
	// 	"bgcommon/mji": "bgcommon/mji/buil/common/texture/com_10_m0002_d.tex",
	// 	"bgcommon/hou": "bgcommon/nature/lightshaft/texture/_n_lightshaft_s1_000.tex",
	// 	"bg/ex2": "bgcommon/nature/lightshaft/texture/_n_lightshaft_s1_000.tex",
	// 	"bg/ffxiv": "bgcommon/nature/lightshaft/texture/_n_lightshaft_s1_000.tex",
	// },
	// 2320401078: {
	// 	"chara/accessory": "chara/accessory/a0143/texture/v01_c0101a0143_nek_m.tex",
	// 	"chara/demihuman": "chara/demihuman/d0001/obj/equipment/e0000/texture/v01_d0001e0000_met_s.tex",
	// 	"chara/weapon": "chara/weapon/w0501/obj/body/b0003/texture/v07_w0501b0003_m.tex",
	// 	"chara/monster": "chara/monster/m0119/obj/body/b0008/texture/v01_m0119b0008_s.tex",
	// 	"chara/human": "dummy.tex",
	// 	"chara/equipment": "chara/equipment/e6221/texture/v01_c0201e6221_top_mask.tex",
	// },
	// 510652316: {
	// 	"bg/ex1": "bg/ex1/01_roc_r2/dun/r2d1/texture/r2d1_u1_ish01_d.tex",
	// 	"bg/ffxiv": "bg/ffxiv/sea_s1/twn/common/texture/s1t0_v0_metl1_d.tex",
	// 	"bg/ex3": "bg/ex3/01_nvt_n4/twn/n4t1/texture/n4t1_l1_lig02_d.tex",
	// 	"bg/ex5": "bg/ex5/01_xkt_x6/twn/x6t1/texture/x6t1_j1_leaf2_d.tex",
	// 	"bgcommon/collision": "bgcommon/collision/texture/id_d.tex",
	// 	"bgcommon/mji": "bgcommon/mji/gath/0007/texture/gth_f0_m0007_0a_d.tex",
	// 	"bg/ex2": "bg/ex2/01_gyr_g3/evt/g3e3/texture/g3e3_a1_pill1_d.tex",
	// 	"bgcommon/hou": "bgcommon/hou/indoor/general/0720/texture/fun_b0_m0720_0a_d.tex",
	// 	"bgcommon/world": "bgcommon/world/sys/002/texture/w_sys_002_kei2_d.tex",
	// 	"bgcommon/nature": "bgcommon/nature/earth/texture/earth01_d.tex",
	// 	"bg/ex4": "bg/ex4/01_nvt_n5/rad/n5r7/texture/n5r7_a1_ston1_d.tex",
	// },
	// 2281064269: {
	// 	"bg/ex2": "bg/ex2/01_gyr_g3/dun/g3d5/texture/g3d5_x9_refl1_f.tex",
	// 	"bg/ex5": "bg/ex5/02_ykt_y6/evt/y6e3/texture/y6e3_w1_wate1_f.tex",
	// 	"bg/ex4": "bg/ex4/02_mid_m5/fld/m5f1/texture/m5f1_w1_sea01_f.tex",
	// 	"bg/ex3": "bg/ex3/01_nvt_n4/evt/n4eb/texture/n4eb_g1_sea01_f.tex",
	// 	"bg/ffxiv": "bg/ffxiv/sea_s1/hou/s1h1/texture/s1h1_w1_wate1_f.tex",
	// 	"bg/ex1": "bg/ex1/03_abr_a2/dun/a2d2/texture/a2d2_w1_wate1_f.tex",
	// },
	// 3845360663: {
	// 	"bg/ex2": "bgcommon/nature/water/texture/_n_wave_200.tex",
	// 	"bgcommon/world": "bgcommon/nature/water/texture/_n_wave_200.tex",
	// 	"bg/ex4": "bgcommon/nature/water/texture/_n_wave_200.tex",
	// 	"bg/ex1": "bgcommon/nature/water/texture/_n_wave_200.tex",
	// 	"bgcommon/hou": "bgcommon/hou/common/water/texture/_n_wave_200a.tex",
	// 	"bg/ex5": "",
	// 	"bg/ex3": "bgcommon/nature/water/texture/_n_wave_200.tex",
	// 	"bg/ffxiv": "bgcommon/nature/water/texture/_n_wave_200.tex",
	// 	"bgcommon/mji": "bgcommon/nature/water/texture/_n_wave_200.tex",
	// },
}